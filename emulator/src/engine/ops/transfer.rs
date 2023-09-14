use crate::cpu::{AddressingMode, Cpu};
use crate::CpuError;

// LDA/X/Y:

// LDA:    M -> A
// status: N. ...Z.
pub fn execute_lda(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let value = cpu.get_effective_operand(mode)?;
    cpu.accumulator = value;
    cpu.status.update_from(value);
    Ok(())
}

// LDX:    M -> X
// status: N. ...Z.
pub fn execute_ldx(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let value = cpu.get_effective_operand(mode)?;
    cpu.index_x = value;
    cpu.status.update_from(value);
    Ok(())
}

// LDY:    M -> Y
// status: N. ...Z.
pub fn execute_ldy(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let value = cpu.get_effective_operand(mode)?;
    cpu.index_y = value;
    cpu.status.update_from(value);
    Ok(())
}

// STA/X/Y:

// STA:    A -> M
// status: N. ...Z.
pub fn execute_sta(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    let val = cpu.accumulator;
    cpu.address_bus.write(effective_address, val).unwrap();
    cpu.status.update_from(val);
    Ok(())
}

// STX:    X -> M
// status: N. ...Z.
pub fn execute_stx(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    let val = cpu.index_x;
    cpu.address_bus.write(effective_address, val)?;
    cpu.status.update_from(val);
    Ok(())
}

// STY:    Y -> M
// status: N. ...Z.
pub fn execute_sty(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    let val = cpu.index_y;
    cpu.address_bus.write(effective_address, val)?;
    cpu.status.update_from(val);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RamMemory;

    const ZERO_PAGE_ADDR: u16 = 0x00E0;
    const NEXT_PC: u16 = 0x0300;

    // create a Cpu instance and write test value value into zero page address $E0
    // writes zero page address $E0 into next PC address $0300
    fn setup_cpu_for_load(value: u8) -> Cpu {
        let mut cpu = create_cpu();
        cpu.address_bus.write(ZERO_PAGE_ADDR, value).unwrap();

        cpu
    }

    fn setup_cpu_for_store() -> Cpu {
        create_cpu()
    }

    fn create_cpu() -> Cpu {
        let mut cpu = Cpu::new(Box::default() as Box<RamMemory>);
        cpu.address_bus
            .write(NEXT_PC, ZERO_PAGE_ADDR as u8)
            .unwrap();
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
        cpu.address_bus.write(ZERO_PAGE_ADDR, 0).unwrap();
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
        cpu.address_bus.write(0x00E0, 0).unwrap();
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
        assert_eq!(cpu.address_bus.read(ZERO_PAGE_ADDR).unwrap(), 0x42);
        assert!(!cpu.status.zero());
        assert!(!cpu.status.negative());

        // is negative flag updated?
        let test_val = (-123 & 0xFF) as u8;
        cpu.accumulator = test_val;
        cpu.address_bus.set_pc(NEXT_PC).unwrap();
        execute_sta(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.address_bus.read(ZERO_PAGE_ADDR).unwrap(), test_val);
        assert!(!cpu.status.zero());
        assert!(cpu.status.negative());
    }

    #[test]
    fn test_stx() {
        let mut cpu = setup_cpu_for_store();
        cpu.index_x = 0x42;
        execute_stx(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.address_bus.read(ZERO_PAGE_ADDR).unwrap(), 0x42);
        assert!(!cpu.status.zero());
        assert!(!cpu.status.negative());

        // is negative flag updated?
        let test_val = (-123 & 0xFF) as u8;
        cpu.index_x = test_val;
        cpu.address_bus.set_pc(NEXT_PC).unwrap();
        execute_stx(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.address_bus.read(ZERO_PAGE_ADDR).unwrap(), test_val);
        assert!(!cpu.status.zero());
        assert!(cpu.status.negative());
    }

    #[test]
    fn test_sty() {
        let mut cpu = setup_cpu_for_store();
        cpu.index_y = 0x42;
        execute_sty(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.address_bus.read(ZERO_PAGE_ADDR).unwrap(), 0x42);
        assert!(!cpu.status.zero());
        assert!(!cpu.status.negative());

        // is negative flag updated?
        let test_val = (-123 & 0xFF) as u8;
        cpu.index_y = test_val;
        cpu.address_bus.set_pc(NEXT_PC).unwrap();
        execute_sty(AddressingMode::ZeroPage, &mut cpu).unwrap();
        assert_eq!(cpu.address_bus.read(ZERO_PAGE_ADDR).unwrap(), test_val);
        assert!(!cpu.status.zero());
        assert!(cpu.status.negative());
    }
}
