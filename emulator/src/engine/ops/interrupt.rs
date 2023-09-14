use crate::cpu::{AddressingMode, Cpu};
use crate::CpuError;

pub fn execute_brk(_: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    cpu.status.set_break(true);
    // TODO: push PC and status to stack, like an IRQ
    Ok(())
}
