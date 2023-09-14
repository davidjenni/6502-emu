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

        // LDA/X/Y:
        0xA4 => Ok(DecodedInstruction {
            opcode: OpCode::LDY,
            mode: AddressingMode::ZeroPage,
            execute: execute_ldy,
        }),
        0xA5 => Ok(DecodedInstruction {
            opcode: OpCode::LDA,
            mode: AddressingMode::ZeroPage,
            execute: execute_lda,
        }),
        0xA6 => Ok(DecodedInstruction {
            opcode: OpCode::LDX,
            mode: AddressingMode::ZeroPage,
            execute: execute_ldx,
        }),
        0xA9 => Ok(DecodedInstruction {
            opcode: OpCode::LDA,
            mode: AddressingMode::Immediate,
            execute: execute_lda,
        }),

        // STA/X/Y:
        0x84 => Ok(DecodedInstruction {
            opcode: OpCode::STY,
            mode: AddressingMode::ZeroPage,
            execute: execute_sty,
        }),
        0x85 => Ok(DecodedInstruction {
            opcode: OpCode::STA,
            mode: AddressingMode::ZeroPage,
            execute: execute_sta,
        }),
        0x86 => Ok(DecodedInstruction {
            opcode: OpCode::STX,
            mode: AddressingMode::ZeroPage,
            execute: execute_stx,
        }),

        // ALU operations:
        0xE5 => Ok(DecodedInstruction {
            opcode: OpCode::SBC,
            mode: AddressingMode::ZeroPage,
            execute: execute_sbc,
        }),

        // Branch/jump operations:
        0x30 => Ok(DecodedInstruction {
            opcode: OpCode::BMI,
            mode: AddressingMode::Relative,
            execute: execute_bmi,
        }),
        0x4C => Ok(DecodedInstruction {
            opcode: OpCode::JMP,
            mode: AddressingMode::Absolute,
            execute: execute_jmp,
        }),
        0xF0 => Ok(DecodedInstruction {
            opcode: OpCode::BEQ,
            mode: AddressingMode::Relative,
            execute: execute_beq,
        }),

        // Set/clear status flags:
        0x38 => Ok(DecodedInstruction {
            opcode: OpCode::SEC,
            mode: AddressingMode::Implied,
            execute: execute_sec,
        }),
        // TODO: BRK instead??
        _ => Err(CpuError::InvalidOpcode),
    }
}
