use crate::cpu::{AddressingMode, Cpu};
use crate::engine::ops::alu::subtract_with_carry;
use crate::CpuError;

// Set/clear status flag operations:

// CLC: Clear carry flag
// 0 -> C
//         76543210
// status: .. ....c
pub fn execute_clc(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    validate_mode(mode)?;
    cpu.status.set_carry(false);
    Ok(())
}

// CLD: Clear decimal flag
// 0 -> D
//         76543210
// status: .. .d...
pub fn execute_cld(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    validate_mode(mode)?;
    cpu.status.set_decimal_mode(false);
    Ok(())
}

// CLI: Clear interrupt flag
// 0 -> I
//         76543210
// status: .. ..i..
pub fn execute_cli(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    validate_mode(mode)?;
    cpu.status.set_interrupt_disable(false);
    Ok(())
}

// CLV: Clear overflow flag
// 0 -> C
//         76543210
// status: .v .....
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
pub fn execute_sed(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    validate_mode(mode)?;
    cpu.status.set_decimal_mode(true);
    Ok(())
}

// SEI: Set interrupt flag
// 1 -> I
//         76543210
// status: .. ..I..
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

// Compare operations:

// BIT: Test bits in memory with accumulator
// A AND M, M7 -> N, M6 -> V
//         76543210
// status: NV ...Z.
pub fn execute_bit(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let operand = cpu.get_effective_operand(mode)?;
    cpu.status.set_negative(operand & 0b1000_0000 != 0);
    cpu.status.set_overflow(operand & 0b0100_0000 != 0);
    cpu.status.set_zero((cpu.accumulator & operand) == 0);
    Ok(())
}

// result = reg - M     Z   C . N
// Register < Operand	0	0	sign bit of result
// Register = Operand	1	1	0
// Register > Operand	0	1	sign bit of result

// CMP: Compare accumulator
// A - M -> C,Z,N
//         76543210
// status: N. ...ZC
pub fn execute_cmp(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    compare_register(cpu.accumulator, mode, cpu)?;
    Ok(())
}

// CPX: Compare X register
// X - M -> C,Z,N
//         76543210
// status: N. ...ZC
pub fn execute_cpx(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    compare_register(cpu.index_x, mode, cpu)?;
    Ok(())
}

// CPY: Compare Y register
// X - M -> C,Z,N
//         76543210
// status: N. ...ZC
pub fn execute_cpy(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    compare_register(cpu.index_y, mode, cpu)?;
    Ok(())
}

