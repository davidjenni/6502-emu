use crate::cpu::{AddressingMode, Cpu};
use crate::CpuError;

// Branch/jump operations:
pub fn execute_beq(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    if cpu.status.zero() {
        cpu.address_bus.set_pc(effective_address)?;
    }
    Ok(())
}

pub fn execute_bmi(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    if cpu.status.negative() {
        cpu.address_bus.set_pc(effective_address)?;
    }
    Ok(())
}

pub fn execute_jmp(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    cpu.address_bus.set_pc(effective_address)?;
    Ok(())
}
