use crate::address_bus::AddressBus;
use crate::cpu::{AddressingMode, Cpu};
use crate::stack_pointer::StackPointer;
// use crate::status_register::StatusRegister;
use crate::CpuError;

// BRK:    Force break
// status: NV ...ZC
pub fn execute_brk(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    if mode != AddressingMode::Implied {
        return Err(CpuError::InvalidAddressingMode);
    }
    // see 6502 prog manual section 9.11 pg 144:
    // the pushed PC skips the byte after the BRK instruction
    let pc = cpu.address_bus.get_pc().wrapping_add(1);
    cpu.stack.push_word(&mut cpu.memory, pc)?;
    let status = cpu.status.get_status();
    // set Break flag on pushed status value only
    cpu.stack.push_byte(&mut cpu.memory, status | 0b0001_0000)?;

    // HACK: set Break flag on CPU status register as well for Cpu.run() to stop on BRK
    cpu.status.set_break_command(true);

    // TODO: continue on IRQ vector: 0xFFFE
    // cpu.address_bus.set_pc(0xFFFE)?;
    Ok(())
}

// RTI:    Return from interrupt
// pull PC, add 1, put result in PC
// status: NV bD.ZC
#[allow(dead_code)] // TODO remove
pub fn execute_rti(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    if mode != AddressingMode::Implied {
        return Err(CpuError::InvalidAddressingMode);
    }
    let status = cpu.stack.pop_byte(&cpu.memory)?;
    let pc = cpu.stack.pop_word(&cpu.memory)?;
    cpu.status.set_status(status & 0xCF); // ignore Break and undefined flags
    cpu.address_bus.set_pc(pc)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brk() -> Result<(), CpuError> {
        // assert_eq!(cpu.address_bus.get_pc(), 0xFFFE);
        let mut cpu = Cpu::default();
        cpu.status.set_carry(true);
        cpu.status.set_negative(true);
        cpu.address_bus.set_pc(0x0123)?;

        execute_brk(AddressingMode::Implied, &mut cpu)?;
        assert_eq!(cpu.stack.pop_byte(&cpu.memory)?, 0b1001_0001);
        assert_eq!(cpu.stack.pop_word(&cpu.memory)?, 0x0124);
        Ok(())
    }

    #[test]
    fn rti() -> Result<(), CpuError> {
        let mut cpu = Cpu::default();
        cpu.stack.push_word(&mut cpu.memory, 0x1234)?;
        cpu.stack.push_byte(&mut cpu.memory, 0b1101_1011)?;

        execute_rti(AddressingMode::Implied, &mut cpu)?;

        assert_eq!(cpu.address_bus.get_pc(), 0x1234);
        assert!(cpu.status.negative());
        assert!(cpu.status.overflow());
        assert!(!cpu.status.break_command()); // bit4, break, is cleared by RTI
        assert!(cpu.status.decimal_mode());
        assert!(cpu.status.zero());
        assert!(cpu.status.carry());
        Ok(())
    }
}
