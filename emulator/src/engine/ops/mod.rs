pub mod alu;
pub mod branch_jump;
pub mod flag_branch;
pub mod interrupt;
pub mod transfer;

use crate::cpu::{AddressingMode, Cpu};
use crate::CpuError;

// special codes:
pub fn execute_nop(_: AddressingMode, _: &mut Cpu) -> Result<(), CpuError> {
    Ok(())
}
