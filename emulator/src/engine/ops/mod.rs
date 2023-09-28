pub mod alu;
pub mod branch_jump;
pub mod flag_compare;
pub mod interrupt;
pub mod stack;
pub mod transfer;

// good overview and reference to 6502 instruction opcodes:
// https://www.masswerk.at/6502/6502_instruction_set.html

use crate::cpu_impl::{AddressingMode, CpuImpl};
use crate::CpuError;

// special codes:
pub fn execute_nop(_: AddressingMode, _: &mut CpuImpl) -> Result<(), CpuError> {
    Ok(())
}
