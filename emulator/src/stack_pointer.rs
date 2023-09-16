use crate::memory_access::MemoryAccess;
use crate::CpuError;

pub trait StackPointer {
    fn get_sp(&self) -> Result<u16, CpuError>;
    fn push_byte(&mut self, mem: &mut dyn MemoryAccess, value: u8) -> Result<(), CpuError>;
    fn pop_byte(&mut self, mem: &dyn MemoryAccess) -> Result<u8, CpuError>;
    fn push_word(&mut self, mem: &mut dyn MemoryAccess, value: u16) -> Result<(), CpuError>;
    fn pop_word(&mut self, mem: &dyn MemoryAccess) -> Result<u16, CpuError>;
    fn reset(&mut self) -> Result<(), CpuError>;
}

#[derive(Clone)]
pub struct StackPointerImpl {
    sp: u8, // only 8 bits are variable, high byte is always 0x01
}

impl StackPointerImpl {
    pub fn new() -> StackPointerImpl {
        StackPointerImpl { sp: 0xFF }
    }

    fn sp_as_u16(&self) -> u16 {
        0x0100 | self.sp as u16
    }
}

#[allow(unused_variables)] // TODO remove
impl StackPointer for StackPointerImpl {
    fn get_sp(&self) -> Result<u16, CpuError> {
        Ok(0x0100 | self.sp as u16)
    }

    fn push_byte(&mut self, mem: &mut dyn MemoryAccess, value: u8) -> Result<(), CpuError> {
        if self.sp == 0x00 {
            return Err(CpuError::StackOverflow);
        }
        mem.write(self.sp_as_u16(), value)?;
        self.sp = self.sp.wrapping_sub(1);
        Ok(())
    }

    fn pop_byte(&mut self, mem: &dyn MemoryAccess) -> Result<u8, CpuError> {
        if self.sp == 0xFF {
            return Err(CpuError::StackOverflow);
        }
        self.sp = self.sp.wrapping_add(1);
        let value = mem.read(self.sp_as_u16())?;
        Ok(value)
    }

    fn push_word(&mut self, mem: &mut dyn MemoryAccess, value: u16) -> Result<(), CpuError> {
        if self.sp <= 0x01 {
            return Err(CpuError::StackOverflow);
        }
        // SP is at top of stack, but writing 2 bytes:
        mem.write_word(self.sp_as_u16() - 1, value)?;
        self.sp = self.sp.wrapping_sub(2);
        Ok(())
    }

    fn pop_word(&mut self, mem: &dyn MemoryAccess) -> Result<u16, CpuError> {
        if self.sp >= 0xFE {
            return Err(CpuError::StackOverflow);
        }
        // move to first byte on stack:
        self.sp = self.sp.wrapping_add(1);
        let value = mem.read_word(self.sp_as_u16())?;
        // pulled 2 bytes from stack, so move SP again:
        self.sp = self.sp.wrapping_add(1);
        Ok(value)
    }

    fn reset(&mut self) -> Result<(), CpuError> {
        self.sp = 0xFF;
        Ok(())
    }
}

impl std::fmt::Debug for dyn StackPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "StackPointer {{ sp: 0x01{:02X} }}",
            (self.get_sp().unwrap() & 0xFF) as u8
        )
    }
}

