use crate::address_bus::SystemVector;
use crate::cpu_impl::{AddressingMode, CpuImpl};
use crate::CpuError;

// BRK:    Force break
// status: NV ...ZC
pub fn execute_brk(mode: AddressingMode, cpu: &mut CpuImpl) -> Result<(), CpuError> {
    if mode != AddressingMode::Implied {
        return Err(CpuError::InvalidAddressingMode);
    }
    // see 6502 prog manual section 9.11 pg 144:
    // the pushed PC skips the byte after the BRK instruction
    let pc = cpu.address_bus.get_pc().wrapping_add(1);
    cpu.stack.push_word(cpu.memory.as_mut(), pc)?;
    let status = cpu.status.get_status();
    // set Break flag on pushed status value only
    cpu.stack
        .push_byte(cpu.memory.as_mut(), status | 0b0001_0000)?;

    // continue via IRQ vector: 0xFFFE
    cpu.address_bus.set_pc(SystemVector::IRQ as u16)?;
    Ok(())
}

// RTI:    Return from interrupt
// pull PC, add 1, put result in PC
// status: NV bD.ZC
pub fn execute_rti(mode: AddressingMode, cpu: &mut CpuImpl) -> Result<(), CpuError> {
    if mode != AddressingMode::Implied {
        return Err(CpuError::InvalidAddressingMode);
    }
    let status = cpu.stack.pop_byte(cpu.memory.as_mut())?;
    let pc = cpu.stack.pop_word(cpu.memory.as_mut())?;
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
        let mut cpu = CpuImpl::default();
        cpu.status.set_carry(true);
        cpu.status.set_negative(true);
        cpu.address_bus.set_pc(0x0123)?;

        execute_brk(AddressingMode::Implied, &mut cpu)?;
        assert_eq!(cpu.stack.pop_byte(cpu.memory.as_mut())?, 0b1001_0001);
        assert_eq!(cpu.stack.pop_word(cpu.memory.as_mut())?, 0x0124);
        Ok(())
    }

    #[test]
    fn rti() -> Result<(), CpuError> {
        let mut cpu = CpuImpl::default();
        cpu.stack.push_word(cpu.memory.as_mut(), 0x1234)?;
        cpu.stack.push_byte(cpu.memory.as_mut(), 0b1101_1011)?;

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
