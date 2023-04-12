use riscv::register::scause::{self, Exception as E, Trap};
use riscv::register::{mtvec::TrapMode, stval, stvec};

use super::TrapFrame;
use crate::{syscall::syscall, task};

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
fn riscv_trap_handler(tf: &mut TrapFrame, from_user: bool) {
    let scause = scause::read();
    trace!("trap {:?} @ {:#x}: {:#x?}", scause.cause(), tf.sepc, tf);
    match scause.cause() {
        Trap::Exception(E::UserEnvCall) => {
            // 这里应该先判断 Ecall 前的 Virt 状态变量
            // 如果这个 Ecall 是来自于 User Mode，则响应 syscall, 
            // 如果这个 Ecall 是来自于 Virtual User Mode，则响应 panic!
            tf.sepc += 4;
            tf.regs.a0 = syscall(tf, tf.regs.a7, tf.regs.a0, tf.regs.a1, tf.regs.a2) as _;
        }
        Trap::Exception(E::LoadPageFault)
        | Trap::Exception(E::StorePageFault)
        | Trap::Exception(E::InstructionPageFault) => {
            // 这里应该先判断 PageFault 前的 Virt 状态变量
            if from_user {
                warn!(
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
        }
        Trap::Exception(E::LoadGuestPageFault) 
        | Trap::Exception(E::StoreGuestPageFault) 
        | Trap::Exception(E::InstructionGuestPageFault) => {
            // 来自于VS-mode的PageFault异常            
            panic!("Unsupported ENV CALL");     
        }
        Trap::Exception(E::VirtualSupervisorEnvCall) => {
            // 来自于VS-mode的异常，应当由HS-mode处理。
            panic!("Unsupported ENV CALL");
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
