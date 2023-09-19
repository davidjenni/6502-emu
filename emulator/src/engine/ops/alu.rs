use crate::cpu::{AddressingMode, Cpu};
use crate::CpuError;

// Arithmetic operations:

// ADC:    A + M + C -> A, C
// status: NV ...ZC
pub fn execute_adc(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    if cpu.status.decimal_mode() {
        todo!("decimal mode not implemented");
    }
    let operand = cpu.get_effective_operand(mode)?;

    let result = cpu.accumulator as u16 + operand as u16 + cpu.status.carry() as u16;

    let acc = result as u8;
    let is_overflow = is_overflow(cpu.accumulator, operand, acc.into());
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
    let operand = cpu.get_effective_operand(mode)?;

    let (acc, carry, overflow) = subtract_with_carry(cpu.accumulator, operand, cpu.status.carry());

    cpu.status.set_overflow(overflow);
    cpu.accumulator = acc;
    cpu.status.update_from(acc);
    cpu.status.set_carry(carry);
    Ok(())
}

pub fn subtract_with_carry(register: u8, operand: u8, carry: bool) -> (u8, bool, bool) {
    let borrow: i16 = if carry { 0 } else { 1 };

    let result = register as i16 - operand as i16 + borrow;

    // let res_byte = result as u8;
    let is_overflow = is_overflow(register, operand, result);
    (result as u8, result >= 0, is_overflow)
}

fn is_overflow(a: u8, b: u8, result_high_bit: i16) -> bool {
    let high_bit = (result_high_bit & 0x80) as u8;
    // Overflow bit is set when the sign of the result is not the same as the sign of both operands
    // see also MOS6502 Programmer's Manual, p. 11, section 2.2.1.1 Signed Arithmetic:
    //
    // "... The overflow occurs whenever the sign bit (bit 7) is changed as a result of the operation."
    let a_sign = a & 0x80;
    let b_sign = b & 0x80;
    (a_sign == b_sign) && (a_sign != high_bit)
}

// Logical Operations:

// AND:    A AND M -> A
// status: N. ...Z.
pub fn execute_and(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let operand = cpu.get_effective_operand(mode)?;
    cpu.accumulator &= operand;
    cpu.status.update_from(cpu.accumulator);
    Ok(())
}

// EOR:    A EOR M -> A
// status: N. ...Z.
pub fn execute_eor(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let operand = cpu.get_effective_operand(mode)?;
    cpu.accumulator ^= operand;
    cpu.status.update_from(cpu.accumulator);
    Ok(())
}

// ORA:    A OR M -> A
// status: N. ...Z.
pub fn execute_ora(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let operand = cpu.get_effective_operand(mode)?;
    cpu.accumulator |= operand;
    cpu.status.update_from(cpu.accumulator);
    Ok(())
}

// Shift and rotate operations:

// ASL:    C <- [76543210] <- 0
// status: N. ...ZC
// affects either accumulator or memory (read/modify/write)
pub fn execute_asl(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    read_modify_write(mode, cpu, |operand, _| {
        let carry = operand & 0x80 != 0;
        let result = operand << 1;
        (result, carry)
    })?;
    Ok(())
}

// LSR:    0 -> [76543210] <- C
// status: N. ...ZC
// affects either accumulator or memory (read/modify/write)
pub fn execute_lsr(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    read_modify_write(mode, cpu, |operand, _| {
        let carry = operand & 0x01 != 0;
        let result = operand >> 1;
        (result, carry)
    })?;
    Ok(())
}

// ROL:    C <- [76543210] <- C
// status: N. ...ZC
// affects either accumulator or memory (read/modify/write)
pub fn execute_rol(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    read_modify_write(mode, cpu, |operand, old_carry| {
        let carry_mask = if old_carry { 0x01 } else { 0x00 };
        let new_carry = operand & 0x80 != 0;
        let result = (operand << 1) | carry_mask;
        (result, new_carry)
    })?;
    Ok(())
}

// ROR:    C -> [76543210] -> C
// status: N. ...ZC
// affects either accumulator or memory (read/modify/write)
pub fn execute_ror(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    read_modify_write(mode, cpu, |operand, old_carry| {
        let carry_mask = if old_carry { 0x80 } else { 0x00 };
        let new_carry = operand & 0x01 != 0;
        let result = (operand >> 1) | carry_mask;
        (result, new_carry)
    })?;
    Ok(())
}

// Increment/decrement operations:

// DEC: Decrement memory by one
// M - 1 -> M
// status: N. ...Z.
pub fn execute_dec(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    read_modify_write(mode, cpu, |operand, _| (operand.wrapping_sub(1), false))?;
    Ok(())
}

