use mos6502_emulator::{create_cpu, CpuError, CpuType};

#[test]
fn create_default_cpu() -> Result<(), CpuError> {
    let mut cpu = create_cpu(CpuType::MOS6502)?;
    cpu.reset()?;
    Ok(())
}

#[test]
fn run_simple_program() -> Result<(), CpuError> {
    let mut cpu = create_cpu(CpuType::MOS6502).unwrap();
    assert!(cpu
        .load_program(
            0x0600,
            &[
                0xA9, 0x42, // LDA #$42
                0x85, 0x0F, // STA $0F
                0x00, 0x00, // BRK
            ]
        )
        .is_ok());
    let snapshot = cpu.run(Some(0x0600))?;
    assert_eq!(snapshot.accumulator, 0x42);
    // assert_eq!(snapshot.program_counter, 0x0605);
    // read back transferred byte from zero page:
    assert_eq!(cpu.get_byte_at(0x000F)?, 0x42);
    Ok(())
}

#[test]
fn run_empty_program_stops_at_irq() -> Result<(), CpuError> {
    let mut cpu = create_cpu(CpuType::MOS6502).unwrap();
    let reg_snapshot = cpu.run(None)?;
    // PC should point at IRQ vector:
    assert_eq!(reg_snapshot.program_counter, 0xFFFE);
    // executed a single BRK since all memory is zeroed out:
    assert_eq!(reg_snapshot.accumulated_instructions, 1);
    // BRK takes 7 cycles:
    assert_eq!(reg_snapshot.accumulated_cycles, 7);
    Ok(())
}
