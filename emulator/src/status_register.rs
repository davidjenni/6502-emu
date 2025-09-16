use bitfield_struct::bitfield;
use std::fmt;

#[bitfield(u8, order = msb)]
pub struct StatusRegister {
    pub negative: bool,          // N, bit 7
    pub overflow: bool,          // V, bit 6
    unused_bit: bool,            // n/a, bit 5 is an unused expansion bit
    pub break_command: bool,     // B, bit 4
    pub decimal_mode: bool,      // D, bit 3
    pub interrupt_disable: bool, // I, bit 2
    pub zero: bool,              // Z, bit 1
    pub carry: bool,             // C, bit 0
}

impl StatusRegister {
    // update status bits from a register value
    pub fn update_from(&mut self, value: u8) {
        const SIGN_BIT: u8 = 0b1000_0000;
        self.set_negative(value & SIGN_BIT == SIGN_BIT);
        self.set_zero(value == 0);
    }

    pub fn set_break(&mut self, value: bool) {
        self.set_break_command(value);
    }

    pub fn get_break(&self) -> bool {
        self.break_command()
    }

    pub fn get_status(&self) -> u8 {
        self.0
    }

    pub fn set_status(&mut self, value: u8) {
        self.0 = value;
    }
}

impl fmt::Display for StatusRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "0x{:02X}:  N={:b}, V={:b}, B={:b}, D={:b}, I={:b}, Z={:b}, C={:b}",
            self.0,
            self.negative() as u8,
            self.overflow() as u8,
            self.break_command() as u8,
            self.decimal_mode() as u8,
            self.interrupt_disable() as u8,
            self.zero() as u8,
            self.carry() as u8
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_status_register_is_zero() {
        let sr = StatusRegister::default();
        assert_eq!(sr.0, 0);
    }

    #[test]
    fn can_set_and_clear_carry() {
        let mut sr = StatusRegister::default();
        assert!(!sr.carry());
        sr.set_carry(true);
        assert!(sr.carry());
        sr.set_carry(false);
        assert!(!sr.carry());
        sr.set_carry(true);
    }

    #[test]
    fn can_print_status_register() {
        let mut sr = StatusRegister::default();
        sr.set_negative(true);
        sr.set_overflow(true);
        sr.set_break_command(false);
        sr.set_decimal_mode(false);
        sr.set_interrupt_disable(true);
        sr.set_zero(false);
        sr.set_carry(false);

        let buffer = format!("{}", sr);
        assert_eq!(buffer, "0xC4:  N=1, V=1, B=0, D=0, I=1, Z=0, C=0");
    }

    #[test]
    fn update_status_register_from_register_value() {
        let mut sr = StatusRegister::default();

        // test with 3 different values: zero, negative, positive
        sr.update_from(0);
        assert!(!sr.negative());
        assert!(sr.zero());
        let b = format!("{}", sr);
        assert_eq!(b, "0x02:  N=0, V=0, B=0, D=0, I=0, Z=1, C=0");

        sr.update_from(-42i8 as u8);
        assert!(sr.negative());
        assert!(!sr.zero());
        let b = format!("{}", sr);
        assert_eq!(b, "0x80:  N=1, V=0, B=0, D=0, I=0, Z=0, C=0");

        sr.update_from(42);
        assert!(!sr.negative());
        assert!(!sr.zero());
        let b = format!("{}", sr);
        assert_eq!(b, "0x00:  N=0, V=0, B=0, D=0, I=0, Z=0, C=0");
    }

    #[test]
    fn can_set_and_clear_break_command() {
        let mut sr = StatusRegister::default();
        assert!(!sr.break_command());
        sr.set_break_command(true);
        assert!(sr.break_command());
        sr.set_break_command(false);
        assert!(!sr.break_command());
        sr.set_break_command(true);
    }
}
