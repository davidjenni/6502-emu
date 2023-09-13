use crate::memory::{AddressError, Memory};

pub struct AddressBus {
    memory: Box<dyn Memory>,
    pc: u16,
}

impl AddressBus {
    pub fn new(memory: Box<dyn Memory>) -> AddressBus {
        AddressBus { memory, pc: 0 }
    }

    pub fn fetch_byte_at_pc(&mut self) -> Result<u8, AddressError> {
        let op = self.memory.read(self.pc)?;
        self.pc += 1;
        Ok(op)
    }

    pub fn fetch_word_at_pc(&mut self) -> Result<u16, AddressError> {
        // little endian, so low byte is read first:
        let lo = self.fetch_byte_at_pc()? as u16;
        let hi = self.fetch_byte_at_pc()? as u16;
        Ok((hi << 8) | lo)
    }

    pub fn set_pc(&mut self, address: u16) -> Result<(), AddressError> {
        if (address as usize) >= self.memory.get_size() {
            return Err(AddressError::InvalidAddress);
        }
        self.pc = address;
        Ok(())
    }

    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    pub fn load_program(&mut self, start_addr: u16, program: &[u8]) -> Result<(), AddressError> {
        self.memory.load_program(start_addr, program)?;
        Ok(())
    }

    pub fn read(&self, address: u16) -> Result<u8, AddressError> {
        self.memory.read(address)
    }

    pub fn read_word(&self, address: u16) -> Result<u16, AddressError> {
        // little endian, so low byte is read first:
        let lo = self.read(address)? as u16;
        let hi = self.read(address + 1)? as u16;
        Ok((hi << 8) | lo)
    }

    pub fn read_zero_page_word(&self, zero_page_addr: u8) -> Result<u16, AddressError> {
        let lo = self.read(zero_page_addr as u16)? as u16;
        let hi = self.read(zero_page_addr.wrapping_add(1) as u16)? as u16;
        Ok((hi << 8) | lo)
    }

    pub fn write(&mut self, address: u16, value: u8) -> Result<(), AddressError> {
        self.memory.write(address, value)
    }
}

impl std::fmt::Debug for AddressBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AddressBus {{ pc: 0x{:04X}, size: {} }}",
            self.pc,
            self.memory.get_size()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::tests;

    #[test]
    fn can_fetch_next_op() -> Result<(), AddressError> {
        let mut memory: Box<tests::MockMemory> = Box::default();
        memory.write(0x0000, 0x01)?;
        let mut bus = AddressBus::new(memory);

        assert_eq!(bus.fetch_byte_at_pc()?, 0x01);
        assert_eq!(bus.get_pc(), 0x0001);
        Ok(())
    }

    #[test]
    fn has_debug_fmt() {
        let memory = Box::new(tests::MockMemory::new());
        let bus = AddressBus::new(memory);

        let debug_msg = format!("{:?}", bus);
        assert_eq!(debug_msg, "AddressBus { pc: 0x0000, size: 1024 }");
    }

    #[test]
    #[should_panic]
    fn fetch_next_op_panics_on_invalid_address() {
        let memory: Box<tests::MockMemory> = Box::default();
        let mut bus = AddressBus::new(memory);

        let top_address = bus.memory.get_size() as u16;
        bus.set_pc(top_address - 1).unwrap();
        // first fetch will succeed, but moves pc to 0x0100
        bus.fetch_byte_at_pc().unwrap();
        assert_eq!(bus.get_pc(), top_address);

        // this fetch should panic
        bus.fetch_byte_at_pc().unwrap();
    }

    #[test]
    fn load_program() -> Result<(), AddressError> {
        let memory: Box<tests::MockMemory> = Box::default();
        let mut bus = AddressBus::new(memory);

        let program = [0x01, 0x02, 0x03];
        bus.load_program(0x00FE, &program)?;

        assert_eq!(bus.memory.read(0x00FE)?, 0x01);
        assert_eq!(bus.memory.read(0x00FF)?, 0x02);
        assert_eq!(bus.memory.read(0x0100)?, 0x03);
        Ok(())
    }
}
