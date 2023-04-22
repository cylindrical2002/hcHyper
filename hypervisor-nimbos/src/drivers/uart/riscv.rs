use super::super::misc::sbi;

pub fn init_early() {}
pub fn init() {}

/*
    riscv 下的串口完全由 openSBI 控制
    HyperVisor 层只调用 SBI 接口即可
 */

pub fn console_putchar(c: u8) {
    sbi::console_putchar(c as usize)
}

pub fn console_getchar() -> Option<u8> {
    match sbi::console_getchar() {
        -1 => None,
        c => Some(c as u8),
    }
}