// DEX: Decrement index X by one
// X - 1 -> X
// status: N. ...Z.
pub fn execute_dex(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    if mode != AddressingMode::Implied {
        return Err(CpuError::InvalidAddressingMode);
    }
    cpu.index_x = cpu.index_x.wrapping_sub(1);
    cpu.status.update_from(cpu.index_x);
    Ok(())
}

// DEY: Decrement index Y by one
// Y - 1 -> Y
// status: N. ...Z.
pub fn execute_dey(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    if mode != AddressingMode::Implied {
        return Err(CpuError::InvalidAddressingMode);
    }
    cpu.index_y = cpu.index_y.wrapping_sub(1);
    cpu.status.update_from(cpu.index_y);
    Ok(())
}

// INC: Increment memory by one
// M + 1 -> M
// status: N. ...Z.
pub fn execute_inc(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    read_modify_write(mode, cpu, |operand, _| (operand.wrapping_add(1), false))?;
    Ok(())
}

// INX: Increment index X by one
// X + 1 -> X
// status: N. ...Z.
pub fn execute_inx(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    if mode != AddressingMode::Implied {
        return Err(CpuError::InvalidAddressingMode);
    }
    cpu.index_x = cpu.index_x.wrapping_add(1);
    cpu.status.update_from(cpu.index_x);
    Ok(())
}

// INY: Increment index Y by one
// Y + 1 -> Y
// status: N. ...Z.
pub fn execute_iny(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    if mode != AddressingMode::Implied {
        return Err(CpuError::InvalidAddressingMode);
    }
    cpu.index_y = cpu.index_y.wrapping_add(1);
    cpu.status.update_from(cpu.index_y);
    Ok(())
}

