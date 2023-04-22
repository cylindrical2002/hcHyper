use crate::{arch::{TrapFrame, instructions}, drivers::misc::sbi};

const HYPERCALL_PUTCHAR: usize = 1;
const HYPERCALL_SHUTDOWN: usize = 8;

// TODO: 完善所有的 hypercall
pub fn hypercall(
    _tf: &mut TrapFrame,
    hypercall_id: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
) -> isize {
    // 处理 hypercall 的过程中，也要允许中断
    instructions::enable_irqs();
    debug!(
        "hypercall {} enter <= ({:#x}, {:#x}, {:#x})",
        hypercall_id, arg0, arg1, arg2
    );
    let ret = match hypercall_id {
        HYPERCALL_PUTCHAR => {
            sbi::console_putchar(arg0);
            0
        }
        HYPERCALL_SHUTDOWN => {
            println!("Guest OS Shutdown");
            crate::task::current().exit(0);
        }
        _ => {
            warn!("Unsupported hypercall_id: {}", hypercall_id);
            crate::task::current().exit(-1);
        }
    };
    debug!("hypercall {} ret => {:#x}", hypercall_id, ret);
    instructions::disable_irqs();
    ret
}