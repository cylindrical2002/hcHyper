cfg_if! {
    if #[cfg(target_arch = "riscv64")] {
        mod riscv;
        use self::riscv as imp;
    }
    // TODO: 加回 x86_64 aarch64 支持
    // else if #[cfg(target_arch = "x86_64")] {
    //     mod uart16550;
    //     use uart16550 as imp;
    // } else if #[cfg(target_arch = "aarch64")] {
    //     mod pl011;
    //     use pl011 as imp;
    // } 
}

pub use self::imp::{console_getchar, console_putchar};
pub(super) use self::imp::{init, init_early};
