use std::time;
use thiserror::Error;

use crate::cpu::Cpu;
pub use crate::debugger::Debugger;

mod address_bus;
mod cpu;
mod debugger;
mod disassembler;
mod engine;
mod memory;
mod stack_pointer;
mod status_register;

#[derive(Debug, PartialEq, Error)]
pub enum CpuError {
    #[error("CPU is not initialized")]
    NotInitialized,
    #[error("PC address is out of bounds")]
    InvalidAddress,
    #[error("addressing mode is not supported")]
    InvalidAddressingMode,
    #[error("illegal op code instruction {0}")]
    InvalidOpcode(u8), // TODO: also capture PC
    #[error("op code instruction expects an operand, but none was found")]
    MissingOperand,
    #[error("op code instruction expects an operand, but none was found")]
    StackOverflow,
}

#[derive(Debug, Clone)]
pub struct CpuRegisterSnapshot {
    pub accumulator: u8,
    pub x_register: u8,
    pub y_register: u8,
    pub stack_pointer: u16,
    pub program_counter: u16,
    pub status: u8,
    // stats counters:
    pub elapsed_time: time::Duration,
    pub accumulated_cycles: u64,
    pub accumulated_instructions: u64,
    pub approximate_clock_speed: f64,
}

pub trait CpuController {
    fn reset(&mut self) -> Result<(), CpuError>;
    fn load_program(&mut self, start_addr: u16, program: &[u8]) -> Result<(), CpuError>;
    // TODO: run/step return a Result with a CpuError AND a CpuRegisterSnapshot to convey where the error occurred
    fn run(&mut self, start_addr: Option<u16>) -> Result<CpuRegisterSnapshot, CpuError>;
    fn get_byte_at(&self, address: u16) -> Result<u8, CpuError>;
    fn set_byte_at(&mut self, address: u16, value: u8) -> Result<(), CpuError>;
    fn as_debugger<'a>(&'a mut self) -> Box<dyn Debugger<'a> + 'a>;
}

pub enum CpuType {
    MOS6502,
}

pub fn create(kind: CpuType) -> Result<Box<dyn CpuController>, CpuError> {
    let mut cpu = match kind {
        CpuType::MOS6502 => Cpu::default(),
    };
    cpu.reset()?;
    Ok(Box::new(cpu))
}
