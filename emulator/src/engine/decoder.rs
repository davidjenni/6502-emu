use crate::cpu_impl::{AddressingMode, CpuImpl};
use crate::engine::opcodes::OpCode;
use crate::engine::ops::alu::*;
use crate::engine::ops::branch_jump::*;
use crate::engine::ops::execute_nop;
use crate::engine::ops::flag_compare::*;
use crate::engine::ops::interrupt::*;
use crate::engine::ops::stack::*;
use crate::engine::ops::transfer::*;
use crate::CpuError;

type OpCodeExecute = fn(AddressingMode, &mut CpuImpl) -> Result<(), CpuError>;

#[derive(Debug)]
pub struct DecodedInstruction {
    pub opcode: OpCode,
    pub mode: AddressingMode,
    pub execute: OpCodeExecute,
    pub extra_bytes: u8,
    pub cycles: u8,
}

impl DecodedInstruction {
    pub fn get_mnemonic(&self) -> String {
        // depends on OpCode::Display impl
        self.opcode.to_string()
    }
}

#[rustfmt::skip]
pub fn decode(opcode: u8) -> Result<DecodedInstruction, CpuError> {
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
    fn decode_illegal_opcode() {
        let decoded = decode(0xff);
        assert!(decoded.is_err());
    }

    #[test]
    fn get_mnemonic() {
        let decoded = decode(0x00).unwrap();
        assert_eq!(decoded.get_mnemonic(), "BRK");
    }
}
