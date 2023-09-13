use crate::CpuError;

pub trait Memory {
    fn read(&self, address: u16) -> Result<u8, CpuError>;
    fn write(&mut self, address: u16, value: u8) -> Result<(), CpuError>;
    fn get_size(&self) -> usize;
    fn load_program(&mut self, start_addr: u16, program: &[u8]) -> Result<(), CpuError>;
}

#[derive(Debug, Clone)]
pub struct RamMemory {
    memory: Vec<u8>,
}

impl RamMemory {
    pub fn new(size: usize) -> RamMemory {
        RamMemory {
            memory: vec![0; size],
        }
    }
}

impl Default for RamMemory {
    fn default() -> Self {
        Self::new(64 * 1024)
    }
}

impl Memory for RamMemory {
    fn read(&self, address: u16) -> Result<u8, CpuError> {
        if (address as usize) >= self.memory.len() {
            return Err(CpuError::InvalidAddress);
        }
        Ok(self.memory[address as usize])
    }

    fn write(&mut self, address: u16, value: u8) -> Result<(), CpuError> {
        if (address as usize) >= self.memory.len() {
            return Err(CpuError::InvalidAddress);
        }
        self.memory[address as usize] = value;
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
        let mut mem = RamMemory::new(10);
        mem.write(10, 0).unwrap();
    }
}
