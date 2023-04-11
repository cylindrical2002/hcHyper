cfg_if! {
    if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
        mod riscv;
        pub use self::riscv::*;
    } 
    // TODO: 添加回 x86_64 aarch64 支持
    // else if #[cfg(target_arch = "x86_64")] {
    //     mod x86_64;
    //     pub use self::x86_64::*;
    // } else if #[cfg(target_arch = "aarch64")] {
    //     mod aarch64;
    //     pub use self::aarch64::*;
    // } 
}
