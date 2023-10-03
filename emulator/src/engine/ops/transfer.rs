use crate::cpu_impl::{AddressingMode, CpuImpl};
use crate::CpuError;

// LDA/X/Y:

// LDA:    M -> A
// status: N. ...Z.
pub fn execute_lda(mode: AddressingMode, cpu: &mut CpuImpl) -> Result<(), CpuError> {
    let value = cpu.get_effective_operand(mode)?;
    cpu.accumulator = value;
    cpu.status.update_from(value);
    Ok(())
}

// LDX:    M -> X
// status: N. ...Z.
pub fn execute_ldx(mode: AddressingMode, cpu: &mut CpuImpl) -> Result<(), CpuError> {
    let value = cpu.get_effective_operand(mode)?;
    cpu.index_x = value;
    cpu.status.update_from(value);
    Ok(())
}

// LDY:    M -> Y
// status: N. ...Z.
pub fn execute_ldy(mode: AddressingMode, cpu: &mut CpuImpl) -> Result<(), CpuError> {
    let value = cpu.get_effective_operand(mode)?;
    cpu.index_y = value;
    cpu.status.update_from(value);
    Ok(())
}

// STA/X/Y:

// STA:    A -> M
// status: N. ...Z.
pub fn execute_sta(mode: AddressingMode, cpu: &mut CpuImpl) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    memory_write_tolerate_readonly(effective_address, cpu.accumulator, cpu)
}

// STX:    X -> M
// status: N. ...Z.
pub fn execute_stx(mode: AddressingMode, cpu: &mut CpuImpl) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    memory_write_tolerate_readonly(effective_address, cpu.index_x, cpu)
}

// STY:    Y -> M
// status: N. ...Z.
pub fn execute_sty(mode: AddressingMode, cpu: &mut CpuImpl) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    memory_write_tolerate_readonly(effective_address, cpu.index_y, cpu)
}

pub fn memory_write_tolerate_readonly(
    effective_address: u16,
    value: u8,
    cpu: &mut CpuImpl,
) -> Result<(), CpuError> {
    match cpu.memory.write(effective_address, value) {
        Ok(_) => {}
        Err(CpuError::ReadOnlyMemory) => {}
        Err(e) => return Err(e),
    }
    cpu.status.update_from(value);
    Ok(())
}

// TAX:    A -> X
// status: N. ...Z.
pub fn execute_tax(mode: AddressingMode, cpu: &mut CpuImpl) -> Result<(), CpuError> {
    if mode != AddressingMode::Implied {
        return Err(CpuError::InvalidAddressingMode);
    }
    cpu.index_x = cpu.accumulator;
    cpu.status.update_from(cpu.index_x);
    Ok(())
}

// TAY:    A -> Y
// status: N. ...Z.
pub fn execute_tay(mode: AddressingMode, cpu: &mut CpuImpl) -> Result<(), CpuError> {
    if mode != AddressingMode::Implied {
        return Err(CpuError::InvalidAddressingMode);
    }
    cpu.index_y = cpu.accumulator;
    cpu.status.update_from(cpu.index_y);
    Ok(())
}

// TXA:    X -> A
// status: N. ...Z.
pub fn execute_txa(mode: AddressingMode, cpu: &mut CpuImpl) -> Result<(), CpuError> {
    if mode != AddressingMode::Implied {
        return Err(CpuError::InvalidAddressingMode);
    }
    cpu.accumulator = cpu.index_x;
    cpu.status.update_from(cpu.accumulator);
    Ok(())
}

