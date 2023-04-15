use core::arch::asm;

use crate::arch::instructions;
use crate::mm::{PhysAddr, VirtAddr};

include_asm_marcos!();

/*
    这里的 Context 对应着 rCore-Tutorial 中的 TrapContext
 */

pub trait Context{
    // fn default() -> Self;
    fn init(&mut self, entry: usize, kstack_top: VirtAddr, page_table_root: PhysAddr, _is_kernel: bool);
    fn is_process(&self) -> bool;
    fn switch_to_process(&mut self, next_ctx: &ProcessContext);
    fn switch_to_guest(&mut self, next_ctx: &GuestContext);
}

/*
    ProcessContext 是进程的 Context，
    还需要分离实现一个 GuestContext
 */

#[repr(C)]
#[derive(Debug)]
pub struct ProcessContext {
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

impl Default for ProcessContext{
    fn default() -> Self {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
    }
}

impl Context for ProcessContext{
    fn init(
        &mut self,
        entry: usize,
        kstack_top: VirtAddr,
        page_table_root: PhysAddr,
        _is_kernel: bool
    ) {
        self.sp = kstack_top.as_usize();
        self.ra = entry;
        self.satp = page_table_root.as_usize();
    }

    fn is_process(&self) -> bool {
        true
    }

    fn switch_to_process(&mut self, next_ctx: &ProcessContext) {
        unsafe {
            instructions::set_user_page_table_root(next_ctx.satp);
            context_switch(self, next_ctx);
        }
    }

    fn switch_to_guest(&mut self, next_ctx: &GuestContext) {
        panic!("unsupported");
    }
}

pub struct GuestContext {
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

#[naked]
unsafe extern "C" fn context_switch(_current_task: &mut ProcessContext, _next_task: &ProcessContext) {
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
