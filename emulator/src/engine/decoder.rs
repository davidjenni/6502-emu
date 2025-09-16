use crate::CpuError;
use crate::cpu_impl::{AddressingMode, CpuImpl};
use crate::engine::opcodes::OpCode;
use crate::engine::ops::alu::*;
use crate::engine::ops::branch_jump::*;
use crate::engine::ops::execute_nop;
use crate::engine::ops::flag_compare::*;
use crate::engine::ops::interrupt::*;
use crate::engine::ops::stack::*;
use crate::engine::ops::transfer::*;

type OpCodeExecute = fn(AddressingMode, &mut CpuImpl) -> Result<(), CpuError>;

#[derive(Debug, Clone)]
pub struct DecodedInstruction {
    pub opcode: OpCode,
    pub mode: AddressingMode,
    pub execute: OpCodeExecute,
    pub extra_bytes: u8,
    pub cycles: u8,
    pub hex_opcode: u8,
}

impl DecodedInstruction {
    pub fn get_mnemonic(&self) -> String {
        // depends on OpCode::Display impl
        self.opcode.to_string()
    }
}

#[rustfmt::skip]
pub fn decode(opcode_byte: u8) -> Result<DecodedInstruction, CpuError> {
    match decode_via_csv(opcode_byte) {
        Ok(decoded) => Ok(decoded),
        Err(_) => Ok(DecodedInstruction {
            opcode: OpCode::ILL(opcode_byte),
            mode: AddressingMode::Implied,
            execute: execute_brk,
            extra_bytes: 0,
            cycles: 0,
            hex_opcode: opcode_byte,
        }),
    }
}

#[rustfmt::skip]
fn decode_via_csv(opcode_byte: u8) -> Result<DecodedInstruction, CpuError> {
    // lookup table is generated via ../build.rs from a CSV file:
    // generated file somewhere at: target/debug/build/mos6502-emulator-<generatedId>/out/opcodes_mos6502.r
    // see also compile output for actual path
    include!(concat!(env!("OUT_DIR"), "/opcodes_mos6502.rs"))
 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_legal_opcode() -> Result<(), CpuError> {
        // BRK
        let decoded = decode(0x00)?;
        assert_eq!(decoded.opcode, OpCode::BRK);
        assert_eq!(decoded.mode, AddressingMode::Implied);
        assert_eq!(decoded.extra_bytes, 0);
        assert_eq!(decoded.cycles, 7);

        // STY
        let decoded = decode(0x8c)?;
        assert_eq!(decoded.opcode, OpCode::STY);
        assert_eq!(decoded.mode, AddressingMode::Absolute);
        assert_eq!(decoded.extra_bytes, 2);
        assert_eq!(decoded.cycles, 4);
        Ok(())
    }

    #[test]
    fn decode_illegal_opcode() -> Result<(), CpuError> {
        let decoded = decode(0xff)?;
        assert_eq!(decoded.opcode, OpCode::ILL(0xff));
        assert_eq!(decoded.mode, AddressingMode::Implied);
        assert_eq!(decoded.extra_bytes, 0);
        assert_eq!(decoded.cycles, 0);
        Ok(())
    }

    #[test]
    fn get_mnemonic() -> Result<(), CpuError> {
        let decoded = decode(0x00)?;
        assert_eq!(decoded.get_mnemonic(), "BRK");

        let decoded = decode(0xFA)?;
        assert_eq!(decoded.get_mnemonic(), "ILL(FA)");
        Ok(())
    }
}