// TYA:    Y -> A
// status: N. ...Z.
pub fn execute_tya(mode: AddressingMode, cpu: &mut CpuImpl) -> Result<(), CpuError> {
    if mode != AddressingMode::Implied {
        return Err(CpuError::InvalidAddressingMode);
    }
    cpu.accumulator = cpu.index_y;
    cpu.status.update_from(cpu.accumulator);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZERO_PAGE_ADDR: u16 = 0x00E0;
    const ABS_ADDR: u16 = 0xC0C0;
    const NEXT_PC: u16 = 0x0300;

    // create a Cpu instance and write test value value into zero page address $E0
    // writes zero page address $E0 into next PC address $0300
    fn setup_cpu_for_load(value: u8) -> CpuImpl {
        let mut cpu = create_cpu();
        cpu.memory.write(ZERO_PAGE_ADDR, value).unwrap();
        cpu
    }

    fn setup_cpu_for_store() -> CpuImpl {
        create_cpu()
    }

    fn setup_cpu_for_abs_store(addr: u16) -> CpuImpl {
        let mut cpu = create_cpu();
        cpu.memory.write_word(NEXT_PC, addr).unwrap();
        cpu
    }

    fn create_cpu() -> CpuImpl {
        let mut cpu = CpuImpl::default();
        cpu.memory.write(NEXT_PC, ZERO_PAGE_ADDR as u8).unwrap();
        cpu.address_bus.set_pc(NEXT_PC).unwrap();
        cpu
    }

    #[test]
    fn test_lda() {
        let mut cpu = setup_cpu_for_load(0x42);
        execute_lda(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0x42);
        assert!(!cpu.status.zero());
        assert!(!cpu.status.negative());

        // is negative flag updated?
        let test_val = -1i8 as u8;
        let mut cpu = setup_cpu_for_load(test_val);
        execute_lda(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, test_val);
        assert!(!cpu.status.zero());
        assert!(cpu.status.negative());
    }

    #[test]
    fn test_ldx() {
        let mut cpu = setup_cpu_for_load(0x42);
        execute_ldx(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.index_x, 0x42);
        assert!(!cpu.status.zero());
        assert!(!cpu.status.negative());

        // is negative flag updated?
        let test_val = -1i8 as u8;
        let mut cpu = setup_cpu_for_load(test_val);
        execute_ldx(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.index_x, test_val);
        assert!(!cpu.status.zero());
        assert!(cpu.status.negative());
        // clear value in zero page address, status flags should be updated
        cpu.memory.write(ZERO_PAGE_ADDR, 0).unwrap();
        execute_ldx(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.index_x, 0);
        assert!(cpu.status.zero());
        assert!(!cpu.status.negative());
    }

    #[test]
    fn test_ldy() {
        let mut cpu = setup_cpu_for_load(0x42);
        execute_ldy(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.index_y, 0x42);
        assert!(!cpu.status.zero());
        assert!(!cpu.status.negative());

        // is negative flag updated?
        let test_val = -1i8 as u8;
        let mut cpu = setup_cpu_for_load(test_val);
        execute_ldy(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.index_y, test_val);
        assert!(!cpu.status.zero());
        assert!(cpu.status.negative());
        // clear value in zero page address, status flags should be updated
        cpu.memory.write(0x00E0, 0).unwrap();
        execute_ldy(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.index_y, 0);
        assert!(cpu.status.zero());
        assert!(!cpu.status.negative());
    }

    #[test]
    fn test_sta() {
        let mut cpu = setup_cpu_for_store();
        cpu.accumulator = 0x42;
        execute_sta(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.memory.read(ZERO_PAGE_ADDR).unwrap(), 0x42);
        assert!(!cpu.status.zero());
        assert!(!cpu.status.negative());

        // is negative flag updated?
        let test_val = (-123 & 0xFF) as u8;
        cpu.accumulator = test_val;
        cpu.address_bus.set_pc(NEXT_PC).unwrap();
        execute_sta(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.memory.read(ZERO_PAGE_ADDR).unwrap(), test_val);
        assert!(!cpu.status.zero());
        assert!(cpu.status.negative());
    }

    #[test]
    fn test_stx() {
        let mut cpu = setup_cpu_for_store();
        cpu.index_x = 0x42;
        execute_stx(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.memory.read(ZERO_PAGE_ADDR).unwrap(), 0x42);
        assert!(!cpu.status.zero());
        assert!(!cpu.status.negative());

        // is negative flag updated?
        let test_val = (-123 & 0xFF) as u8;
        cpu.index_x = test_val;
        cpu.address_bus.set_pc(NEXT_PC).unwrap();
        execute_stx(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.memory.read(ZERO_PAGE_ADDR).unwrap(), test_val);
        assert!(!cpu.status.zero());
        assert!(cpu.status.negative());
    }

    #[test]
    fn test_sty() {
        let mut cpu = setup_cpu_for_store();
        cpu.index_y = 0x42;
        execute_sty(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.memory.read(ZERO_PAGE_ADDR).unwrap(), 0x42);
        assert!(!cpu.status.zero());
        assert!(!cpu.status.negative());

        // is negative flag updated?
        let test_val = (-123 & 0xFF) as u8;
        cpu.index_y = test_val;
        cpu.address_bus.set_pc(NEXT_PC).unwrap();
        execute_sty(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.memory.read(ZERO_PAGE_ADDR).unwrap(), test_val);
        assert!(!cpu.status.zero());
        assert!(cpu.status.negative());
    }

    #[test]
    fn test_tax() {
        let mut cpu = create_cpu();
        cpu.accumulator = 0x42;
        execute_tax(AddressingMode::Implied, &mut cpu).unwrap();
        assert_eq!(cpu.index_x, 0x42);
        assert!(!cpu.status.zero());
        assert!(!cpu.status.negative());

        // is negative flag updated?
        let test_val = (-123 & 0xFF) as u8;
        cpu.accumulator = test_val;
        cpu.address_bus.set_pc(NEXT_PC).unwrap();
        execute_tax(AddressingMode::Implied, &mut cpu).unwrap();
        assert_eq!(cpu.index_x, test_val);
        assert!(!cpu.status.zero());
        assert!(cpu.status.negative());
    }

    #[test]
    #[should_panic(expected = "InvalidAddressingMode")]
    fn test_tay_illegal_address_mode() {
        let mut cpu = create_cpu();
        cpu.accumulator = 0x42;
        execute_tay(AddressingMode::Immediate, &mut cpu).unwrap();
    }

    #[test]
    fn test_tay() {
        let mut cpu = create_cpu();
        cpu.accumulator = 0x42;
        execute_tay(AddressingMode::Implied, &mut cpu).unwrap();
        assert_eq!(cpu.index_y, 0x42);
        assert!(!cpu.status.zero());
        assert!(!cpu.status.negative());
    }

    #[test]
    fn test_txa() {
        let mut cpu = create_cpu();
        cpu.index_x = 0x42;
        execute_txa(AddressingMode::Implied, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0x42);
        assert!(!cpu.status.zero());
        assert!(!cpu.status.negative());
    }

    #[test]
    fn test_tya() {
        let mut cpu = create_cpu();
        cpu.index_y = 0x42;
        execute_tya(AddressingMode::Implied, &mut cpu).unwrap();
        assert_eq!(cpu.accumulator, 0x42);
        assert!(!cpu.status.zero());
        assert!(!cpu.status.negative());
    }

    #[test]
    fn test_sta_readonly_memory() -> Result<(), CpuError> {
        let mut cpu = setup_cpu_for_abs_store(ABS_ADDR);
        cpu.memory.add_readonly(0xC000..0xFFFF)?;
        cpu.accumulator = 0x42;

        assert_eq!(cpu.memory.read(ABS_ADDR)?, 0x00);
        execute_sta(AddressingMode::Absolute, &mut cpu)?;
        // Store operation succeeded, but value was not written to readonly memory:
        assert_eq!(cpu.memory.read(ABS_ADDR)?, 0x00);
        Ok(())
    }
}
