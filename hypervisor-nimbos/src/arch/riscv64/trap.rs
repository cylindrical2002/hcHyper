use riscv::register::sstatus::{self, SPP};
use riscv::register::scause::{self, Exception as E, Trap};
use riscv::register::{mtvec::TrapMode, stval, stvec, hstatus};

use super::TrapFrame;
use crate::{hypercall::hypercall, syscall::syscall, task};

include_asm_marcos!();

core::arch::global_asm!(
    include_str!("trap.S"),
    trapframe_size = const core::mem::size_of::<TrapFrame>(),
);

pub fn init() {
    extern "C" {
        fn trap_vector_base();
    }
    unsafe { stvec::write(trap_vector_base as usize, TrapMode::Direct) };
}

#[no_mangle]
fn riscv_trap_handler(tf: &mut TrapFrame) {
    let scause = scause::read();
    trace!("trap {:?} @ {:#x}: {:#x?}", scause.cause(), tf.sepc, tf);
    match scause.cause() {
        Trap::Exception(E::UserEnvCall) => {
            // 这里应该先判断 Ecall 前的 Virt 状态变量
            let hstatus = hstatus::read();
            if !hstatus.spv() {
                // 这个是 U-Mode 下的 ECall，应该转到 syscall 处理
                tf.sepc += 4;
                tf.regs.a0 = syscall(tf, tf.regs.a7, tf.regs.a0, tf.regs.a1, tf.regs.a2) as _;
            } else {
                panic!("Reject User Env Call in HS-Mode, It needs to be handled in VS-Mode")
            }
        }
        Trap::Exception(E::LoadPageFault)
        | Trap::Exception(E::StorePageFault)
        | Trap::Exception(E::InstructionPageFault) => {
            // 这里应该先判断 PageFault 前的 Virt 状态变量
            let hstatus = hstatus::read();
            if !hstatus.spv() {
                let sstatus = sstatus::read();
                if sstatus.spp() == SPP::User {
                    println!(
                        "Page Fault @ {:#x}, stval={:#x}, scause={}, kernel killed it.",
                        tf.sepc,
                        stval::read(),
                        scause.code(),
                    );
                    task::current().exit(-1);
                } else {
                    panic!(
                        "Kernel Page Fault @ {:#x}, stval={:#x}, scause={}",
                        tf.sepc,
                        stval::read(),
                        scause.code(),
                    );
                }
            } else {
                panic!("Reject Page Fault in HS-Mode, It needs to be handled in VS-Mode")
            }
        }
        Trap::Exception(E::LoadGuestPageFault)
        | Trap::Exception(E::StoreGuestPageFault)
        | Trap::Exception(E::InstructionGuestPageFault) => {
            // 来自于VS-mode的PageFault异常
            println!(
                "Guest Page Fault @ {:#x}, stval={:#x}, scause={}, kernel killed it.",
                tf.sepc,
                stval::read(),
                scause.code(),
            );
            task::current().exit(-1); // 退出目前的任务
        }
        Trap::Exception(E::VirtualSupervisorEnvCall) => {
            // 来自于VS-mode的异常，应当由HS-mode处理。
            tf.sepc += 4;
            tf.regs.a0 = hypercall(tf, tf.regs.a7, tf.regs.a0, tf.regs.a1, tf.regs.a2) as _;
        }
        Trap::Exception(E::VirtualInstruction) => {
            // TODO: 处理 VS-mode 希望执行的特殊指令
        }
        Trap::Interrupt(_) => task::handle_irq(scause.bits()),
        _ => {
            panic!(
                "Unsupported trap {:?} @ {:#x}:\n{:#x?}",
                scause.cause(),
                tf.sepc,
                tf
            );
        }
    }
}
