#[derive(Debug)]
pub enum AddressError {
    InvalidAddress,
    InvalidAddressingMode,
}

pub trait Memory {
    fn read(&self, address: u16) -> Result<u8, AddressError>;
    fn write(&mut self, address: u16, value: u8) -> Result<(), AddressError>;
    fn get_size(&self) -> usize;
    fn load_program(&mut self, start_addr: u16, program: &[u8]) -> Result<(), AddressError>;
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy)]
    pub struct MockMemory {
        memory: [u8; 4 * 256],
    }

    impl MockMemory {
        pub fn new() -> MockMemory {
            MockMemory {
                memory: [0; 4 * 256],
            }
        }
    }

    impl Default for MockMemory {
        fn default() -> Self {
            Self::new()
        }
    }

    // TODO: the "real" computer memory will look a lot like this mock -> refactor needed
    impl Memory for MockMemory {
        fn read(&self, address: u16) -> Result<u8, AddressError> {
            if (address as usize) >= self.memory.len() {
                return Err(AddressError::InvalidAddress);
            }
            Ok(self.memory[address as usize])
        }

        fn write(&mut self, address: u16, value: u8) -> Result<(), AddressError> {
            if (address as usize) >= self.memory.len() {
                return Err(AddressError::InvalidAddress);
            }
            self.memory[address as usize] = value;
            Ok(())
        }

        fn get_size(&self) -> usize {
            self.memory.len()
        }

        fn load_program(&mut self, start_addr: u16, program: &[u8]) -> Result<(), AddressError> {
            for (i, byte) in program.iter().enumerate() {
                self.write(start_addr + i as u16, *byte)?;
            }
            Ok(())
        }
    }
}
