#[derive(Debug)]
pub enum AddressError {
    InvalidAddress,
}

pub trait Memory {
    fn read(&self, address: u16) -> Result<u8, AddressError>;
    fn write(&mut self, address: u16, value: u8) -> Result<(), AddressError>;
    fn get_size(&self) -> usize;
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub struct MockMemory {
        memory: [u8; 256],
    }

    impl MockMemory {
        pub fn new() -> MockMemory {
            MockMemory { memory: [0; 256] }
        }
    }

    impl Default for MockMemory {
        fn default() -> Self {
            Self::new()
        }
    }

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
    }
}
