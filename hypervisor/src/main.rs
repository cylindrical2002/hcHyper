#![no_std]
#![no_main]
#![feature(asm_const, naked_functions)]
#![feature(panic_info_message, alloc_error_handler)]
#![feature(const_refs_to_cell)]
#![feature(const_maybe_uninit_zeroed)]
#![feature(get_mut_unchecked)]

extern crate alloc;
#[macro_use]
extern crate cfg_if;
#[macro_use]
extern crate log;

#[macro_use]
mod logging;
mod arch;
mod config;
mod drivers;
mod hypercall;
mod lang_items;
mod loader;
mod mm;
mod percpu;
mod platform;
mod sync;
mod syscall;
mod task;
mod timer;
mod utils;

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

const LOGO: &str = r"
 _          _   _                       
| |__   ___| | | |_   _ _ __   ___ _ __ 
| '_ \ / __| |_| | | | | '_ \ / _ \ '__|
| | | | (__|  _  | |_| | |_) |  __/ |   
|_| |_|\___|_| |_|\__, | .__/ \___|_|   
                  |___/|_|              
";

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    drivers::init_early();

    print!("{}\n", LOGO);

    println!("Start HyperVisor");
    println!("arch = {}", option_env!("ARCH").unwrap_or(""));
    println!("platform = {}", option_env!("PLATFORM").unwrap_or(""));
    println!("build_mode = {}", option_env!("MODE").unwrap_or(""));
    println!("log_level = {}", option_env!("LOG").unwrap_or(""));

    // 检查是否支持 hypervisor extension
    // TODO: 改用设备树检查
    if !arch::detect::detect_h_extension() {
        println!("no RISC-V hypervisor extension on current environment");
    } else {
        println!("RISC-V H ISA Available");
    }

    mm::init_heap_early();
    logging::init();
    info!("Logging is enabled.");

    arch::init();
    arch::init_percpu();
    percpu::init_percpu_early();
    mm::init();
    drivers::init();
    percpu::init_percpu();
    timer::init();

    task::init();

    // 输出 APP 还有 GUEST OS 的名字
    print!("\n");
    loader::list_apps();
    println!("");
    loader::list_guests();
    print!("\n");

    task::run();
}
