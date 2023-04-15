use alloc::sync::{Arc, Weak};
use alloc::{boxed::Box, vec::Vec};
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, AtomicUsize, Ordering};

use super::manager::{TaskLockedCell, TASK_MANAGER};
use super::schedule::SchedulerState;
use super::wait_queue::WaitCurrent;
use crate::arch::{instructions, Context, ProcessContext, ProcessTrapFrame};
use crate::config::KERNEL_STACK_SIZE;
use crate::loader;
use crate::mm::{kernel_aspace, MemorySet, VirtAddr};
use crate::percpu::PerCpu;
use crate::sync::{LazyInit, Mutex};
use crate::timer::TimeValue;

// pub trait Task;

pub(super) static ROOT_TASK: LazyInit<Arc<dyn Task>> = LazyInit::new();

#[derive(Debug)]
enum EntryState {
    Kernel { pc: usize, arg: usize },
    User(Box<ProcessTrapFrame>),
}

/*
    TaskId 是当前 hypervisor 中任务编号
    对 Guest 和 Process 都适用
 */
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TaskId(usize);

impl TaskId {
    const IDLE_TASK_ID: Self = Self(0);

    fn alloc() -> Self {
        static NEXT_PID: AtomicUsize = AtomicUsize::new(1);
        Self(NEXT_PID.fetch_add(1, Ordering::AcqRel))
    }

    pub const fn as_usize(&self) -> usize {
        self.0
    }
}

impl From<usize> for TaskId {
    fn from(pid: usize) -> Self {
        Self(pid)
    }
}

/*
    TaskState 是当前 hypervisor 中任务状态
    对 Guest 和 Process 都适用  
 */
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TaskState {
    Ready = 1,
    Running = 2,
    Sleeping = 3,
    Zombie = 4,
}

impl From<u8> for TaskState {
    fn from(state: u8) -> Self {
        match state {
            1 => Self::Ready,
            2 => Self::Running,
            3 => Self::Sleeping,
            4 => Self::Zombie,
            _ => panic!("invalid task state: {}", state),
        }
    }
}

/*
    Task 派生出 Guest 和 Process
    TaskManager 中仅仅管理 dyn Task, 不管任何详细的东西
 */
pub trait Task {
    fn new_common(id: TaskId) -> Self;

    fn id(&self) -> TaskId;
    fn context(&self) -> &TaskLockedCell<dyn Context>;
    fn state(&self) -> TaskState;
    fn set_state(&self, state: TaskState);
    fn exit_code(&self) -> i32;
    fn set_exit_code(&self, exit_code: i32);

    // 以下是与Scheduler相关的函数
    fn need_resched(&self) -> bool;
    fn clear_need_resched(&self);
    fn set_need_resched(&self);
    fn sched_state(&self) -> &SchedulerState;

    fn process(&mut self) -> &mut Process;
    fn is_process(&self) -> bool;
    fn guest(&mut self) -> &mut Guest;
    fn is_guest(&self) -> bool;
}

pub struct Process {
    id: TaskId,
    is_kernel: bool,
    is_shared: bool,
    entry: EntryState,

    state: AtomicU8,
    exit_code: AtomicI32,
    need_resched: AtomicBool,
    sched_state: SchedulerState,

    kstack: Stack<KERNEL_STACK_SIZE>,
    ctx: TaskLockedCell<ProcessContext>,

    pub(super) wait_children_exit: WaitCurrent,

    vm: Option<Arc<Mutex<MemorySet>>>,
    pub(super) parent: Mutex<Weak<Process>>,
    pub(super) children: Mutex<Vec<Arc<Process>>>,
}

impl Task for Process {

    // 初始化

    fn new_common(id: TaskId) -> Self {
        Self {
            id,
            is_kernel: false,
            is_shared: false,
            entry: EntryState::Kernel { pc: 0, arg: 0 },

            state: AtomicU8::new(TaskState::Ready as u8),
            exit_code: AtomicI32::new(0),
            need_resched: AtomicBool::new(false),
            sched_state: SchedulerState::default(),

            kstack: Stack::default(),
            ctx: TaskLockedCell::new(ProcessContext::default()),

            wait_children_exit: WaitCurrent::new(),

            vm: None,
            parent: Mutex::new(Weak::default()),
            children: Mutex::new(Vec::new()),
        }
    }

    // 获得值或设置值

    fn id(&self) -> TaskId {
        self.id
    }

    fn state(&self) -> TaskState {
        self.state.load(Ordering::SeqCst).into()
    }

