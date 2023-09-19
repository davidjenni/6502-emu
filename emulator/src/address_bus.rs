use crate::memory::Memory;
use crate::CpuError;

#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub enum SystemVector {
    NMI = 0xFFFA,
    Reset = 0xFFFC,
    IRQ = 0xFFFE,
}

pub trait AddressBus {
    fn fetch_byte_at_pc(&mut self, mem: &mut dyn Memory) -> Result<u8, CpuError>;
    fn fetch_word_at_pc(&mut self, mem: &mut dyn Memory) -> Result<u16, CpuError>;
    fn set_pc(&mut self, address: u16) -> Result<(), CpuError>;
    fn get_pc(&self) -> u16;
}

#[derive(Clone)]
pub struct AddressBusImpl {
    pc: u16,
}

impl AddressBusImpl {
    pub fn new() -> AddressBusImpl {
        AddressBusImpl { pc: 0 }
    }
}

impl AddressBus for AddressBusImpl {
    fn fetch_byte_at_pc(&mut self, mem: &mut dyn Memory) -> Result<u8, CpuError> {
        if mem.get_size() <= self.pc as usize {
            return Err(CpuError::InvalidAddress);
        }
        let op = mem.read(self.pc)?;
        self.pc += 1;
        Ok(op)
    }

    fn fetch_word_at_pc(&mut self, mem: &mut dyn Memory) -> Result<u16, CpuError> {
        // little endian, so low byte is read first:
        let lo = self.fetch_byte_at_pc(mem)? as u16;
        let hi = self.fetch_byte_at_pc(mem)? as u16;
        Ok((hi << 8) | lo)
    }

    fn set_pc(&mut self, address: u16) -> Result<(), CpuError> {
        self.pc = address;
        Ok(())
    }

    fn get_pc(&self) -> u16 {
        self.pc
    }
}
impl std::fmt::Debug for dyn AddressBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AddressBus {{ pc: 0x{:04X} }}", self.get_pc(),)
    }
}

impl std::fmt::Debug for AddressBusImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AddressBus {{ pc: 0x{:04X} }}", self.pc,)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;

    mock! {
        pub _Memory {}
        impl Memory for _Memory {
            fn read(&self, address: u16) -> Result<u8, CpuError>;
            fn read_word(&self, address: u16) -> Result<u16, CpuError>;
            fn read_zero_page_word(&self, address: u8) -> Result<u16, CpuError>;
            fn write(&mut self, address: u16, value: u8) -> Result<(), CpuError>;
            fn write_word(&mut self, address: u16, value: u16) -> Result<(), CpuError>;
            fn get_size(&self) -> usize;
            fn load_program(&mut self, start_addr: u16, program: &[u8]) -> Result<(), CpuError>;
            fn write_zero_page_word(&mut self, address: u8, value: u16) -> Result<(), CpuError>;
        }
    }
    #[test]
    fn can_fetch_next_op() -> Result<(), CpuError> {
        let mut bus = AddressBusImpl::new();
        let mut mem = Mock_Memory::new();
        mem.expect_read().returning(|_| Ok(42));
        mem.expect_get_size().returning(|| 0x80);

        assert_eq!(bus.fetch_byte_at_pc(&mut mem)?, 42);
        assert_eq!(bus.get_pc(), 0x0001);
        Ok(())
    }

    #[test]
    fn has_debug_fmt() {
        let bus = AddressBusImpl::new();

        let debug_msg = format!("{:?}", bus);
        assert_eq!(debug_msg, "AddressBus { pc: 0x0000 }");
    }

    #[test]
    fn fetch_next_op_panics_on_invalid_address() -> Result<(), CpuError> {
        let mut bus = AddressBusImpl::new();
        let mut mem = Mock_Memory::new();
        mem.expect_read().returning(|_| Ok(42));
        mem.expect_get_size().returning(|| 0x80);

        let top_address = mem.get_size() as u16;
        bus.set_pc(top_address - 1).unwrap();
        // first fetch will succeed, but moves pc to 0x0100
        bus.fetch_byte_at_pc(&mut mem).unwrap();
        assert_eq!(bus.get_pc(), top_address);

        // this fetch should panic
        assert_eq!(
            bus.fetch_byte_at_pc(&mut mem),
            Err(CpuError::InvalidAddress)
        );
        Ok(())
    }
}
