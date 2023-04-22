use core::arch::asm;

use riscv::register::{sepc, sscratch};

use crate::arch::instructions;
use crate::mm::{PhysAddr, VirtAddr};

include_asm_marcos!();

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct GeneralRegisters {
    pub ra: usize,
    pub sp: usize,
    pub gp: usize, // only valid for user traps
    pub tp: usize, // only valid for user traps
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub s0: usize,
    pub s1: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TrapFrame {
    pub regs: GeneralRegisters,
    pub sepc: usize,
    pub sstatus: usize,
    pub hstatus: usize,
}

impl TrapFrame {
    pub fn new_user(entry: VirtAddr, ustack_top: VirtAddr, arg0: usize) -> Self {
        const SPIE: usize = 1 << 5;
        const SUM: usize = 1 << 18;
        Self {
            regs: GeneralRegisters {
                a0: arg0,
                sp: ustack_top.as_usize(),
                ..Default::default()
            },
            sepc: entry.as_usize(),
            sstatus: SPIE | SUM,
            hstatus: 0,
        }
    }

    pub fn new_guest(entry: VirtAddr, ustack_top: VirtAddr, arg0: usize) -> Self {
        const SPIE: usize = 1 << 5;
        const SPP: usize = 1 << 8;
        const SUM: usize = 1 << 18;
        const SPV: usize = 1 << 7;
        Self {
            regs: GeneralRegisters {
                a0: arg0,
                sp: ustack_top.as_usize(),
                ..Default::default()
            },
            sepc: entry.as_usize(),
            sstatus: SPIE | SUM | SPP,
            hstatus: SPV,
        }
    }

    pub const fn new_clone(&self, ustack_top: VirtAddr) -> Self {
        let mut tf = *self;
        tf.regs.sp = ustack_top.as_usize();
        tf.regs.a0 = 0; // for child thread, clone returns 0
        tf
    }

    pub const fn new_fork(&self) -> Self {
        let mut tf = *self;
        tf.regs.a0 = 0; // for child process, fork returns 0
        tf
    }

    pub unsafe fn exec(&self, kstack_top: VirtAddr) -> ! {
        info!(
            "user task start: entry={:#x}, ustack={:#x}, kstack={:#x}",
            self.sepc,
            self.regs.sp,
            kstack_top.as_usize(),
        );
        instructions::disable_irqs();
        sscratch::write(kstack_top.as_usize());
        sepc::write(self.sepc);
        let kernel_tp_addr = kstack_top.as_usize() - core::mem::size_of::<TrapFrame>()
            + memoffset::offset_of!(GeneralRegisters, tp);
        asm!("
            mv      sp, {tf}

            LDR     t0, sp, 32
            csrw    sstatus, t0

            STR     tp, {kernel_tp_addr}, 0
            LDR     gp, sp, 2
            LDR     tp, sp, 3

            POP_GENERAL_REGS
            LDR     sp, sp, 1

            sret",
            tf = in(reg) self,
            kernel_tp_addr = in(reg) kernel_tp_addr,
            options(noreturn),
        )
    }
}

/*
    TaskContext 目前是进程的 Context，
    还需要分离实现一个 GuestContext
 */

#[repr(C)]
#[derive(Debug, Default)]
pub struct TaskContext {
    pub ra: usize, // return address (x1)
    pub sp: usize, // stack pointer (x2)

    pub s0: usize, // x8-x9
    pub s1: usize,

    pub s2: usize, // x18-x27
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,

    pub satp: usize,
}

impl TaskContext {
    pub const fn default() -> Self {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
    }

    pub fn init(
        &mut self,
        entry: usize,
        kstack_top: VirtAddr,
        page_table_root: PhysAddr,
        _is_kernel: bool,
    ) {
        self.sp = kstack_top.as_usize();
        self.ra = entry;
        self.satp = page_table_root.as_usize();
    }

    pub fn switch_to(&mut self, next_ctx: &Self) {
        unsafe {
            instructions::set_user_page_table_root(next_ctx.satp);
            context_switch(self, next_ctx)
        }
    }
}

#[naked]
unsafe extern "C" fn context_switch(_current_task: &mut TaskContext, _next_task: &TaskContext) {
    asm!(
        "
        // save old context (callee-saved registers)
        STR     ra, a0, 0
        STR     sp, a0, 1
        STR     s0, a0, 2
        STR     s1, a0, 3
        STR     s2, a0, 4
        STR     s3, a0, 5
        STR     s4, a0, 6
        STR     s5, a0, 7
        STR     s6, a0, 8
        STR     s7, a0, 9
        STR     s8, a0, 10
        STR     s9, a0, 11
        STR     s10, a0, 12
        STR     s11, a0, 13

        // restore new context
        LDR     s11, a1, 13
        LDR     s10, a1, 12
        LDR     s9, a1, 11
        LDR     s8, a1, 10
        LDR     s7, a1, 9
        LDR     s6, a1, 8
        LDR     s5, a1, 7
        LDR     s4, a1, 6
        LDR     s3, a1, 5
        LDR     s2, a1, 4
        LDR     s1, a1, 3
        LDR     s0, a1, 2
        LDR     sp, a1, 1
        LDR     ra, a1, 0

        ret",
        options(noreturn),
    )
}
