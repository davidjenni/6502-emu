use crate::cpu::{AddressingMode, Cpu};
use crate::CpuError;

// Set/clear status flags:

// CLC: Clear carry flag
// 0 -> C
//         76543210
// status: .. ....c
#[allow(dead_code)] // TODO remove
pub fn execute_clc(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    validate_mode(mode)?;
    cpu.status.set_carry(false);
    Ok(())
}

// CLD: Clear decimal flag
// 0 -> D
//         76543210
// status: .. .d...
#[allow(dead_code)] // TODO remove
pub fn execute_cld(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    validate_mode(mode)?;
    cpu.status.set_decimal_mode(false);
    Ok(())
}

// CLI: Clear interrupt flag
// 0 -> I
//         76543210
// status: .. ..i..
#[allow(dead_code)] // TODO remove
pub fn execute_cli(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    validate_mode(mode)?;
    cpu.status.set_interrupt_disable(false);
    Ok(())
}

// CLV: Clear overflow flag
// 0 -> C
//         76543210
// status: .v .....
#[allow(dead_code)] // TODO remove
pub fn execute_clv(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    validate_mode(mode)?;
    cpu.status.set_overflow(false);
    Ok(())
}

//--------

// SEC: Set carry flag
// 1 -> C
//         76543210
// status: .. ....C
pub fn execute_sec(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    validate_mode(mode)?;
    cpu.status.set_carry(true);
    Ok(())
}

// SED: Set decimal flag
// 1 -> D
//         76543210
// status: .. .D...
#[allow(dead_code)] // TODO remove
pub fn execute_sed(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    validate_mode(mode)?;
    cpu.status.set_decimal_mode(true);
    Ok(())
}

// SEI: Set interrupt flag
// 1 -> I
//         76543210
// status: .. ..I..
#[allow(dead_code)] // TODO remove
pub fn execute_sei(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    validate_mode(mode)?;
    cpu.status.set_interrupt_disable(true);
    Ok(())
}

fn validate_mode(mode: AddressingMode) -> Result<(), CpuError> {
    if mode != AddressingMode::Implied {
        return Err(CpuError::InvalidAddressingMode);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_flags_address_mode_implied() -> Result<(), CpuError> {
        let mut cpu = Cpu::new();
        assert_eq!(
            execute_clc(AddressingMode::Absolute, &mut cpu),
            Err(CpuError::InvalidAddressingMode)
        );
        assert_eq!(
            execute_cld(AddressingMode::AbsoluteX, &mut cpu),
            Err(CpuError::InvalidAddressingMode)
        );
        assert_eq!(
            execute_cli(AddressingMode::AbsoluteY, &mut cpu),
            Err(CpuError::InvalidAddressingMode)
        );
        assert_eq!(
            execute_clv(AddressingMode::Immediate, &mut cpu),
            Err(CpuError::InvalidAddressingMode)
        );

        assert_eq!(
            execute_sec(AddressingMode::ZeroPage, &mut cpu),
            Err(CpuError::InvalidAddressingMode)
        );
        assert_eq!(
            execute_sed(AddressingMode::ZeroPageX, &mut cpu),
            Err(CpuError::InvalidAddressingMode)
        );
        assert_eq!(
            execute_sei(AddressingMode::Relative, &mut cpu),
            Err(CpuError::InvalidAddressingMode)
        );
        Ok(())
    }

    #[test]
    fn set_and_clear_all_status_flags() -> Result<(), CpuError> {
        let mut cpu = Cpu::new();

        execute_sec(AddressingMode::Implied, &mut cpu)?;
        assert!(cpu.status.carry());
        execute_clc(AddressingMode::Implied, &mut cpu)?;
        assert!(!cpu.status.carry());

        execute_sed(AddressingMode::Implied, &mut cpu)?;
        assert!(cpu.status.decimal_mode());
        execute_cld(AddressingMode::Implied, &mut cpu)?;
        assert!(!cpu.status.decimal_mode());

        execute_sei(AddressingMode::Implied, &mut cpu)?;
        assert!(cpu.status.interrupt_disable());
        execute_cli(AddressingMode::Implied, &mut cpu)?;
        assert!(!cpu.status.interrupt_disable());

        cpu.status.set_overflow(true);
        execute_clv(AddressingMode::Implied, &mut cpu)?;
        assert!(!cpu.status.overflow());

        Ok(())
    }
}
