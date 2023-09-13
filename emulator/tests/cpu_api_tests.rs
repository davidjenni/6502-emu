use mos6502_emulator::{create, CpuType};

#[test]
#[should_panic(expected = "not yet implemented")]
fn create_default_cpu() {
    let mut cpu = create(CpuType::MOS6502).unwrap();
    assert!(cpu.reset().is_ok());
}
