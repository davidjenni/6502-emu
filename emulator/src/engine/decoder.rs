use crate::cpu::{AddressingMode, Cpu};
use crate::engine::opcodes::OpCode;
use crate::engine::ops::alu::*;
use crate::engine::ops::branch_jump::*;
use crate::engine::ops::execute_nop;
use crate::engine::ops::flag_compare::*;
use crate::engine::ops::interrupt::*;
use crate::engine::ops::stack::*;
use crate::engine::ops::transfer::*;
use crate::CpuError;

type OpCodeExecute = fn(AddressingMode, &mut Cpu) -> Result<(), CpuError>;

#[derive(Debug)]
pub struct DecodedInstruction {
    pub opcode: OpCode,
    pub mode: AddressingMode,
    pub execute: OpCodeExecute,
}

pub fn decode(opcode: u8) -> Result<DecodedInstruction, CpuError> {
    // lookup table is generated via ../build.rs from a CSV file:
    include!(concat!(env!("OUT_DIR"), "/opcodes_mos6502.rs"))
}