fn compare_register(register: u8, mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let operand = cpu.get_effective_operand(mode)?;
    let (result, carry, _) = subtract_with_carry(register, operand, true);
    cpu.status.update_from(result);
    cpu.status.set_carry(carry);
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    const NEXT_PC: u16 = 0x1234;

    fn setup_cpu_for_compare(operand: u8) -> Result<Cpu, CpuError> {
        let mut cpu = Cpu::default();
        cpu.address_bus.set_pc(NEXT_PC)?;
        cpu.memory.write(NEXT_PC, operand)?;
        Ok(cpu)
    }

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

    #[test]
    fn cmp() -> Result<(), CpuError> {
        let mut cpu = setup_cpu_for_compare(42)?;

        // positive inputs:
        // A = M
        cpu.accumulator = 42;
        execute_cmp(AddressingMode::Immediate, &mut cpu)?;
        assert!(cpu.status.zero());
        assert!(cpu.status.carry());
        assert!(!cpu.status.negative());

        // A < M
        cpu.accumulator = 33;
        cpu.address_bus.set_pc(NEXT_PC)?;
        execute_cmp(AddressingMode::Immediate, &mut cpu)?;
        assert!(!cpu.status.zero());
        assert!(!cpu.status.carry());
        assert!(cpu.status.negative());

        // A > M
        cpu.accumulator = 77;
        execute_cmp(AddressingMode::Immediate, &mut cpu)?;
        assert!(!cpu.status.zero());
        assert!(cpu.status.carry());
        assert!(!cpu.status.negative());

        // negative inputs:
        let mut cpu = setup_cpu_for_compare(222)?;

        // A < M
        cpu.accumulator = 33;
        cpu.address_bus.set_pc(NEXT_PC)?;
        execute_cmp(AddressingMode::Immediate, &mut cpu)?;
        assert!(!cpu.status.zero());
        assert!(!cpu.status.carry());
        assert!(!cpu.status.negative());

        // A > M
        cpu.accumulator = 244;
        execute_cmp(AddressingMode::Immediate, &mut cpu)?;
        assert!(!cpu.status.zero());
        assert!(cpu.status.carry());
        assert!(cpu.status.negative());

        Ok(())
    }

    #[test]
    fn cpx() -> Result<(), CpuError> {
        let mut cpu = setup_cpu_for_compare(42)?;

        // positive inputs:
        // X = M
        cpu.index_x = 42;
        execute_cpx(AddressingMode::Immediate, &mut cpu)?;
        assert!(cpu.status.zero());
        assert!(cpu.status.carry());
        assert!(!cpu.status.negative());

        // X < M
        cpu.index_x = 33;
        cpu.address_bus.set_pc(NEXT_PC)?;
        execute_cpx(AddressingMode::Immediate, &mut cpu)?;
        assert!(!cpu.status.zero());
        assert!(!cpu.status.carry());
        assert!(cpu.status.negative());

        // X > M
        cpu.index_x = 77;
        execute_cpx(AddressingMode::Immediate, &mut cpu)?;
        assert!(!cpu.status.zero());
        assert!(cpu.status.carry());
        assert!(!cpu.status.negative());

        Ok(())
    }

    #[test]
    fn cpy() -> Result<(), CpuError> {
        let mut cpu = setup_cpu_for_compare(42)?;

        // positive inputs:
        // Y = M
        cpu.index_y = 42;
        execute_cpy(AddressingMode::Immediate, &mut cpu)?;
        assert!(cpu.status.zero());
        assert!(cpu.status.carry());
        assert!(!cpu.status.negative());

        // Y < M
        cpu.index_y = 33;
        cpu.address_bus.set_pc(NEXT_PC)?;
        execute_cpy(AddressingMode::Immediate, &mut cpu)?;
        assert!(!cpu.status.zero());
        assert!(!cpu.status.carry());
        assert!(cpu.status.negative());

        // Y > M
        cpu.index_y = 77;
        execute_cpy(AddressingMode::Immediate, &mut cpu)?;
        assert!(!cpu.status.zero());
        assert!(cpu.status.carry());
        assert!(!cpu.status.negative());

        Ok(())
    }

    #[test]
    fn bit() -> Result<(), CpuError> {
        let mut cpu = setup_cpu_for_compare(0b1010_1010)?;

        cpu.accumulator = 0b1010_1010;
        execute_bit(AddressingMode::Immediate, &mut cpu)?;
        assert!(!cpu.status.zero());
        assert!(!cpu.status.overflow());
        assert!(cpu.status.negative());

        let mut cpu = setup_cpu_for_compare(0b0101_0101)?;
        cpu.accumulator = 0b1111_0101;
        execute_bit(AddressingMode::Immediate, &mut cpu)?;
        assert!(!cpu.status.zero());
        assert!(cpu.status.overflow());
        assert!(!cpu.status.negative());

        let mut cpu = setup_cpu_for_compare(0b0000_0101)?;
        cpu.accumulator = 0b0000_1010;
        cpu.address_bus.set_pc(NEXT_PC)?;
        cpu.status.set_status(0);
        execute_bit(AddressingMode::Immediate, &mut cpu)?;
        assert!(cpu.status.zero());
        assert!(!cpu.status.overflow());
        assert!(!cpu.status.negative());

        Ok(())
    }
}
