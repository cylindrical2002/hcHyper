#[allow(unused)]
pub mod hedeleg {
    use core::arch::asm;

    pub const INST_ADDR_MISALIGN: usize = 1 << 0;
    pub const INST_ACCESSS_FAULT: usize = 1 << 1;
    pub const ILLEGAL_INST: usize = 1 << 2;
    pub const BREAKPOINT: usize = 1 << 3;
    pub const LOAD_ADDR_MISALIGNED: usize = 1 << 4;
    pub const LOAD_ACCESS_FAULT: usize = 1 << 5;
    pub const STORE_ADDR_MISALIGNED: usize = 1 << 6;
    pub const STORE_ACCESS_FAULT: usize = 1 << 7;
    pub const ENV_CALL_FROM_U_OR_VU: usize = 1 << 8;
    pub const ENV_CALL_FROM_HS: usize = 1 << 9;
    pub const ENV_CALL_FROM_VS: usize = 1 << 10;
    pub const ENV_CALL_FROM_M: usize = 1 << 11;
    pub const INST_PAGE_FAULT: usize = 1 << 12;
    pub const LOAD_PAGE_FAULT: usize = 1 << 13;
    pub const STORE_PAGE_FAULT: usize = 1 << 15;
    pub const INST_GUEST_PAGE_FAULT: usize = 1 << 20;
    pub const LOAD_GUEST_PAGE_FAULT: usize = 1 << 21;
    pub const VIRTUAL_INST: usize = 1 << 22;
    pub const STORE_GUEST_PAGE_FAULT: usize = 1 << 23;

    pub fn write(hedeleg: usize) {
        unsafe {
            asm!(
                "csrw hedeleg, {}",
                in(reg) hedeleg
            )
        }
    }
}

#[allow(unused)]
pub mod hideleg {
    use core::arch::asm;

    pub const VSSIP: usize = 1 << 2;
    pub const VSTIP: usize = 1 << 6;
    pub const VSEIP: usize = 1 << 10;
    pub fn write(hideleg: usize) {
        unsafe {
            asm!(
                "csrw hideleg, {}",
                in(reg) hideleg
            )
        }
    }
}

#[allow(unused)]
pub mod hcounteren {
    use core::arch::asm;

    pub fn write(hcounteren: u32) {
        unsafe {
            asm!(
                "csrw hcounteren, {}",
                in(reg) hcounteren
            )
        }
    }
}
