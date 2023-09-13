use crate::cpu::Cpu;
use crate::memory::RamMemory;

pub mod address_bus;
mod cpu;
mod memory;
mod status_register;

#[derive(Debug)]
pub enum CpuError {
    NotInitialized,
    InvalidAddress,
    InvalidAddressingMode,
}

pub trait CpuController {
    fn reset(&mut self) -> Result<(), CpuError>;
    fn load_program(&mut self, start_addr: u16, program: &[u8]) -> Result<(), CpuError>;
    fn step(&mut self) -> Result<(), CpuError>;
    fn run(&mut self) -> Result<(), CpuError>;
}

pub enum CpuType {
    MOS6502,
}
pub fn create(kind: CpuType) -> Result<Box<dyn CpuController>, CpuError> {
    match kind {
        CpuType::MOS6502 => Ok(Box::new(Cpu::new(Box::<RamMemory>::default()))),
    }
}
