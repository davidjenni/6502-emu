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
    pub extra_bytes: u8,
    pub cycles: u8,
}

#[rustfmt::skip]
pub fn decode(opcode: u8) -> Result<DecodedInstruction, CpuError> {
    // lookup table is generated via ../build.rs from a CSV file:
    // generated file somewhere at: target/debug/build/mos6502-emulator-<generatedId>/out/opcodes_mos6502.r
    // see also compile output for actual path
    include!(concat!(env!("OUT_DIR"), "/opcodes_mos6502.rs"))
}
