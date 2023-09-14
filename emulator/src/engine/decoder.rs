use crate::cpu::{AddressingMode, Cpu};
use crate::engine::handlers::*;
use crate::engine::opcodes::OpCode;
use crate::CpuError;

type OpCodeExecute = fn(AddressingMode, &mut Cpu) -> Result<(), CpuError>;

#[derive(Debug)]
pub struct DecodedInstruction {
    pub opcode: OpCode,
    pub mode: AddressingMode,
    pub execute: OpCodeExecute,
}

pub fn decode(opcode: u8) -> Result<DecodedInstruction, CpuError> {
    match opcode {
        // special codes:
        0xEA => Ok(DecodedInstruction {
            opcode: OpCode::NOP,
            mode: AddressingMode::Implied,
            execute: execute_nop,
        }),
        0x00 => Ok(DecodedInstruction {
            opcode: OpCode::BRK,
            mode: AddressingMode::Implied,
            execute: execute_brk,
        }),

        // LDA:
        0xA9 => Ok(DecodedInstruction {
            opcode: OpCode::LDA,
            mode: AddressingMode::Immediate,
            execute: execute_lda,
        }),
        0xA5 => Ok(DecodedInstruction {
            opcode: OpCode::LDA,
            mode: AddressingMode::ZeroPage,
            execute: execute_lda,
        }),

        // STA:
        0x85 => Ok(DecodedInstruction {
            opcode: OpCode::STA,
            mode: AddressingMode::ZeroPage,
            execute: execute_sta,
        }),
        // TODO: BRK instead??
        _ => Err(CpuError::InvalidOpcode),
    }
}
