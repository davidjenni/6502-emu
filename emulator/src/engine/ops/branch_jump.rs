use crate::cpu::{AddressingMode, Cpu};
use crate::CpuError;

// Branch operations:

// BCC:    Branch on Carry clear (C = 0)
// status: n/c
#[allow(dead_code)] // TODO remove
pub fn execute_bcc(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    if !cpu.status.carry() {
        cpu.address_bus.set_pc(effective_address)?;
    }
    Ok(())
}

// BCS:    Branch on Carry set (C = 1)
// status: n/c
#[allow(dead_code)] // TODO remove
pub fn execute_bcs(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    if cpu.status.carry() {
        cpu.address_bus.set_pc(effective_address)?;
    }
    Ok(())
}

// BEQ:    Branch on result zero (Z = 1)
// status: n/c
pub fn execute_beq(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    if cpu.status.zero() {
        cpu.address_bus.set_pc(effective_address)?;
    }
    Ok(())
}

// BMI:    Branch on result minus (N = 1)
// status: n/c
pub fn execute_bmi(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    if cpu.status.negative() {
        cpu.address_bus.set_pc(effective_address)?;
    }
    Ok(())
}

// BNE:    Branch on result non zero (Z = 0)
// status: n/c
#[allow(dead_code)] // TODO remove
pub fn execute_bne(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    if !cpu.status.zero() {
        cpu.address_bus.set_pc(effective_address)?;
    }
    Ok(())
}

// BPL:    Branch on result plus (N = 0)
// status: n/c
#[allow(dead_code)] // TODO remove
pub fn execute_bpl(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    if !cpu.status.negative() {
        cpu.address_bus.set_pc(effective_address)?;
    }
    Ok(())
}

// BVC:    Branch on Overflow clear (V = 1)
// status: n/c
#[allow(dead_code)] // TODO remove
pub fn execute_bvc(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    if !cpu.status.overflow() {
        cpu.address_bus.set_pc(effective_address)?;
    }
    Ok(())
}

// BVS:    Branch on Overflow set (V = 1)
// status: n/c
#[allow(dead_code)] // TODO remove
pub fn execute_bvs(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    if cpu.status.overflow() {
        cpu.address_bus.set_pc(effective_address)?;
    }
    Ok(())
}

// Jump operations:

// JMP:    Jump to new location
// (PC + 1) -> PCL
// (PC + 2) -> PCH
// status: n/c
pub fn execute_jmp(mode: AddressingMode, cpu: &mut Cpu) -> Result<(), CpuError> {
    let effective_address = cpu.get_effective_address(mode)?;
    cpu.address_bus.set_pc(effective_address)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{status_register::StatusRegister, RamMemory};

    #[test]
    fn bcc() -> Result<(), CpuError> {
        assert_branching(execute_bcc, |cpu, should_branch_status| {
            cpu.set_carry(!should_branch_status)
        })
    }

    #[test]
    fn bcs() -> Result<(), CpuError> {
        assert_branching(execute_bcs, |cpu, should_branch_status| {
            cpu.set_carry(should_branch_status)
        })
    }

    #[test]
    fn beq() -> Result<(), CpuError> {
        assert_branching(execute_beq, |cpu, should_branch_status| {
            cpu.set_zero(should_branch_status)
        })
    }

    #[test]
    fn bmi() -> Result<(), CpuError> {
        assert_branching(execute_bmi, |cpu, should_branch_status| {
            cpu.set_negative(should_branch_status)
        })
    }

    #[test]
    fn bne() -> Result<(), CpuError> {
        assert_branching(execute_bne, |cpu, should_branch_status| {
            cpu.set_zero(!should_branch_status)
        })
    }

    #[test]
    fn bpl() -> Result<(), CpuError> {
        assert_branching(execute_bpl, |cpu, should_branch_status| {
            cpu.set_negative(!should_branch_status)
        })
    }

    #[test]
    fn bvc() -> Result<(), CpuError> {
        assert_branching(execute_bvc, |cpu, should_branch_status| {
            cpu.set_overflow(!should_branch_status)
        })
    }

    #[test]
    fn bvs() -> Result<(), CpuError> {
        assert_branching(execute_bvs, |cpu, should_branch_status| {
            cpu.set_overflow(should_branch_status)
        })
    }

    #[test]
    fn jmp() -> Result<(), CpuError> {
        let mut cpu = create_cpu_jump(0x5432)?;
        execute_jmp(AddressingMode::Absolute, &mut cpu)?;
        assert_eq!(cpu.address_bus.get_pc(), 0x5432);

        // indirect jump:
        let mut cpu = create_cpu_jump(0x5432)?;
        cpu.address_bus.write_word(0x5432, 0x3344)?;
        execute_jmp(AddressingMode::Indirect, &mut cpu)?;
        assert_eq!(cpu.address_bus.get_pc(), 0x3344);
        Ok(())
    }

    const NEXT_PC: u16 = 0x0123;

    fn create_cpu_branch_test(relative_offset: u8) -> Result<Cpu, CpuError> {
        let mut cpu = Cpu::new(Box::default() as Box<RamMemory>);
        cpu.address_bus.write(NEXT_PC, relative_offset).unwrap();
        cpu.address_bus.set_pc(NEXT_PC).unwrap();
        Ok(cpu)
    }

    fn create_cpu_jump(address: u16) -> Result<Cpu, CpuError> {
        let mut cpu = Cpu::new(Box::default() as Box<RamMemory>);
        // JMP has 2 byte operand, moving PC 1 byte further:
        cpu.address_bus.write_word(NEXT_PC, address)?;
        cpu.address_bus.set_pc(NEXT_PC)?;
        Ok(cpu)
    }

    fn assert_branching(
        exec: fn(AddressingMode, &mut Cpu) -> Result<(), CpuError>,
        set_flag: fn(&mut StatusRegister, bool),
    ) -> Result<(), CpuError> {
        const POS_OFFSET: u8 = 0x12;
        const NEG_OFFSET: u8 = (-42i8) as u8;
        let expected_pos_pc = NEXT_PC + POS_OFFSET as u16 + 1;
        let expected_neg_pc = NEXT_PC.wrapping_add(NEG_OFFSET as i8 as u16) + 1;

        // positive branch offset:
        // no branch:
        let mut cpu = create_cpu_branch_test(POS_OFFSET)?;
        set_flag(&mut cpu.status, false);
        exec(AddressingMode::Relative, &mut cpu)?;
        assert_eq!(cpu.address_bus.get_pc(), NEXT_PC + 1);

        // with branch:
        let mut cpu = create_cpu_branch_test(POS_OFFSET)?;
        set_flag(&mut cpu.status, true);
        exec(AddressingMode::Relative, &mut cpu)?;
        assert_eq!(cpu.address_bus.get_pc(), expected_pos_pc);

        // negative branch offset:
        // no branch:
        let mut cpu = create_cpu_branch_test(NEG_OFFSET)?;
        set_flag(&mut cpu.status, false);
        exec(AddressingMode::Relative, &mut cpu)?;
        assert_eq!(cpu.address_bus.get_pc(), NEXT_PC + 1);

        // with branch:
        let mut cpu = create_cpu_branch_test(NEG_OFFSET)?;
        set_flag(&mut cpu.status, true);
        exec(AddressingMode::Relative, &mut cpu)?;
        assert_eq!(cpu.address_bus.get_pc(), expected_neg_pc);
        Ok(())
    }
}
