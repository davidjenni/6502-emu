pub mod alu;
pub mod branch_jump;
pub mod flag_branch;
pub mod interrupt;
pub mod transfer;

// good overview and reference to 6502 instruction opcodes:
// https://www.masswerk.at/6502/6502_instruction_set.html

use crate::cpu::{AddressingMode, Cpu};
use crate::CpuError;

// special codes:
pub fn execute_nop(_: AddressingMode, _: &mut Cpu) -> Result<(), CpuError> {
    Ok(())
}
