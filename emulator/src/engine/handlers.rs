use crate::cpu::{AddressingMode, Cpu};
use crate::CpuError;

pub fn execute_nop(_: AddressingMode, _: &mut Cpu) -> Result<(), CpuError> {
    Ok(())
}

pub fn execute_brk(_: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    cpu.status.set_break(true);
    // TODO: push PC and status to stack, like an IRQ
    Ok(())
}

pub fn execute_lda(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let value = cpu.get_effective_operand(mode)?;
    cpu.accumulator = value;
    cpu.status.update_from(value);
    Ok(())
}

pub fn execute_sta(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    cpu.address_bus.write(effective_address, cpu.accumulator)?;
    Ok(())
}
