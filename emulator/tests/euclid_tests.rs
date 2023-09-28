// See algo and assembly code in euclid_gcd.md

use mos6502_emulator::{create_cpu, CpuError, CpuType};

#[test]
fn run_gcd_euclid() -> Result<(), CpuError> {
    let mut cpu = create_cpu(CpuType::MOS6502).unwrap();
    cpu.load_program(
        0x0200,
        &[
            0xA5, 0x40, // LDA VAR_A
            0x38, // SEC
            0xE5, 0x41, // SBC VAR_B
            0xF0, 0x12, // BEQ done
            0x30, 0x05, // BMI swap
            0x85, 0x40, // STA VAR_A
            0x4C, 0x02, 0x02, // JMP diff
            0xA6, 0x40, // LDX VAR_A
            0xA4, 0x41, // LDY VAR_B
            0x86, 0x41, // STX VAR_B
            0x84, 0x40, // STY VAR_A
            0x4C, 0x00, 0x02, // JMP start
            0xA5, 0x40, // LDA VAR_A
            0x00, // BRK
        ],
    )?;
    // initialize zero page variables:
    cpu.set_byte_at(0x0040, 126)?; // VAR_A
    cpu.set_byte_at(0x0041, 49)?; // VAR_B

    let snapshot = cpu.run(Some(0x0200))?;
    assert_eq!(snapshot.program_counter, 0xFFFE);
    // assert loop termination due to a == b:
    assert_eq!(cpu.get_byte_at(0x0040)?, cpu.get_byte_at(0x0041)?);
    // read back transferred byte from zero page:
    assert_eq!(cpu.get_byte_at(0x0040)?, 7);
    assert_eq!(snapshot.accumulated_instructions, 72);
    assert_eq!(snapshot.accumulated_cycles, 191);
    // clock speed is returned in Hz, so even a slow machine should be faster than 1kHz:
    assert!(snapshot.approximate_clock_speed > 1000.0);
    println!(
        "approximate clock speed: {} MHz",
        snapshot.approximate_clock_speed / 1_000_000.0
    );
    Ok(())
}
