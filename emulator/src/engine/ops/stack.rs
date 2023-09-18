use crate::cpu::{AddressingMode, Cpu};
use crate::stack_pointer::StackPointer;
use crate::CpuError;

// PHA: Push accumulator
//  A -> SP
// status: n/c
#[allow(dead_code)] // TODO remove
pub fn execute_pha(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    if mode != AddressingMode::Implied {
        return Err(CpuError::InvalidAddressingMode);
    }
    cpu.stack.push_byte(&mut cpu.memory, cpu.accumulator)?;
    Ok(())
}

// PHP: Push status register
//  S -> SP
// status: n/c
#[allow(dead_code)] // TODO remove
pub fn execute_php(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    if mode != AddressingMode::Implied {
        return Err(CpuError::InvalidAddressingMode);
    }
    cpu.stack
        .push_byte(&mut cpu.memory, cpu.status.get_status() & 0xCF)?;
    Ok(())
}

// PLA: Pull accumulator
//  SP -> A
// status: N. ...Z.
#[allow(dead_code)] // TODO remove
pub fn execute_pla(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    if mode != AddressingMode::Implied {
        return Err(CpuError::InvalidAddressingMode);
    }
    cpu.accumulator = cpu.stack.pop_byte(&cpu.memory)?;
    cpu.status.update_from(cpu.accumulator);
    Ok(())
}

// PLP: Pull status register
//  SP -> P
// status: NV .DIZC
#[allow(dead_code)] // TODO remove
pub fn execute_plp(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    if mode != AddressingMode::Implied {
        return Err(CpuError::InvalidAddressingMode);
    }
    let status = cpu.stack.pop_byte(&cpu.memory)?;
    cpu.status.set_status(status & 0xCF); // ignore Break and undefined flags
    Ok(())
}

// TSX:    SP -> X
// status: N. ...Z.
#[allow(dead_code)] // TODO remove
pub fn execute_tsx(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    if mode != AddressingMode::Implied {
        return Err(CpuError::InvalidAddressingMode);
    }
    cpu.index_x = cpu.stack.get_sp().unwrap() as u8; // stack pointer has fixed 0x01 high byte
    cpu.status.update_from(cpu.index_x);
    Ok(())
}

// TXS:    X -> SP
// status: N. ...Z.
#[allow(dead_code)] // TODO remove
pub fn execute_txs(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    if mode != AddressingMode::Implied {
        return Err(CpuError::InvalidAddressingMode);
    }
    cpu.stack.set_sp(0x0100 | cpu.index_x as u16)?; // stack pointer has fixed 0x01 high byte
    cpu.status.update_from(cpu.index_x);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address_bus::AddressBus;
    use crate::memory_access::MemoryAccess;

    const ZERO_PAGE_ADDR: u16 = 0x00E0;
    const NEXT_PC: u16 = 0x0300;

    fn create_cpu() -> Cpu {
        let mut cpu = Cpu::default();
        cpu.memory.write(NEXT_PC, ZERO_PAGE_ADDR as u8).unwrap();
        cpu.address_bus.set_pc(NEXT_PC).unwrap();
        cpu
    }

    #[test]
    fn test_tsx() -> Result<(), CpuError> {
        let mut cpu = create_cpu();
        cpu.stack.set_sp(0x0142)?;
        execute_tsx(AddressingMode::Implied, &mut cpu).unwrap();
        assert_eq!(cpu.index_x, 0x42);
        assert!(!cpu.status.zero());
        assert!(!cpu.status.negative());
        Ok(())
    }

    #[test]
    fn test_txs() -> Result<(), CpuError> {
        let mut cpu = create_cpu();
        cpu.index_x = 0x42;
        execute_txs(AddressingMode::Implied, &mut cpu).unwrap();
        assert_eq!(cpu.stack.get_sp()?, 0x0142); // stack pointer has fixed 0x01 high byte
        assert!(!cpu.status.zero());
        assert!(!cpu.status.negative());
        Ok(())
    }

    #[test]
    fn pha() -> Result<(), CpuError> {
        let mut cpu = create_cpu();
        cpu.accumulator = 0x42;
        let org_status = cpu.status.get_status();
        execute_pha(AddressingMode::Implied, &mut cpu).unwrap();
        assert_eq!(cpu.stack.pop_byte(&cpu.memory)?, 0x42);
        assert_eq!(cpu.status.get_status(), org_status);
        Ok(())
    }

    #[test]
    fn php() -> Result<(), CpuError> {
        let mut cpu = create_cpu();
        cpu.accumulator = 0x42;

        cpu.status.set_break(true); // break flag should be removed on push
        cpu.status.set_carry(true);
        cpu.status.set_decimal_mode(true);
        let org_status = cpu.status.get_status();
        execute_php(AddressingMode::Implied, &mut cpu).unwrap();
        // break and undefined flags are ignored
        assert_eq!(cpu.stack.pop_byte(&cpu.memory)?, org_status & 0xCF);
        Ok(())
    }

    #[test]
    fn pla() -> Result<(), CpuError> {
        let mut cpu = create_cpu();
        cpu.stack.push_byte(&mut cpu.memory, 0x42)?;
        cpu.status.set_negative(true);
        cpu.status.set_zero(true);
        execute_pla(AddressingMode::Implied, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0x42);
        assert!(!cpu.status.zero());
        assert!(!cpu.status.negative());
        Ok(())
    }

    #[test]
    fn plp() -> Result<(), CpuError> {
        let mut cpu = create_cpu();
        let pushed_status = 0b0100_1100;
        cpu.stack.push_byte(&mut cpu.memory, pushed_status)?;
        cpu.status.set_negative(true);
        cpu.status.set_zero(true);
        cpu.status.set_break_command(true);
        execute_plp(AddressingMode::Implied, &mut cpu).unwrap();
        // break and undefined flags are ignored
        assert_eq!(cpu.status.get_status(), pushed_status & 0xCF);
        Ok(())
    }
}
