use core::arch::asm;

use riscv::register::{sepc, sscratch};

use crate::arch::instructions;
use crate::mm::VirtAddr;

include_asm_marcos!();

/*
    这里的 Frame 对应着 rCore-Tutorial 中 TaskContext
 */

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
pub struct ProcessTrapFrame {
    pub regs: GeneralRegisters,
    pub sepc: usize,
    pub sstatus: usize,
}

impl ProcessTrapFrame {
    pub fn new_user(entry: VirtAddr, ustack_top: VirtAddr, arg0: usize) -> Self {
        const SPIE: usize = 1 << 5;
        const SUM: usize = 1 << 18;
        // TrapFrame 初始化后执行sret 
        Self {
            regs: GeneralRegisters {
                a0: arg0,
                sp: ustack_top.as_usize(),
                ..Default::default()
            },
            sepc: entry.as_usize(),
            sstatus: SPIE | SUM,
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
        let kernel_tp_addr = kstack_top.as_usize() - core::mem::size_of::<ProcessTrapFrame>()
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
