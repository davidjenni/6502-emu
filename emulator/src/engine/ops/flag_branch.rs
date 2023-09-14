use crate::cpu::{AddressingMode, Cpu};
use crate::CpuError;

// Set/clear status flags:
pub fn execute_sec(_: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    cpu.status.set_carry(true);
    Ok(())
}
