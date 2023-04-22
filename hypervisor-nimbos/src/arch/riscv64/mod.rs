#[macro_use]
mod macros;

// FIXME: 以后可以抛弃riscv库。
mod context;
mod hregister;
mod page_table;
mod percpu;
mod trap;

pub mod config;
pub mod detect;
pub mod instructions;

use core::arch::asm;

pub use self::context::{TaskContext, TrapFrame};
pub use self::page_table::{PageTable, PageTableEntry};
pub use self::percpu::ArchPerCpu;

use self::hregister::{hedeleg, hideleg, hcounteren};
use riscv::register::{sie, sscratch, hvip};

pub fn init() {}

pub fn init_percpu() {
    unsafe {
        sscratch::write(0);
        sie::clear_sext();
        sie::clear_ssoft();
        sie::clear_stimer();
    }
    trap::init();
}

pub fn init_hypervisor() {
    // hedeleg: delegate some synchronous exceptions
    hedeleg::write(
        hedeleg::INST_ADDR_MISALIGN
            | hedeleg::BREAKPOINT
            | hedeleg::ENV_CALL_FROM_U_OR_VU
            | hedeleg::INST_PAGE_FAULT
            | hedeleg::LOAD_PAGE_FAULT
            | hedeleg::STORE_PAGE_FAULT,
    );

    // hideleg: delegate all interrupts
    hideleg::write(
        hideleg::VSEIP 
            | hideleg::VSSIP 
            | hideleg::VSTIP
    );

    // hvip: clear all interrupts
    unsafe {
        hvip::clear_vseip();
        hvip::clear_vssip();
        hvip::clear_vstip();
    }

    hcounteren::write(0xffff_ffff);

    // enable all interupts
    unsafe {
        sie::set_sext();
        sie::set_ssoft();
        sie::set_stimer();
    }

    unsafe {
        asm!(
            "csrw vsatp, 0"
        );
    }

}
