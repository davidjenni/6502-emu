use crate::memory::{AddressError, Memory};

pub struct AddressBus {
    memory: Box<dyn Memory>,
    pc: u16,
}

impl AddressBus {
    pub fn new(memory: Box<dyn Memory>) -> AddressBus {
        AddressBus { memory, pc: 0 }
    }

    pub fn fetch_next_op(&mut self) -> Result<u8, AddressError> {
        let op = self.memory.read(self.pc)?;
        self.pc += 1;
        Ok(op)
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

#[derive(Debug)]
pub enum AddressingMode {
    Implied,         // CLC
    Accumulator,     // ASL A
    Immediate,       // LDA #10
    ZeroPage,        // LDA $10
    ZeroPageX,       // LDA $10,X
    ZeroPageY,       // LDA $10,Y
    Relative,        // BNE $10
    Absolute,        // LDA $1234
    AbsoluteX,       // LDA $1234,X
    AbsoluteY,       // LDA $1234,Y
    IndexedIndirect, // LDA ($10,X)
    Indirect,        // JMP ($1234)
    IndirectIndexed, // LDA ($10),Y
}

impl AddressingMode {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::tests;

    #[test]
    fn can_fetch_next_op() -> Result<(), AddressError> {
        let mut memory: Box<tests::MockMemory> = Box::default();
        memory.write(0x0000, 0x01)?;
        let mut bus = AddressBus::new(memory);

        assert_eq!(bus.fetch_next_op()?, 0x01);
        assert_eq!(bus.get_pc(), 0x0001);
        Ok(())
    }

    #[test]
    fn has_debug_fmt() {
        let memory = Box::new(tests::MockMemory::new());
        let bus = AddressBus::new(memory);

        let debug_msg = format!("{:?}", bus);
        assert_eq!(debug_msg, "AddressBus { pc: 0x0000, size: 256 }");
    }

    #[test]
    #[should_panic]
    fn fetch_next_op_panics_on_invalid_address() {
        let memory = Box::new(tests::MockMemory::new());
        let mut bus = AddressBus::new(memory);

        bus.set_pc(0x0100 - 1).unwrap();
        // first fetch will succeed, but moves pc to 0x0100
        bus.fetch_next_op().unwrap();
        assert_eq!(bus.get_pc(), 0x0100);

        // this fetch should panic
        bus.fetch_next_op().unwrap();
    }
}
