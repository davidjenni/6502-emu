use mos6502_emulator::{create, CpuError, CpuType};

#[test]
#[should_panic(expected = "not yet implemented")]
fn create_default_cpu() {
    let mut cpu = create(CpuType::MOS6502).unwrap();
    assert!(cpu.reset().is_ok());
}

#[test]
fn run_simple_program() -> Result<(), CpuError> {
    let mut cpu = create(CpuType::MOS6502).unwrap();
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
    assert_eq!(snapshot.program_counter, 0x0605);
    // read back transferred byte from zero page:
    assert_eq!(cpu.get_byte_at(0x000F)?, 0x42);
    Ok(())
}
