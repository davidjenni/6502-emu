use crate::memory::Memory;

pub struct AddressBus {
    memory: Box<dyn Memory>,
    pc: u16,
}

impl AddressBus {
    pub fn new(memory: Box<dyn Memory>) -> AddressBus {
        AddressBus { memory, pc: 0 }
    }

    pub fn fetch_next_op(&mut self) -> u8 {
        let op = self.memory.read(self.pc);
        self.pc += 1;
        op
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

    struct MockMemory {
        memory: [u8; 256],
    }
    impl MockMemory {
        fn new() -> MockMemory {
            MockMemory { memory: [0; 256] }
        }
    }

    impl Memory for MockMemory {
        fn read(&self, address: u16) -> u8 {
            self.memory[address as usize]
        }

        fn write(&mut self, address: u16, value: u8) {
            self.memory[address as usize] = value;
        }

        fn get_size(&self) -> usize {
            self.memory.len()
        }
    }

    #[test]
    fn can_fetch_next_op() {
        let mut memory = Box::new(MockMemory::new());
        memory.write(0x0000, 0x01);
        let mut bus = AddressBus::new(memory);
        assert_eq!(bus.fetch_next_op(), 0x01);
        assert_eq!(bus.get_pc(), 0x0001);
    }

    #[test]
    fn has_debug_fmt() {
        let memory = Box::new(MockMemory::new());
        let bus = AddressBus::new(memory);
        let debug_msg = format!("{:?}", bus);
        assert_eq!(debug_msg, "AddressBus { pc: 0x0000, size: 256 }");
    }
}
