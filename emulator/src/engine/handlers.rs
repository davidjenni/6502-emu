use crate::cpu::{AddressingMode, Cpu};
use crate::CpuError;

// special codes:
pub fn execute_nop(_: AddressingMode, _: &mut Cpu) -> Result<(), CpuError> {
    Ok(())
}

pub fn execute_brk(_: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    cpu.status.set_break(true);
    // TODO: push PC and status to stack, like an IRQ
    Ok(())
}

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

// ALU operations:
// SBC:    A - M - C̅ -> A
// status: NV ...ZC
pub fn execute_sbc(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    if cpu.status.decimal_mode() {
        todo!("decimal mode not implemented");
    }

    let borrow: i16 = if cpu.status.carry() { 0 } else { 1 };
    let left = cpu.accumulator;
    let right = cpu.get_effective_operand(mode)? as i16;
    // 6502 does subtraction by adding the two's complement from effective address to the accumulator
    let value: i16 = right + borrow;
    let result: i16 = cpu.accumulator as i16 - value;
    if result < 0 {
        cpu.accumulator = (result & 0x00FF) as u8;
        cpu.status.set_carry(false);
    } else {
        cpu.accumulator = result as u8;
        cpu.status.set_carry(true);
    }
    cpu.status
        .set_overflow(is_overflow(left, !right as u8, result as u8));
    cpu.status.update_from(result as u8);
    Ok(())
}

fn is_overflow(a: u8, b: u8, result: u8) -> bool {
    // overflow occurs when the sign of the two operands is the same, but the sign of the result is different
    // (i.e. the sign bit changes)
    let a_sign = a & 0x80;
    let b_sign = b & 0x80;
    let result_sign = result & 0x80;
    (a_sign == b_sign) && (a_sign != result_sign)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RamMemory;

    // create a Cpu instance and write operand value into zero page address $00
    fn setup_cpu(operand: u8) -> Cpu {
        let mut cpu = Cpu::new(Box::default() as Box<RamMemory>);
        cpu.address_bus.write(0x0000, operand).unwrap();
        cpu.address_bus.set_pc(0x0000).unwrap();
        cpu
    }

    #[test]
    fn alu_subtract_with_carry() {
        // calculate positive result: 56 - 14 = 42
        let mut cpu = setup_cpu(14);
        cpu.accumulator = 56;
        cpu.status.set_carry(true); // idiosyncrasy of 6502: must set carry flag before subtracting
        execute_sbc(AddressingMode::Immediate, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 42);
        assert!(cpu.status.carry()); // carry still set
        assert!(!cpu.status.overflow()); // no overflow
        assert!(!cpu.status.negative()); // positive result

        // calculate negative result: 4 - 9 = -5
        let mut cpu = setup_cpu(9);
        cpu.accumulator = 4;
        cpu.status.set_carry(true);
        execute_sbc(AddressingMode::Immediate, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0xFB); // -5
        assert!(!cpu.status.carry()); // carry cleared -> borrow
        assert!(!cpu.status.overflow()); // no overflow
        assert!(cpu.status.negative()); // negative result

        // calculate result with overflow
        let mut cpu = setup_cpu(150);
        cpu.accumulator = 2;
        cpu.status.set_carry(true);
        execute_sbc(AddressingMode::Immediate, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 108); // wrapped around result
        assert!(!cpu.status.carry()); // carry set
                                      // assert!(cpu.status.overflow()); // overflow !!
        assert!(!cpu.status.negative()); // positive result
    }
}

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

// Set/clear status flags:
pub fn execute_sec(_: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    cpu.status.set_carry(true);
    Ok(())
}
