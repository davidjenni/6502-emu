use crate::cpu::{AddressingMode, Cpu};
use crate::CpuError;

// LDA/X/Y:
pub fn execute_lda(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let value = cpu.get_effective_operand(mode)?;
    cpu.accumulator = value;
    cpu.status.update_from(value);
    Ok(())
}

pub fn execute_ldx(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let value = cpu.get_effective_operand(mode)?;
    cpu.index_x = value;
    cpu.status.update_from(value);
    Ok(())
}

pub fn execute_ldy(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let value = cpu.get_effective_operand(mode)?;
    cpu.index_y = value;
    cpu.status.update_from(value);
    Ok(())
}

// STA/X/Y:
pub fn execute_sta(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    cpu.address_bus.write(effective_address, cpu.accumulator)?;
    Ok(())
}

pub fn execute_stx(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    cpu.address_bus.write(effective_address, cpu.index_x)?;
    Ok(())
}

pub fn execute_sty(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    cpu.address_bus.write(effective_address, cpu.index_y)?;
    Ok(())
}