impl std::fmt::Debug for StackPointerImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StackPointer {{ sp: 0x{:04X} }}", self.sp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;

    mock! {
        pub _MemoryAccess {}
        impl MemoryAccess for _MemoryAccess {
            fn read(&self, address: u16) -> Result<u8, CpuError>;
            fn read_word(&self, address: u16) -> Result<u16, CpuError>;
            fn write(&mut self, address: u16, value: u8) -> Result<(), CpuError>;
            fn write_word(&mut self, address: u16, value: u16) -> Result<(), CpuError>;
            fn get_size(&self) -> usize;
            fn load_program(&mut self, start_addr: u16, program: &[u8]) -> Result<(), CpuError>;
        }
    }

    #[test]
    fn pop_from_empty_stack_error_stack_overflow() -> Result<(), CpuError> {
        let mut sp = StackPointerImpl::new();
        let mut mem = Mock_MemoryAccess::new();
        mem.expect_write().returning(|_, _| Ok(()));

        // try byte:
        assert_eq!(sp.pop_byte(&mem), Err(CpuError::StackOverflow));

        // push a single byte, then try popping a word:
        sp.push_byte(&mut mem, 42)?;
        assert_eq!(sp.pop_word(&mem), Err(CpuError::StackOverflow));

        Ok(())
    }

    #[test]
    fn push_to_full_stack_error_stack_overflow() -> Result<(), CpuError> {
        let mut sp = StackPointerImpl::new();
        let mut mem = Mock_MemoryAccess::new();
        mem.expect_read().returning(|_| Ok(42));

        // indicate stack reached bottom:
        sp.sp = 0x00;
        // try byte:
        assert_eq!(sp.push_byte(&mut mem, 0x12), Err(CpuError::StackOverflow));

        // pop a single byte, then try pushing a word:
        sp.pop_byte(&mem)?;
        assert_eq!(sp.push_word(&mut mem, 0x1234), Err(CpuError::StackOverflow));

        Ok(())
    }

    #[test]
    fn pop_stack_overflow() {
        let mut sp = StackPointerImpl::new();
        let mut mem = Mock_MemoryAccess::new();
        mem.expect_read().returning(|_| Ok(0));
        mem.expect_read_word().returning(|_| Ok(0));
        mem.expect_write().returning(|_, _| Ok(()));
        mem.expect_write_word().returning(|_, _| Ok(()));

        assert_eq!(0x01FF, sp.get_sp().unwrap());
        assert!(sp.pop_byte(&mem).is_err());
        assert_eq!(0x01FF, sp.get_sp().unwrap());
    }

    #[test]
    fn get_sp() -> Result<(), CpuError> {
        let sp = StackPointerImpl::new();
        assert_eq!(0x01FF, sp.get_sp()?);
        Ok(())
    }

    #[test]
    fn reset() -> Result<(), CpuError> {
        let mut sp = StackPointerImpl::new();
        sp.sp = 0xFF;
        sp.reset()?;
        assert_eq!(0x01FF, sp.get_sp()?);
        Ok(())
    }

    #[test]
    fn push_byte() -> Result<(), CpuError> {
        let mut sp = StackPointerImpl::new();

        let mut mem = Mock_MemoryAccess::new();
        mem.expect_write()
            .with(eq(0x01FF), eq(0x12))
            .returning(|_, _| Ok(()));
        mem.expect_write()
            .with(eq(0x01FE), eq(0x34))
            .returning(|_, _| Ok(()));

        sp.push_byte(&mut mem, 0x12)?;
        assert_eq!(0x01FF - 1, sp.get_sp()?);
        sp.push_byte(&mut mem, 0x34)?;
        assert_eq!(0x01FF - 2, sp.get_sp()?);
        Ok(())
    }

    #[test]
    fn push_word() -> Result<(), CpuError> {
        let mut sp = StackPointerImpl::new();

        let mut mem = Mock_MemoryAccess::new();
        mem.expect_write_word()
            .with(eq(0x01FE), eq(0x1234))
            .returning(|_, _| Ok(()));

        sp.push_word(&mut mem, 0x1234)?;
        assert_eq!(0x01FF - 2, sp.get_sp()?);
        Ok(())
    }

    #[test]
    fn pop_byte() -> Result<(), CpuError> {
        let mut sp = StackPointerImpl::new();

        let mut mem = Mock_MemoryAccess::new();
        mem.expect_read().with(eq(0x01FF)).returning(|_| Ok(0x12));
        mem.expect_read().with(eq(0x01FE)).returning(|_| Ok(0x34));

        // popping 2 separate bytes:
        sp.sp = 0xFD;
        assert_eq!(0x34, sp.pop_byte(&mem)?);
        assert_eq!(0x01FE, sp.get_sp()?);
        assert_eq!(0x12, sp.pop_byte(&mem)?);
        assert_eq!(0x01FF, sp.get_sp()?);
        Ok(())
    }

    #[test]
    fn pop_word() -> Result<(), CpuError> {
        let mut sp = StackPointerImpl::new();

        let mut mem = Mock_MemoryAccess::new();
        mem.expect_read_word()
            .with(eq(0x01FE))
            .returning(|_| Ok(0x1234));

        // popping a word:
        sp.sp = 0xFD;
        assert_eq!(0x1234, sp.pop_word(&mem)?);
        assert_eq!(0x01FF, sp.get_sp()?);
        Ok(())
    }
}
