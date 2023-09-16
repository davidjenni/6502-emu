use crate::cpu::Cpu;
use crate::memory::RamMemory;

pub mod address_bus;
mod address_bus2;
mod cpu;
mod engine;
mod memory;
mod memory_access;
mod stack_pointer;
mod status_register;

#[derive(Debug, PartialEq)]
pub enum CpuError {
    NotInitialized,
    InvalidAddress,
    InvalidAddressingMode,
    InvalidOpcode,
    MissingOperand,
    StackOverflow,
}

pub struct CpuRegisterSnapshot {
    pub accumulator: u8,
    pub x_register: u8,
    pub y_register: u8,
    pub stack_pointer: u16,
    pub program_counter: u16,
    pub status: u8,
}

pub trait CpuController {
    fn reset(&mut self) -> Result<(), CpuError>;
    fn load_program(&mut self, start_addr: u16, program: &[u8]) -> Result<(), CpuError>;
    fn step(&mut self) -> Result<(), CpuError>;
    fn run(&mut self, start_addr: Option<u16>) -> Result<CpuRegisterSnapshot, CpuError>;
    fn get_register_snapshot(&self) -> CpuRegisterSnapshot;
    fn get_byte_at(&self, address: u16) -> Result<u8, CpuError>;
    fn set_byte_at(&mut self, address: u16, value: u8) -> Result<(), CpuError>;
}

pub enum CpuType {
    MOS6502,
}
pub fn create(kind: CpuType) -> Result<Box<dyn CpuController>, CpuError> {
    match kind {
        CpuType::MOS6502 => Ok(Box::new(Cpu::new(Box::<RamMemory>::default()))),
    }
}