// function to handle the divergent accumulator vs in-memory read prolog and write sequel
fn read_modify_write(
    mode: AddressingMode,
    cpu: &mut Cpu,
    f: fn(u8, bool) -> (u8, bool),
) -> Result<(), CpuError> {
    // determine read source for operand:
    let (operand, address) = if mode == AddressingMode::Accumulator {
        (cpu.accumulator, None)
    } else {
        let address = cpu.get_effective_address(mode)?;
        (cpu.memory.read(address)?, Some(address))
    };

    let (result, carry) = f(operand, cpu.status.carry());

    // write result back to accumulator or memory:
    cpu.status.update_from(result);
    cpu.status.set_carry(carry);
    if mode == AddressingMode::Accumulator {
        cpu.accumulator = result;
    } else {
        cpu.memory.write(address.unwrap(), result)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZERO_PAGE_ADDR: u16 = 0x00E0;
    const NEXT_PC: u16 = 0x0200;

    // create a Cpu instance and write operand value into zero page address $00
    fn setup_cpu(operand: u8, carry: bool) -> Result<Cpu, CpuError> {
        let mut cpu = create_cpu()?;
        cpu.memory.write(0x0000, operand)?;
        // idiosyncrasy of 6502:
        // for first byte, carry flag must be cleared for ADC, set for SBC
        cpu.status.set_carry(carry);
        Ok(cpu)
    }

    fn setup_cpu_zero_page(operand: u8) -> Result<Cpu, CpuError> {
        let mut cpu = create_cpu()?;
        cpu.memory.write(ZERO_PAGE_ADDR, operand)?;
        cpu.memory.write(NEXT_PC, ZERO_PAGE_ADDR as u8)?;
        cpu.address_bus.set_pc(NEXT_PC)?;
        Ok(cpu)
    }

    fn create_cpu() -> Result<Cpu, CpuError> {
        let mut cpu = Cpu::default();
        cpu.address_bus.set_pc(0x0000)?;
        Ok(cpu)
    }

    #[test]
    fn add_with_carry() -> Result<(), CpuError> {
        // calculate result, no overflow: 211 + 14 = 225
        let mut cpu = setup_cpu(211, false)?;
        cpu.accumulator = 14;
        execute_adc(AddressingMode::Immediate, &mut cpu)?;
        assert_eq!(cpu.accumulator, 225);
        assert!(!cpu.status.carry()); // carry not set
        assert!(!cpu.status.overflow()); // no overflow
        assert!(cpu.status.negative()); // negative result

        // calculate result, with carry but no overflow: 222 + 42 = 264
        let mut cpu = setup_cpu(222, false)?;
        cpu.accumulator = 42;
        execute_adc(AddressingMode::Immediate, &mut cpu)?;
        assert_eq!(cpu.accumulator, 8); // result has wrap around
        assert!(cpu.status.carry()); // carry set
        assert!(!cpu.status.overflow()); // no overflow
        assert!(!cpu.status.negative()); // positive result

        // calculate result, no carry but with overflow: 127 + 2 = unsigned 129 = signed -127
        let mut cpu = setup_cpu(2, false)?;
        cpu.accumulator = 127;
        execute_adc(AddressingMode::Immediate, &mut cpu)?;
        assert_eq!(cpu.accumulator, 0x81); // == -127
        assert!(!cpu.status.carry()); // no carry set
        assert!(cpu.status.overflow()); // overflow !!!
        assert!(cpu.status.negative()); // negative result
        Ok(())
    }

    #[test]
    fn subtract_with_carry() -> Result<(), CpuError> {
        // calculate positive result: 56 - 14 = 42
        let mut cpu = setup_cpu(14, true)?;
        cpu.accumulator = 56;
        execute_sbc(AddressingMode::Immediate, &mut cpu)?;
        assert_eq!(cpu.accumulator, 42);
        assert!(cpu.status.carry()); //     carry still set
        assert!(!cpu.status.overflow()); // no overflow
        assert!(!cpu.status.negative()); // positive result

        // calculate negative result: 4 - 9 = -5
        let mut cpu = setup_cpu(9, true)?;
        cpu.accumulator = 4;
        execute_sbc(AddressingMode::Immediate, &mut cpu)?;
        assert_eq!(cpu.accumulator, 0xFB); // -5
        assert!(!cpu.status.carry()); //   carry cleared -> borrow
        assert!(cpu.status.overflow()); // overflow !!
        assert!(cpu.status.negative()); // negative result

        // calculate result with overflow
        let mut cpu = setup_cpu(150, true)?;
        cpu.accumulator = 2;
        execute_sbc(AddressingMode::Immediate, &mut cpu)?;
        assert_eq!(cpu.accumulator, 108); // wrapped around result
        assert!(!cpu.status.carry()); //    carry set
        assert!(!cpu.status.overflow()); // no overflow
        assert!(!cpu.status.negative()); // positive result
        Ok(())
    }

    #[test]
    fn and() -> Result<(), CpuError> {
        let mut cpu = setup_cpu(0b1010_1010, false)?;
        cpu.accumulator = 0b1111_0000;
        execute_and(AddressingMode::Immediate, &mut cpu)?;
        assert_eq!(cpu.accumulator, 0b1010_0000);
        assert!(cpu.status.negative());
        assert!(!cpu.status.zero());

        let mut cpu = setup_cpu(0b1010_1010, false)?;
        cpu.accumulator = 0b0000_0000;
        execute_and(AddressingMode::Immediate, &mut cpu)?;
        assert_eq!(cpu.accumulator, 0b0000_0000);
        assert!(!cpu.status.negative());
        assert!(cpu.status.zero());
        Ok(())
    }

    #[test]
    fn eor() -> Result<(), CpuError> {
        let mut cpu = setup_cpu(0b1010_1010, false)?;
        cpu.accumulator = 0b1111_0000;
        execute_eor(AddressingMode::Immediate, &mut cpu)?;
        assert_eq!(cpu.accumulator, 0b0101_1010);
        assert!(!cpu.status.negative());
        assert!(!cpu.status.zero());

        let mut cpu = setup_cpu(0b1010_1010, false)?;
        cpu.accumulator = 0b1010_1010;
        execute_eor(AddressingMode::Immediate, &mut cpu)?;
        assert_eq!(cpu.accumulator, 0b0000_0000);
        assert!(!cpu.status.negative());
        assert!(cpu.status.zero());
        Ok(())
    }

    #[test]
    fn ora() -> Result<(), CpuError> {
        let mut cpu = setup_cpu(0b1010_1010, false)?;
        cpu.accumulator = 0b1111_0000;
        execute_ora(AddressingMode::Immediate, &mut cpu)?;
        assert_eq!(cpu.accumulator, 0b1111_1010);
        assert!(cpu.status.negative());
        assert!(!cpu.status.zero());

        let mut cpu = setup_cpu(0b1010_1010, false)?;
        cpu.accumulator = 0b0000_0000;
        execute_ora(AddressingMode::Immediate, &mut cpu)?;
        assert_eq!(cpu.accumulator, 0b1010_1010);
        assert!(cpu.status.negative());
        assert!(!cpu.status.zero());
        Ok(())
    }

    #[test]
    fn asl() -> Result<(), CpuError> {
        let mut cpu = create_cpu()?;
        cpu.accumulator = 0b1010_1010;
        execute_asl(AddressingMode::Accumulator, &mut cpu)?;
        assert_eq!(cpu.accumulator, 0b0101_0100);
        assert!(!cpu.status.negative());
        assert!(!cpu.status.zero());
        assert!(cpu.status.carry());

        cpu.accumulator = 0b0101_0100;
        execute_asl(AddressingMode::Accumulator, &mut cpu)?;
        assert_eq!(cpu.accumulator, 0b1010_1000);
        assert!(!cpu.status.carry());

        let mut cpu = setup_cpu_zero_page(0b1010_1010)?;
        execute_asl(AddressingMode::ZeroPage, &mut cpu)?;
        assert_eq!(cpu.memory.read(ZERO_PAGE_ADDR)?, 0b0101_0100);
        assert!(!cpu.status.negative());
        assert!(!cpu.status.zero());
        assert!(cpu.status.carry());
        Ok(())
    }

    #[test]
    fn lsr() -> Result<(), CpuError> {
        let mut cpu = create_cpu()?;
        cpu.accumulator = 0b1010_1010;
        execute_lsr(AddressingMode::Accumulator, &mut cpu)?;
        assert_eq!(cpu.accumulator, 0b0101_0101);
        assert!(!cpu.status.negative());
        assert!(!cpu.status.zero());
        assert!(!cpu.status.carry());

        cpu.accumulator = 0b0101_0101;
        execute_lsr(AddressingMode::Accumulator, &mut cpu)?;
        assert_eq!(cpu.accumulator, 0b0010_1010);
        assert!(cpu.status.carry());

        let mut cpu = setup_cpu_zero_page(0b1010_1010)?;
        execute_lsr(AddressingMode::ZeroPage, &mut cpu)?;
        assert_eq!(cpu.memory.read(ZERO_PAGE_ADDR)?, 0b0101_0101);
        assert!(!cpu.status.negative());
        assert!(!cpu.status.zero());
        assert!(!cpu.status.carry());
        Ok(())
    }

    #[test]
    fn rol() -> Result<(), CpuError> {
        let mut cpu = create_cpu()?;
        cpu.accumulator = 0b1010_1010;
        cpu.status.set_carry(true);
        execute_rol(AddressingMode::Accumulator, &mut cpu)?;
        assert_eq!(cpu.accumulator, 0b0101_0101);
        assert!(!cpu.status.negative());
        assert!(!cpu.status.zero());
        assert!(cpu.status.carry());

        cpu.accumulator = 0b1010_1010;
        cpu.status.set_carry(false);
        execute_rol(AddressingMode::Accumulator, &mut cpu)?;
        assert_eq!(cpu.accumulator, 0b0101_0100);
        assert!(cpu.status.carry());

        cpu.accumulator = 0b0101_0101;
        cpu.status.set_carry(false);
        execute_rol(AddressingMode::Accumulator, &mut cpu)?;
        assert_eq!(cpu.accumulator, 0b1010_1010);
        assert!(!cpu.status.carry());
        Ok(())
    }

    #[test]
    fn ror() -> Result<(), CpuError> {
        let mut cpu = create_cpu()?;
        cpu.accumulator = 0b1010_1010;
        cpu.status.set_carry(true);
        execute_ror(AddressingMode::Accumulator, &mut cpu)?;
        assert_eq!(cpu.accumulator, 0b1101_0101);
        assert!(cpu.status.negative());
        assert!(!cpu.status.zero());
        assert!(!cpu.status.carry());

        cpu.accumulator = 0b1010_1010;
        cpu.status.set_carry(false);
        execute_ror(AddressingMode::Accumulator, &mut cpu)?;
        assert_eq!(cpu.accumulator, 0b0101_0101);
        assert!(!cpu.status.carry());

        cpu.accumulator = 0b0101_0101;
        cpu.status.set_carry(false);
        execute_ror(AddressingMode::Accumulator, &mut cpu)?;
        assert_eq!(cpu.accumulator, 0b0010_1010);
        assert!(cpu.status.carry());
        Ok(())
    }

    #[test]
    fn inc() -> Result<(), CpuError> {
        let mut cpu = setup_cpu_zero_page(0x42)?;
        execute_inc(AddressingMode::ZeroPage, &mut cpu)?;
        assert_eq!(cpu.memory.read(ZERO_PAGE_ADDR)?, 0x43);
        assert!(!cpu.status.negative());
        assert!(!cpu.status.zero());

        let mut cpu = setup_cpu_zero_page(0xFF)?;
        execute_inc(AddressingMode::ZeroPage, &mut cpu)?;
        assert_eq!(cpu.memory.read(ZERO_PAGE_ADDR)?, 0x00);
        assert!(!cpu.status.negative());
        assert!(cpu.status.zero());
        Ok(())
    }

    #[test]
    fn inx() -> Result<(), CpuError> {
        let mut cpu = create_cpu()?;
        cpu.index_x = 0x42;
        execute_inx(AddressingMode::Implied, &mut cpu)?;
        assert_eq!(cpu.index_x, 0x43);
        assert!(!cpu.status.negative());
        assert!(!cpu.status.zero());

        let mut cpu = create_cpu()?;
        cpu.index_x = 0xFF;
        execute_inx(AddressingMode::Implied, &mut cpu)?;
        assert_eq!(cpu.index_x, 0x00);
        assert!(!cpu.status.negative());
        assert!(cpu.status.zero());
        Ok(())
    }

    #[test]
    fn iny() -> Result<(), CpuError> {
        let mut cpu = create_cpu()?;
        cpu.index_y = 0x42;
        execute_iny(AddressingMode::Implied, &mut cpu)?;
        assert_eq!(cpu.index_y, 0x43);
        assert!(!cpu.status.negative());
        assert!(!cpu.status.zero());

        let mut cpu = create_cpu()?;
        cpu.index_y = 0xFF;
        execute_iny(AddressingMode::Implied, &mut cpu)?;
        assert_eq!(cpu.index_y, 0x00);
        assert!(!cpu.status.negative());
        assert!(cpu.status.zero());
        Ok(())
    }

    #[test]
    fn dec() -> Result<(), CpuError> {
        let mut cpu = setup_cpu_zero_page(0x42)?;
        execute_dec(AddressingMode::ZeroPage, &mut cpu)?;
        assert_eq!(cpu.memory.read(ZERO_PAGE_ADDR)?, 0x41);
        assert!(!cpu.status.negative());
        assert!(!cpu.status.zero());

        let mut cpu = setup_cpu_zero_page(0x01)?;
        execute_dec(AddressingMode::ZeroPage, &mut cpu)?;
        assert_eq!(cpu.memory.read(ZERO_PAGE_ADDR)?, 0x00);
        assert!(!cpu.status.negative());
        assert!(cpu.status.zero());

        let mut cpu = setup_cpu_zero_page(0x00)?;
        execute_dec(AddressingMode::ZeroPage, &mut cpu)?;
        assert_eq!(cpu.memory.read(ZERO_PAGE_ADDR)?, 0xFF);
        assert!(cpu.status.negative());
        assert!(!cpu.status.zero());
        Ok(())
    }

    #[test]
    fn dex() -> Result<(), CpuError> {
        let mut cpu = create_cpu()?;
        cpu.index_x = 0x42;
        execute_dex(AddressingMode::Implied, &mut cpu)?;
        assert_eq!(cpu.index_x, 0x41);
        assert!(!cpu.status.negative());
        assert!(!cpu.status.zero());

        let mut cpu = create_cpu()?;
        cpu.index_x = 0x01;
        execute_dex(AddressingMode::Implied, &mut cpu)?;
        assert_eq!(cpu.index_x, 0x00);
        assert!(!cpu.status.negative());
        assert!(cpu.status.zero());

        let mut cpu = create_cpu()?;
        cpu.index_x = 0x00;
        execute_dex(AddressingMode::Implied, &mut cpu)?;
        assert_eq!(cpu.index_x, 0xFF);
        assert!(cpu.status.negative());
        assert!(!cpu.status.zero());
        Ok(())
    }

    #[test]
    fn dey() -> Result<(), CpuError> {
        let mut cpu = create_cpu()?;
        cpu.index_y = 0x42;
        execute_dey(AddressingMode::Implied, &mut cpu)?;
        assert_eq!(cpu.index_y, 0x41);
        assert!(!cpu.status.negative());
        assert!(!cpu.status.zero());

        let mut cpu = create_cpu()?;
        cpu.index_y = 0x01;
        execute_dey(AddressingMode::Implied, &mut cpu)?;
        assert_eq!(cpu.index_y, 0x00);
        assert!(!cpu.status.negative());
        assert!(cpu.status.zero());

        let mut cpu = create_cpu()?;
        cpu.index_y = 0x00;
        execute_dey(AddressingMode::Implied, &mut cpu)?;
        assert_eq!(cpu.index_y, 0xFF);
        assert!(cpu.status.negative());
        assert!(!cpu.status.zero());
        Ok(())
    }
}
