use crate::cpu::{AddressingMode, Cpu};
use crate::CpuError;

// ALU operations:

// ADC:    A + M + C -> A, C
// status: NV ...ZC
pub fn execute_adc(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    if cpu.status.decimal_mode() {
        todo!("decimal mode not implemented");
    }
    let operand = cpu.get_effective_operand(mode)?;

    let result = cpu.accumulator as u16 + operand as u16 + cpu.status.carry() as u16;

    let acc = result as u8;
    let is_overflow = is_overflow(cpu.accumulator, operand, acc & 0x80);
    cpu.status.set_overflow(is_overflow);
    cpu.accumulator = acc;
    cpu.status.update_from(acc);
    cpu.status.set_carry(result > 0xFF);
    Ok(())
}

// SBC:    A - M - C̅ -> A
// status: NV ...ZC
pub fn execute_sbc(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    if cpu.status.decimal_mode() {
        todo!("decimal mode not implemented");
    }
    let borrow: i16 = if cpu.status.carry() { 0 } else { 1 };
    let operand = cpu.get_effective_operand(mode)?;

    let result = cpu.accumulator as i16 - operand as i16 + borrow;

    let acc = result as u8;
    let is_overflow = is_overflow(cpu.accumulator, operand, acc & 0x80);
    cpu.status.set_overflow(is_overflow);
    cpu.accumulator = acc;
    cpu.status.update_from(result as u8);
    cpu.status.set_carry(result >= 0);
    Ok(())
}

fn is_overflow(a: u8, b: u8, result_high_bit: u8) -> bool {
    // Overflow bit is set when the sign of the result is not the same as the sign of both operands
    // see also MOS6502 Programmer's Manual, p. 11, section 2.2.1.1 Signed Arithmetic:
    //
    // "... The overflow occurs whenever the sign bit (bit 7) is changed as a result of the operation."
    let a_sign = a & 0x80;
    let b_sign = b & 0x80;
    (a_sign == b_sign) && (a_sign != result_high_bit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RamMemory;

    // create a Cpu instance and write operand value into zero page address $00
    fn setup_cpu(operand: u8, carry: bool) -> Cpu {
        let mut cpu = Cpu::new(Box::default() as Box<RamMemory>);
        cpu.address_bus.write(0x0000, operand).unwrap();
        cpu.address_bus.set_pc(0x0000).unwrap();
        // idiosyncrasy of 6502:
        // for first byte, carry flag must be cleared for ADC, set for SBC
        cpu.status.set_carry(carry);
        cpu
    }

    #[test]
    fn alu_add_with_carry() {
        // calculate result, no overflow: 211 + 14 = 225
        let mut cpu = setup_cpu(211, false);
        cpu.accumulator = 14;
        execute_adc(AddressingMode::Immediate, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 225);
        assert!(!cpu.status.carry()); // carry not set
        assert!(!cpu.status.overflow()); // no overflow
        assert!(cpu.status.negative()); // negative result

        // calculate result, with carry but no overflow: 222 + 42 = 264
        let mut cpu = setup_cpu(222, false);
        cpu.accumulator = 42;
        execute_adc(AddressingMode::Immediate, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 8); // result has wrap around
        assert!(cpu.status.carry()); // carry set
        assert!(!cpu.status.overflow()); // no overflow
        assert!(!cpu.status.negative()); // positive result

        // calculate result, no carry but with overflow: 127 + 2 = unsigned 129 = signed -127
        let mut cpu = setup_cpu(2, false);
        cpu.accumulator = 127;
        execute_adc(AddressingMode::Immediate, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0x81); // == -127
        assert!(!cpu.status.carry()); // no carry set
        assert!(cpu.status.overflow()); // overflow !!!
        assert!(cpu.status.negative()); // negative result
    }

    #[test]
    fn alu_subtract_with_carry() {
        // calculate positive result: 56 - 14 = 42
        let mut cpu = setup_cpu(14, true);
        cpu.accumulator = 56;
        execute_sbc(AddressingMode::Immediate, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 42);
        assert!(cpu.status.carry()); //     carry still set
        assert!(!cpu.status.overflow()); // no overflow
        assert!(!cpu.status.negative()); // positive result

        // calculate negative result: 4 - 9 = -5
        let mut cpu = setup_cpu(9, true);
        cpu.accumulator = 4;
        execute_sbc(AddressingMode::Immediate, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0xFB); // -5
        assert!(!cpu.status.carry()); //   carry cleared -> borrow
        assert!(cpu.status.overflow()); // overflow !!
        assert!(cpu.status.negative()); // negative result

        // calculate result with overflow
        let mut cpu = setup_cpu(150, true);
        cpu.accumulator = 2;
        execute_sbc(AddressingMode::Immediate, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 108); // wrapped around result
        assert!(!cpu.status.carry()); //    carry set
        assert!(!cpu.status.overflow()); // no overflow
        assert!(!cpu.status.negative()); // positive result
    }
}
