use crate::cpu::{AddressingMode, Cpu};
use crate::memory_access::MemoryAccess;
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

// Logical Operations:

// AND:    A AND M -> A
// status: N. ...Z.
#[allow(dead_code)] // TODO remove
pub fn execute_and(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let operand = cpu.get_effective_operand(mode)?;
    cpu.accumulator &= operand;
    cpu.status.update_from(cpu.accumulator);
    Ok(())
}

// EOR:    A EOR M -> A
// status: N. ...Z.
#[allow(dead_code)] // TODO remove
pub fn execute_eor(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let operand = cpu.get_effective_operand(mode)?;
    cpu.accumulator ^= operand;
    cpu.status.update_from(cpu.accumulator);
    Ok(())
}

// ORA:    A OR M -> A
// status: N. ...Z.
#[allow(dead_code)] // TODO remove
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
#[allow(dead_code)] // TODO remove
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
#[allow(dead_code)] // TODO remove
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
#[allow(dead_code)] // TODO remove
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
#[allow(dead_code)] // TODO remove
pub fn execute_ror(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    read_modify_write(mode, cpu, |operand, old_carry| {
        let carry_mask = if old_carry { 0x80 } else { 0x00 };
        let new_carry = operand & 0x01 != 0;
        let result = (operand >> 1) | carry_mask;
        (result, new_carry)
    })?;
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
    use crate::address_bus::AddressBus;

    const ZERO_PAGE_ADDR: u16 = 0x00E0;
    const NEXT_PC: u16 = 0x0200;

    // create a Cpu instance and write operand value into zero page address $00
    fn setup_cpu(operand: u8, carry: bool) -> Cpu {
        let mut cpu = create_cpu();
        cpu.memory.write(0x0000, operand).unwrap();
        // idiosyncrasy of 6502:
        // for first byte, carry flag must be cleared for ADC, set for SBC
        cpu.status.set_carry(carry);
        cpu
    }

    fn setup_cpu_zero_page(operand: u8) -> Cpu {
        let mut cpu = create_cpu();
        cpu.memory.write(ZERO_PAGE_ADDR, operand).unwrap();
        cpu.memory.write(NEXT_PC, ZERO_PAGE_ADDR as u8).unwrap();
        cpu.address_bus.set_pc(NEXT_PC).unwrap();
        cpu
    }

    fn create_cpu() -> Cpu {
        let mut cpu = Cpu::default();
        cpu.address_bus.set_pc(0x0000).unwrap();
        cpu
    }

    #[test]
    fn add_with_carry() {
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
    fn subtract_with_carry() {
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

    #[test]
    fn and() {
        let mut cpu = setup_cpu(0b1010_1010, false);
        cpu.accumulator = 0b1111_0000;
        execute_and(AddressingMode::Immediate, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0b1010_0000);
        assert!(cpu.status.negative());
        assert!(!cpu.status.zero());

        let mut cpu = setup_cpu(0b1010_1010, false);
        cpu.accumulator = 0b0000_0000;
        execute_and(AddressingMode::Immediate, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0b0000_0000);
        assert!(!cpu.status.negative());
        assert!(cpu.status.zero());
    }

    #[test]
    fn eor() {
        let mut cpu = setup_cpu(0b1010_1010, false);
        cpu.accumulator = 0b1111_0000;
        execute_eor(AddressingMode::Immediate, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0b0101_1010);
        assert!(!cpu.status.negative());
        assert!(!cpu.status.zero());

        let mut cpu = setup_cpu(0b1010_1010, false);
        cpu.accumulator = 0b1010_1010;
        execute_eor(AddressingMode::Immediate, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0b0000_0000);
        assert!(!cpu.status.negative());
        assert!(cpu.status.zero());
    }

    #[test]
    fn ora() {
        let mut cpu = setup_cpu(0b1010_1010, false);
        cpu.accumulator = 0b1111_0000;
        execute_ora(AddressingMode::Immediate, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0b1111_1010);
        assert!(cpu.status.negative());
        assert!(!cpu.status.zero());

        let mut cpu = setup_cpu(0b1010_1010, false);
        cpu.accumulator = 0b0000_0000;
        execute_ora(AddressingMode::Immediate, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0b1010_1010);
        assert!(cpu.status.negative());
        assert!(!cpu.status.zero());
    }

    #[test]
    fn asl() {
        let mut cpu = create_cpu();
        cpu.accumulator = 0b1010_1010;
        execute_asl(AddressingMode::Accumulator, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0b0101_0100);
        assert!(!cpu.status.negative());
        assert!(!cpu.status.zero());
        assert!(cpu.status.carry());

        cpu.accumulator = 0b0101_0100;
        execute_asl(AddressingMode::Accumulator, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0b1010_1000);
        assert!(!cpu.status.carry());

        let mut cpu = setup_cpu_zero_page(0b1010_1010);
        execute_asl(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.memory.read(ZERO_PAGE_ADDR).unwrap(), 0b0101_0100);
        assert!(!cpu.status.negative());
        assert!(!cpu.status.zero());
        assert!(cpu.status.carry());
    }

    #[test]
    fn lsr() {
        let mut cpu = create_cpu();
        cpu.accumulator = 0b1010_1010;
        execute_lsr(AddressingMode::Accumulator, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0b0101_0101);
        assert!(!cpu.status.negative());
        assert!(!cpu.status.zero());
        assert!(!cpu.status.carry());

        cpu.accumulator = 0b0101_0101;
        execute_lsr(AddressingMode::Accumulator, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0b0010_1010);
        assert!(cpu.status.carry());

        let mut cpu = setup_cpu_zero_page(0b1010_1010);
        execute_lsr(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.memory.read(ZERO_PAGE_ADDR).unwrap(), 0b0101_0101);
        assert!(!cpu.status.negative());
        assert!(!cpu.status.zero());
        assert!(!cpu.status.carry());
    }

    #[test]
    fn rol() {
        let mut cpu = create_cpu();
        cpu.accumulator = 0b1010_1010;
        cpu.status.set_carry(true);
        execute_rol(AddressingMode::Accumulator, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0b0101_0101);
        assert!(!cpu.status.negative());
        assert!(!cpu.status.zero());
        assert!(cpu.status.carry());

        cpu.accumulator = 0b1010_1010;
        cpu.status.set_carry(false);
        execute_rol(AddressingMode::Accumulator, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0b0101_0100);
        assert!(cpu.status.carry());

        cpu.accumulator = 0b0101_0101;
        cpu.status.set_carry(false);
        execute_rol(AddressingMode::Accumulator, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0b1010_1010);
        assert!(!cpu.status.carry());
    }

    #[test]
    fn ror() {
        let mut cpu = create_cpu();
        cpu.accumulator = 0b1010_1010;
        cpu.status.set_carry(true);
        execute_ror(AddressingMode::Accumulator, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0b1101_0101);
        assert!(cpu.status.negative());
        assert!(!cpu.status.zero());
        assert!(!cpu.status.carry());

        cpu.accumulator = 0b1010_1010;
        cpu.status.set_carry(false);
        execute_ror(AddressingMode::Accumulator, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0b0101_0101);
        assert!(!cpu.status.carry());

        cpu.accumulator = 0b0101_0101;
        cpu.status.set_carry(false);
        execute_ror(AddressingMode::Accumulator, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0b0010_1010);
        assert!(cpu.status.carry());
    }
}
