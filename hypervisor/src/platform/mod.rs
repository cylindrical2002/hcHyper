cfg_if! {
    if #[cfg(feature = "platform-qemu-virt-riscv")] {
        mod qemu_virt_riscv;
        pub use self::qemu_virt_riscv::*;
    }
    // TODO: 加回 x86_64 aarch64 支持
    // else if #[cfg(any(feature = "platform-pc", feature = "platform-pc-rvm", feature = "platform-rvm-guest-x86_64"))] {
    //     mod pc;
    //     pub use self::pc::*;
    // } else if #[cfg(feature = "platform-qemu-virt-arm")] {
    //     mod qemu_virt_arm;
    //     pub use self::qemu_virt_arm::*;
    // }
}

pub mod config;
