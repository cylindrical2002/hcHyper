use riscv::register::time;

use super::super::interrupt::{self, ScauseIntCode};
use super::super::misc::sbi;

const NANOS_PER_TICK: u64 = crate::timer::NANOS_PER_SEC / crate::config::TIMER_FREQUENCY as u64;

pub fn current_ticks() -> u64 {
    time::read() as u64
}

// 纳秒和tick的相互转换

pub fn nanos_to_ticks(nanos: u64) -> u64 {
    nanos / NANOS_PER_TICK
}

pub fn ticks_to_nanos(ticks: u64) -> u64 {
    ticks * NANOS_PER_TICK
}

pub fn set_oneshot_timer(deadline_ns: u64) {
    sbi::set_timer(nanos_to_ticks(deadline_ns));
}

pub fn init() {
    // 开一个时钟中断的 handler
    // 这个 handler 是一个 timer.rs 中的函数，支持多种指令集
    interrupt::register_handler(ScauseIntCode::Timer as _, crate::timer::handle_timer_irq);
    // 在 riscv_intc 中 Enable 时钟中断
    interrupt::set_enable(ScauseIntCode::Timer as _, true);
}
