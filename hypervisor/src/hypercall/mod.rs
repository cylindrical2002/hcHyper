const HYPERCALL_READ: usize = 0;
const HYPERCALL_WRITE: usize = 1;

mod fs;

use self::fs::*;
use crate::arch::{ProcessTrapFrame, instructions};


// TODO: 完善所有的 hypercall
pub fn hypercall(
    _tf: &mut ProcessTrapFrame,
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
        HYPERCALL_READ => hyper_read(arg0, arg1.into(), arg2),
        HYPERCALL_WRITE => hyper_write(arg0, arg1.into(), arg2),
        _ => {
            warn!("Unsupported syscall_id: {}", hypercall_id);
            crate::task::current().exit(-1);
        }
    };
    debug!("hypercall {} ret => {:#x}", hypercall_id, ret);
    instructions::disable_irqs();
    ret
}