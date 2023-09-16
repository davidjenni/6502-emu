use crate::CpuError;

#[allow(unused_variables)] // TODO remove

// #[cfg_attr(test, automock)]
pub trait MemoryAccess {
    fn read(&self, address: u16) -> Result<u8, CpuError>;
    fn read_word(&self, address: u16) -> Result<u16, CpuError>;
    fn write(&mut self, address: u16, value: u8) -> Result<(), CpuError>;
    fn write_word(&mut self, address: u16, value: u16) -> Result<(), CpuError>;
    fn get_size(&self) -> usize;
    fn load_program(&mut self, start_addr: u16, program: &[u8]) -> Result<(), CpuError>;
}

#[derive(Debug, Clone)]
pub struct Memory2 {
    memory: Vec<u8>,
}

impl Memory2 {
    pub fn new(size: usize) -> Memory2 {
        Memory2 {
            memory: vec![0; size],
        }
    }
}

impl Default for Memory2 {
    fn default() -> Self {
        Self::new(64 * 1024)
    }
}

impl MemoryAccess for Memory2 {
    fn read(&self, address: u16) -> Result<u8, CpuError> {
        if (address as usize) >= self.memory.len() {
            return Err(CpuError::InvalidAddress);
        }
        Ok(self.memory[address as usize])
    }

    fn read_word(&self, address: u16) -> Result<u16, CpuError> {
        // little endian, so low byte is read first:
        let lo = self.read(address)? as u16;
        let hi = self.read(address + 1)? as u16;
        Ok((hi << 8) | lo)
    }

    fn write(&mut self, address: u16, value: u8) -> Result<(), CpuError> {
        if (address as usize) >= self.memory.len() {
            return Err(CpuError::InvalidAddress);
        }
        self.memory[address as usize] = value;
        Ok(())
    }

    fn write_word(&mut self, address: u16, value: u16) -> Result<(), CpuError> {
        // little endian, so low byte is written to lower byte address:
        self.write(address, value as u8)?;
        self.write(address + 1, (value >> 8) as u8)?;
        Ok(())
    }

    fn get_size(&self) -> usize {
        self.memory.len()
    }

    fn load_program(&mut self, start_addr: u16, program: &[u8]) -> Result<(), CpuError> {
        for (i, byte) in program.iter().enumerate() {
            self.write(start_addr + i as u16, *byte)?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    #[test]
    #[should_panic(expected = "InvalidAddress")]
    fn enforce_size_limit() {
        let mut mem = Memory2::new(10);
        mem.write(10, 0).unwrap();
    }

    #[test]
    fn get_size() -> Result<(), CpuError> {
        let mem = Memory2::new(10);
        assert_eq!(10, mem.get_size());
        Ok(())
    }

    #[test]
    fn write_read() -> Result<(), CpuError> {
        let mut mem = Memory2::new(10);
        mem.write(0, 0xBB)?;
        assert_eq!(0xBB, mem.read(0)?);
        mem.write(9, 0xEE)?;
        assert_eq!(0xEE, mem.read(9)?);
        Ok(())
    }

    #[test]
    fn has_little_endian_read() -> Result<(), CpuError> {
        let mut mem = Memory2::new(10);
        mem.write(0, 0x34)?;
        mem.write(1, 0x12)?;
        assert_eq!(0x1234, mem.read_word(0)?);
        Ok(())
    }

    #[test]
    fn has_little_endian_write() -> Result<(), CpuError> {
        let mut mem = Memory2::new(10);
        mem.write_word(0, 0x1234)?;
        assert_eq!(0x34, mem.read(0)?);
        assert_eq!(0x12, mem.read(1)?);
        Ok(())
    }

    #[test]
    fn load_program() -> Result<(), CpuError> {
        let mut mem = Memory2::default();
        mem.load_program(0x1000, &[0x12, 0x34, 0x56])?;
        assert_eq!(0x12, mem.read(0x1000)?);
        assert_eq!(0x34, mem.read(0x1001)?);
        assert_eq!(0x56, mem.read(0x1002)?);
        Ok(())
    }
}