    fn context(&self) -> &TaskLockedCell<ProcessContext> {
        &self.ctx
    }

    fn set_state(&self, state: TaskState) {
        self.state.store(state as u8, Ordering::SeqCst)
    }

    fn exit_code(&self) -> i32 {
        self.exit_code.load(Ordering::SeqCst)
    }

    fn set_exit_code(&self, exit_code: i32) {
        self.exit_code.store(exit_code, Ordering::SeqCst)
    }

    // Scheduler 相关

    fn need_resched(&self) -> bool {
        self.need_resched.load(Ordering::SeqCst)
    }

    fn clear_need_resched(&self) {
        self.need_resched.store(false, Ordering::SeqCst);
    }

    fn set_need_resched(&self) {
        self.need_resched.store(true, Ordering::SeqCst);
    }

    fn sched_state(&self) -> &SchedulerState {
        &self.sched_state
    }

    // 转换类型

    fn process(&mut self) -> &mut Process {
        self
    }

    fn is_process(&self) -> bool {
        true
    }

    fn guest(&mut self) -> &mut Guest {
        panic!("This is not a guest")
    }

    fn is_guest(&self) -> bool {
        false
    }

}

impl Process { 

    pub(super) fn add_child(self: &Arc<Self>, child: &Arc<Process>) {
        *child.parent.lock() = Arc::downgrade(self);
        self.children.lock().push(child.clone());
    }

    pub fn new_idle() -> Arc<Self> {
        let mut t = Self::new_common(TaskId::IDLE_TASK_ID);
        t.is_kernel = true;
        Arc::new(t)
    }

    pub fn init_idle(&mut self) {
        self.set_state(TaskState::Running);
        self.ctx.get_mut().init(
            0,
            self.kstack.top(),
            kernel_aspace().page_table_root(),
            true,
        );
    }

    pub fn new_kernel(entry: fn(usize) -> usize, arg: usize) -> Arc<Self> {
        let mut t = Self::new_common(TaskId::alloc());
        t.is_kernel = true; // 唯一的内核进程？
        t.entry = EntryState::Kernel {
            pc: entry as usize, // 取出了函数指针
            arg,
        };
        t.ctx.get_mut().init(
            task_entry as _,
            t.kstack.top(),
            kernel_aspace().page_table_root(),
            true,
        );

        let t = Arc::new(t);
        if !t.is_root() {
            ROOT_TASK.add_child(&t);
        }
        t
    }

    pub fn new_user(path: &str) -> Arc<Self> {
        let elf_data = loader::get_app_data_by_name(path).expect("new_user: no such app");
        let mut vm = MemorySet::new();
        let (entry, ustack_top) = vm.load_user(elf_data);

        let mut t = Self::new_common(TaskId::alloc());
        t.entry = EntryState::User(Box::new(ProcessTrapFrame::new_user(entry, ustack_top, 0)));
        t.ctx
            .get_mut()
            .init(task_entry as _, t.kstack.top(), vm.page_table_root(), false);
        t.vm = Some(Arc::new(Mutex::new(vm)));

        let t = Arc::new(t);
        ROOT_TASK.add_child(&t);
        t
    }

    pub fn new_clone(self: &Arc<Self>, newsp: usize, tf: &ProcessTrapFrame) -> Arc<Self> {
        assert!(!self.is_kernel_task());
        let mut t = Self::new_common(TaskId::alloc());
        t.is_shared = true;
        let vm = self.vm.as_ref().unwrap().clone();
        t.entry = EntryState::User(Box::new(tf.new_clone(VirtAddr::new(newsp))));
        t.ctx.get_mut().init(
            task_entry as _,
            t.kstack.top(),
            vm.lock().page_table_root(),
            false,
        );
        t.vm = Some(vm);

        let t = Arc::new(t);
        self.add_child(&t);
        t
    }

    pub fn new_fork(self: &Arc<Self>, tf: &ProcessTrapFrame) -> Arc<Self> {
        assert!(!self.is_kernel_task());
        let mut t = Self::new_common(TaskId::alloc());
        let vm = self.vm.as_ref().unwrap().lock().dup();
        t.entry = EntryState::User(Box::new(tf.new_fork()));
        t.ctx
            .get_mut()
            .init(task_entry as _, t.kstack.top(), vm.page_table_root(), false);
        t.vm = Some(Arc::new(Mutex::new(vm)));

        let t = Arc::new(t);
        self.add_child(&t);
        t
    }

    pub const fn is_kernel_task(&self) -> bool {
        self.is_kernel
    }

