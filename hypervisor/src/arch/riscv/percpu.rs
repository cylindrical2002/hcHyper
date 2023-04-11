pub struct ArchPerCpu;

// 除了寄存器之外，RISC-V 不需要额外记录 CPU 状态
impl ArchPerCpu {
    pub fn new() -> Self {
        Self
    }

    pub fn init(&mut self, _cpu_id: usize) {}
}
