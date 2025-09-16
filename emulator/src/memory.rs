use std::ops;

use crate::CpuError;

// write_zero_page_word & clear_readonly_ranges are not used yet
#[allow(dead_code)]
pub trait Memory {
    fn read(&self, address: u16) -> Result<u8, CpuError>;
    fn read_word(&self, address: u16) -> Result<u16, CpuError>;
    fn read_zero_page_word(&self, address: u8) -> Result<u16, CpuError>;
    fn write(&mut self, address: u16, value: u8) -> Result<(), CpuError>;
    fn write_word(&mut self, address: u16, value: u16) -> Result<(), CpuError>;
    fn write_zero_page_word(&mut self, address: u8, value: u16) -> Result<(), CpuError>;
    fn get_size(&self) -> usize;
    fn load_program(&mut self, start_addr: u16, program: &[u8]) -> Result<(), CpuError>;
    fn add_readonly(&mut self, range: ops::Range<u16>) -> Result<(), CpuError>;
    fn clear_readonly_ranges(&mut self);
}

#[derive(Clone)]
pub struct MemoryImpl {
    memory: Vec<u8>,
    ranges: Vec<ops::Range<u16>>,
}

impl MemoryImpl {
    pub fn new(size: usize) -> MemoryImpl {
        MemoryImpl {
            memory: vec![0; size],
            ranges: vec![],
        }
    }

    fn write_byte(
        &mut self,
        address: u16,
        value: u8,
        ignore_readonly: bool,
    ) -> Result<(), CpuError> {
        if (address as usize) >= self.memory.len() {
            return Err(CpuError::InvalidAddress);
        }
        if !ignore_readonly && self.ranges.iter().any(|r| r.contains(&address)) {
            return Err(CpuError::ReadOnlyMemory);
        }
        self.memory[address as usize] = value;
        Ok(())
    }
}

impl Default for MemoryImpl {
    fn default() -> Self {
        Self::new(64 * 1024)
    }
}

impl std::fmt::Debug for dyn Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MemoryAccess {{ size: {} }}", self.get_size(),)
    }
}

impl std::fmt::Debug for MemoryImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MemoryAccess {{ size: {} }}", self.get_size(),)
    }
}

impl Memory for MemoryImpl {
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

    fn read_zero_page_word(&self, address: u8) -> Result<u16, CpuError> {
        self.read_word(address as u16)
    }

    fn write(&mut self, address: u16, value: u8) -> Result<(), CpuError> {
        self.write_byte(address, value, false)
    }

    fn write_word(&mut self, address: u16, value: u16) -> Result<(), CpuError> {
        // little endian, so low byte is written to lower byte address:
        self.write(address, value as u8)?;
        self.write(address + 1, (value >> 8) as u8)?;
        Ok(())
    }

    fn write_zero_page_word(&mut self, address: u8, value: u16) -> Result<(), CpuError> {
        self.write_word(address as u16, value)
    }

    fn get_size(&self) -> usize {
        self.memory.len()
    }

    fn load_program(&mut self, start_addr: u16, program: &[u8]) -> Result<(), CpuError> {
        for (i, byte) in program.iter().enumerate() {
            // allow writing to readonly memory here:
            self.write_byte(start_addr + i as u16, *byte, true)?;
        }
        Ok(())
    }

    fn add_readonly(&mut self, range: ops::Range<u16>) -> Result<(), CpuError> {
        self.ranges.push(range);
        Ok(())
    }

    fn clear_readonly_ranges(&mut self) {
        self.ranges.clear();
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    #[test]
    #[should_panic(expected = "InvalidAddress")]
    fn enforce_size_limit() {
        let mut mem = MemoryImpl::new(10);
        mem.write(10, 0).unwrap();
    }

    #[test]
    fn get_size() -> Result<(), CpuError> {
        let mem = MemoryImpl::new(10);
        assert_eq!(10, mem.get_size());
        Ok(())
    }

    #[test]
    fn write_read() -> Result<(), CpuError> {
        let mut mem = MemoryImpl::new(0x2000);
        let addr = 0x1234;
        mem.write(addr, 0xBB)?;
        assert_eq!(0xBB, mem.read(addr)?);
        mem.write(addr + 1, 0xEE)?;
        assert_eq!(0xEE, mem.read(addr + 1)?);
        Ok(())
    }

    #[test]
    fn has_little_endian_read() -> Result<(), CpuError> {
        let mut mem = MemoryImpl::new(10);
        mem.write(0, 0x34)?;
        mem.write(1, 0x12)?;
        assert_eq!(0x1234, mem.read_word(0)?);
        Ok(())
    }

    #[test]
    fn has_little_endian_write() -> Result<(), CpuError> {
        let mut mem = MemoryImpl::new(10);
        mem.write_word(0, 0x1234)?;
        assert_eq!(0x34, mem.read(0)?);
        assert_eq!(0x12, mem.read(1)?);
        Ok(())
    }

    #[test]
    fn load_program() -> Result<(), CpuError> {
        let mut mem = MemoryImpl::default();
        mem.load_program(0x1000, &[0x12, 0x34, 0x56])?;
        assert_eq!(0x12, mem.read(0x1000)?);
        assert_eq!(0x34, mem.read(0x1001)?);
        assert_eq!(0x56, mem.read(0x1002)?);
        Ok(())
    }

    #[test]
    fn zero_page_read_write() -> Result<(), CpuError> {
        let mut mem = MemoryImpl::new(10);
        mem.write(0, 0xBB)?;
        assert_eq!(0xBB, mem.read(0)?);
        mem.write(9, 0xEE)?;
        assert_eq!(0xEE, mem.read(9)?);
        Ok(())
    }

    #[test]
    fn write_to_readonly_rejected() -> Result<(), CpuError> {
        let mut mem = MemoryImpl::new(0x0200);
        mem.add_readonly(0x0100..0x0200)?;

        // load_program to readonly area is still allowed:
        mem.load_program(0x0180, &[0x12, 0x34, 0x56])?;

        assert!(mem.write(0x0100, 0x55).is_err());
        assert!(mem.write(0x0180, 0xAA).is_err());
        assert!(mem.write(0x01FF, 0xDD).is_err());
        assert_eq!(0x12, mem.read(0x0180)?);

        mem.write(0x00AB, 0x55)?;
        assert_eq!(0x55, mem.read(0x00AB)?);

        mem.clear_readonly_ranges();
        mem.write(0x0180, 0xAA)?;
        assert_eq!(0xAA, mem.read(0x0180)?);
        Ok(())
    }
}