    pub const fn is_root(&self) -> bool {
        self.id.as_usize() == 1
    }

    pub const fn is_idle(&self) -> bool {
        self.id.as_usize() == 0
    }

    pub const fn is_shared_with_parent(&self) -> bool {
        self.is_shared
    }

    pub(super) fn traverse(self: &Arc<Self>, func: &impl Fn(&Arc<Process>)) {
        func(self);
        for c in self.children.lock().iter() {
            c.traverse(func);
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        debug!("Process({}) dropped", self.pid().as_usize());
    }
}

pub struct Guest {

}

fn task_entry() -> ! {
    // release the lock that was implicitly held across the reschedule
    unsafe { TASK_MANAGER.force_unlock() };
    instructions::enable_irqs();
    let task = CurrentTask::get();
    match &task.entry {
        EntryState::Kernel { pc, arg } => {
            let entry: fn(usize) -> i32 = unsafe { core::mem::transmute(*pc) };
            let ret = entry(*arg);
            task.exit(ret as _);
        }
        EntryState::User(tf) => {
            unsafe { tf.exec(task.kstack.top()) };
        }
    }
}

/*
    以下实现的 current 函数主要是为了能够获得当前的Task
 */
pub struct CurrentTask<'a>(pub &'a Arc<dyn Task>);

impl<'a> CurrentTask<'a> {
    pub(super) fn get() -> Self {
        PerCpu::current_task()
    }

    pub fn clone_task(&self) -> Arc<dyn Task> {
        self.0.clone()
    }

    pub fn clear_need_resched(&self) {
        self.0.clear_need_resched();
    }

    pub fn set_need_resched(&self) {
        self.0.set_need_resched();
    }

    pub fn yield_now(&self) {
        // switch to next
        TASK_MANAGER.lock().yield_current(self);
    }

    pub fn sleep(&self, deadline: TimeValue) {
        TASK_MANAGER.lock().sleep_current(self, deadline);
    }

    pub fn exit(&self, exit_code: i32) -> ! {
        info!("task exit with code {}", exit_code);
        if let Some(vm) = self.vm.as_ref() {
            if Arc::strong_count(vm) == 1 {
                vm.lock().clear(); // drop memory set before lock
            }
        }
        TASK_MANAGER.lock().exit_current(self, exit_code)
    }

    /*
        目前可以 exec 一个 Guest 还有一个 Process
     */
    pub fn exec(&self, path: &str, tf: &mut ProcessTrapFrame) -> isize {
        assert!(!self.is_kernel_task()); // 唯一一个在 dyn Task 中使用 is_kernel_task 的地方
        assert_eq!(Arc::strong_count(self.vm.as_ref().unwrap()), 1);
        if let Some(elf_data) = loader::get_app_data_by_name(path) {
            let mut vm = self.vm.as_ref().unwrap().lock();
            vm.clear();
            let (entry, ustack_top) = vm.load_user(elf_data);
            *tf = ProcessTrapFrame::new_user(entry, ustack_top, 0);
            instructions::flush_tlb_all();
            0
        } else {
            -1
        }
    }

    /*
        目前也可以 waitid 一个 Guest 还有一个 Process
     */
    pub fn waitpid(&self, pid: isize, _options: u32) -> Option<(TaskId, i32)> {
        let mut found_pid = false;
        for t in self.children.lock().iter() {
            if pid == -1 || t.pid().as_usize() == pid as usize {
                found_pid = true;
                break;
            }
        }
        if !found_pid {
            return None;
        }

        loop {
            {
                let mut children = self.children.lock();
                for (idx, t) in children.iter().enumerate() {
                    if (pid == -1 || t.pid().as_usize() == pid as usize)
                        && t.state() == TaskState::Zombie
                    {
                        let child = children.remove(idx);
                        assert_eq!(Arc::strong_count(&child), 1);
                        return Some((child.pid(), child.exit_code()));
                    }
                }
            }
            self.wait_children_exit.wait();
        }
    }
}

impl<'a> core::ops::Deref for CurrentTask<'a> {
    type Target = Arc<dyn Task>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/*
    Stack 是一个特殊的数据结构, 为 Kernel Stack 提供服务
    TODO: 迁移到 utils 里
 */
struct Stack<const N: usize>(Box<[u8]>);

impl<const N: usize> Stack<N> {
    pub fn default() -> Self {
        Self(Box::from(alloc::vec![0; N]))
    }

    pub fn top(&self) -> VirtAddr {
        VirtAddr::new(self.0.as_ptr_range().end as usize)
    }
}
