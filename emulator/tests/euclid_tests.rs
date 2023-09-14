// calculate greatest commom denominator using Euclid's integer subtraction algorithm:
// https://en.wikipedia.org/wiki/Euclidean_algorithm#Implementations
// function gcd(a, b)
//   while a != b
//     if a > b
//        a := a - b
//     else
//        b := b - a
//   return a
//
// 6502 assembly:
// ; zero page addresses:
// VAR_A = $40
// VAR_B = $41
//
// .org $0600
// ; start:
// 0600 A5 40      LDA VAR_A
// ; diff:
// 0602 38         SEC
// 0603 E5 41      SBC VAR_B
// 0605 F0 12      BEQ done
// 0607 30 05      BMI swap
// 0609 85 40      STA VAR_A
// 060B 4C 02 06   JMP diff
// ; swap
// 060E A6 40      LDX VAR_A
// 0610 A4 41      LDY VAR_B
// 0612 86 41      STX VAR_B
// 0614 84 40      STY VAR_A
// 0616 4C 00 06   JMP start
// ; done:
// 0619 A5 40      LDA VAR_A
// 061B 00         BRK

use mos6502_emulator::{create, CpuError, CpuType};

#[test]
fn run_gcd_euclid() -> Result<(), CpuError> {
    let mut cpu = create(CpuType::MOS6502).unwrap();
    cpu.load_program(
        0x0600,
        &[
            0xA5, 0x40, // LDA VAR_A
            0x38, // SEC
            0xE5, 0x41, // SBC VAR_B
            0xF0, 0x12, // BEQ done
            0x30, 0x05, // BMI swap
            0x85, 0x40, // STA VAR_A
            0x4C, 0x02, 0x06, // JMP diff
            0xA6, 0x40, // LDX VAR_A
            0xA4, 0x41, // LDY VAR_B
            0x86, 0x41, // STX VAR_B
            0x84, 0x40, // STY VAR_A
            0x4C, 0x00, 0x06, // JMP start
            0xA5, 0x40, // LDA VAR_A
            0x00, // BRK
        ],
    )?;
    // initialize zero page variables:
    cpu.set_byte_at(0x0040, 96)?; // VAR_A
    cpu.set_byte_at(0x0041, 56)?; // VAR_B

    let snapshot = cpu.run(Some(0x0600))?;
    assert_eq!(snapshot.program_counter, 0x061C);
    // assert loop termination due to a == b:
    assert_eq!(cpu.get_byte_at(0x0040)?, cpu.get_byte_at(0x0041)?);
    // read back transferred byte from zero page:
    assert_eq!(cpu.get_byte_at(0x0040)?, 8);
    Ok(())
}
