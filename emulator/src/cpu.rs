use crate::address_bus::AddressBusImpl;
use crate::address_bus::{AddressBus, SystemVector};
use crate::engine::decoder;
use crate::engine::decoder::DecodedInstruction;
use crate::engine::opcodes::OpCode;
use crate::memory::Memory;
use crate::memory::MemoryImpl;
use crate::stack_pointer::StackPointer;
use crate::stack_pointer::StackPointerImpl;
use crate::status_register::StatusRegister;
use crate::CpuController;
use crate::CpuError;
use crate::CpuRegisterSnapshot;
use std::time::Instant;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AddressingMode {
    Implied,          // CLC
    Accumulator,      // ASL A
    Immediate,        // LDA #10
    ZeroPage,         // LDA $10
    ZeroPageX,        // LDA $10,X
    ZeroPageY,        // LDA $10,Y
    Relative,         // BNE $10
    Absolute,         // LDA $1234
    AbsoluteX,        // LDA $1234,X
    AbsoluteY,        // LDA $1234,Y
    Indirect,         // JMP ($1234)
    IndexedXIndirect, // LDA ($10,X)
    IndirectIndexedY, // LDA ($10),Y
}

#[derive(Debug)]
pub struct Cpu {
    pub accumulator: u8,
    pub index_x: u8,
    pub index_y: u8,
    pub status: StatusRegister,

    pub memory: Box<dyn Memory>, // TODO: should be reverted back to private
    pub address_bus: Box<dyn AddressBus>, // TODO: should be reverted back to private
    pub stack: Box<dyn StackPointer>, // TODO: should be reverted back to private

    // stats counters:
    accumulated_cycles: u64,
    accumulated_instructions: u64,
    approximate_clock_speed: f64,
}

impl CpuController for Cpu {
    fn reset(&mut self) -> Result<(), CpuError> {
        self.stack.reset()?;
        self.address_bus.set_pc(SystemVector::Reset as u16)?;
        self.accumulated_cycles = 0;
        self.accumulated_instructions = 0;
        Ok(())
    }

    fn run(&mut self, start_addr: Option<u16>) -> Result<CpuRegisterSnapshot, CpuError> {
        // delegate implementation to Cpu
        self.run(start_addr)?;
        Ok(self.get_register_snapshot())
    }

    fn step(&mut self) -> Result<u16, CpuError> {
        // delegate implementation to Cpu
        self.step()?;
        Ok(self.address_bus.get_pc())
    }

    fn load_program(&mut self, start_addr: u16, program: &[u8]) -> Result<(), CpuError> {
        self.memory.load_program(start_addr, program)
    }

    fn get_register_snapshot(&self) -> crate::CpuRegisterSnapshot {
        crate::CpuRegisterSnapshot {
            accumulator: self.accumulator,
            x_register: self.index_x,
            y_register: self.index_y,
            stack_pointer: self.stack.get_sp().unwrap(),
            program_counter: self.address_bus.get_pc(),
            status: self.status.get_status(),
            accumulated_cycles: self.accumulated_cycles,
            accumulated_instructions: self.accumulated_instructions,
            approximate_clock_speed: self.approximate_clock_speed,
        }
    }

    fn get_byte_at(&self, address: u16) -> Result<u8, CpuError> {
        self.memory.read(address)
    }

    fn set_byte_at(&mut self, address: u16, value: u8) -> Result<(), CpuError> {
        self.memory.write(address, value)
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            accumulator: 0,
            index_x: 0,
            index_y: 0,
            status: StatusRegister::new(),
            memory: Box::<MemoryImpl>::default(),
            address_bus: Box::new(AddressBusImpl::new()),
            stack: Box::new(StackPointerImpl::new()),
            accumulated_cycles: 0,
            accumulated_instructions: 0,
            approximate_clock_speed: 0.0,
        }
    }

    pub fn run(&mut self, start_addr: Option<u16>) -> Result<(), CpuError> {
        self.address_bus.set_pc(if start_addr.is_some() {
            start_addr.unwrap()
        } else {
            SystemVector::Reset as u16
        })?;
        let start = Instant::now();
        loop {
            let (last_opcode, cycles) = self.step()?;
            self.accumulated_instructions += 1;
            self.accumulated_cycles += cycles as u64;
            if last_opcode == OpCode::BRK {
                break;
            }
        }
        let elapsed = start.elapsed().as_secs_f64();
        self.approximate_clock_speed = self.accumulated_cycles as f64 / elapsed;
        Ok(())
    }

    pub fn step(&mut self) -> Result<(OpCode, u8), CpuError> {
        let decoded = self.fetch_and_decode()?;
        (decoded.execute)(decoded.mode, self)?;
        Ok((decoded.opcode, decoded.cycles))
    }

    fn fetch_and_decode(&mut self) -> Result<DecodedInstruction, CpuError> {
        let opcode = self.address_bus.fetch_byte_at_pc(self.memory.as_mut())?;
        let res = decoder::decode(opcode)?;
        Ok(res)
    }

    /// Fetches bytes according to addressing mode to calculate effective address;
    /// it also moves the PC to the next instruction.
    pub fn get_effective_address(&mut self, mode: AddressingMode) -> Result<u16, CpuError> {
        match mode {
            AddressingMode::ZeroPage => {
                Ok(self.address_bus.fetch_byte_at_pc(self.memory.as_mut())? as u16)
            }
            AddressingMode::ZeroPageX => Ok(self
                .address_bus
                .fetch_byte_at_pc(self.memory.as_mut())?
                .wrapping_add(self.index_x) as u16),
            AddressingMode::ZeroPageY => Ok(self
                .address_bus
                .fetch_byte_at_pc(self.memory.as_mut())?
                .wrapping_add(self.index_y) as u16),
            AddressingMode::Relative => {
                let offset = self.address_bus.fetch_byte_at_pc(self.memory.as_mut())? as i8;
                Ok(self.address_bus.get_pc().wrapping_add(offset as u16))
            }
            AddressingMode::Absolute => {
                let word = self.address_bus.fetch_word_at_pc(self.memory.as_mut())?;
                Ok(word)
            }
            AddressingMode::AbsoluteX => {
                let word = self.address_bus.fetch_word_at_pc(self.memory.as_mut())?;
                Ok(word + self.index_x as u16)
            }
            AddressingMode::AbsoluteY => {
                let word = self.address_bus.fetch_word_at_pc(self.memory.as_mut())?;
                Ok(word + self.index_y as u16)
            }
            AddressingMode::Indirect => {
                let indirect_addr = self.address_bus.fetch_word_at_pc(self.memory.as_mut())?;
                // 6502 bug: if low byte is 0xff, then high byte is fetched from non-incremented high byte
                // i.e. no proper page boundary crossing
                let low_indirect = self.memory.read(indirect_addr)? as u16;

                let high_indirect = if indirect_addr & 0xff == 0xff {
                    self.memory.read(indirect_addr & 0xff00)?
                } else {
                    self.memory.read(indirect_addr + 1)?
                } as u16;
                Ok(high_indirect << 8 | low_indirect)
            }
            AddressingMode::IndexedXIndirect => {
                let zero_page_addr = self.get_effective_address(AddressingMode::ZeroPageX)?;
                let word = self.memory.read_zero_page_word(zero_page_addr as u8)?;
                Ok(word)
            }
            AddressingMode::IndirectIndexedY => {
                let zero_page_addr = self.get_effective_address(AddressingMode::ZeroPage)?;
                let word =
                    self.memory.read_zero_page_word(zero_page_addr as u8)? + self.index_y as u16;
                Ok(word)
            }
            // Implied, Accumulator, Immediate modes have no address
            _ => Err(CpuError::InvalidAddressingMode),
        }
    }

    pub fn get_effective_operand(&mut self, mode: AddressingMode) -> Result<u8, CpuError> {
        match mode {
            AddressingMode::Immediate => self.address_bus.fetch_byte_at_pc(self.memory.as_mut()),
            AddressingMode::Accumulator => Ok(self.accumulator),
            AddressingMode::Relative => Err(CpuError::InvalidAddressingMode),
            _ => {
                let effective_address = self.get_effective_address(mode)?;
                self.memory.read(effective_address)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const START_ADDR: u16 = 0x0300;
    const ZERO_PAGE_ADDR: u8 = 0xE0;
    const OP_CODE: u8 = 0xa5; // arbitrary opcode used for unit testing
    const EXPECTED: u8 = 42;

    fn setup_test_cpu(program: &[u8]) -> Result<Cpu, CpuError> {
        let mut cpu = Cpu::default();

        cpu.load_program(START_ADDR, program)?;
        cpu.address_bus.set_pc(START_ADDR)?;

        // prepare PC to point past the opcode:
        let opcode_read = cpu.address_bus.fetch_byte_at_pc(cpu.memory.as_mut())?;
        assert_eq!(opcode_read, OP_CODE);

        Ok(cpu)
    }

    fn populate_zero_page(mem: &mut dyn Memory, data: &[u8]) -> Result<(), CpuError> {
        mem.load_program(ZERO_PAGE_ADDR as u16, data)?;
        Ok(())
    }

    fn assert_final_pc(cpu: &Cpu, delta: u16) {
        assert_eq!(cpu.address_bus.get_pc(), START_ADDR + delta);
    }

    //============= get_effective_operand tests =============
    #[test]
    #[should_panic(expected = "InvalidAddressingMode")]
    fn get_effective_operand_implied() {
        let mut cpu = Cpu::default();

        cpu.get_effective_operand(AddressingMode::Implied).unwrap();
    }

    #[test]
    fn get_effective_operand_accumulator() -> Result<(), CpuError> {
        let mut cpu = setup_test_cpu(&[OP_CODE])?;
        cpu.accumulator = EXPECTED;

        let res = cpu.get_effective_operand(AddressingMode::Accumulator)?;
        assert_eq!(res, EXPECTED);
        assert_final_pc(&cpu, 1);
        Ok(())
    }

    #[test]
    fn get_effective_operand_immediate() -> Result<(), CpuError> {
        let mut cpu = setup_test_cpu(&[OP_CODE, EXPECTED])?;

        let res = cpu.get_effective_operand(AddressingMode::Immediate)?;
        assert_eq!(res, EXPECTED);
        assert_final_pc(&cpu, 2);
        Ok(())
    }

    #[test]
    fn get_effective_operand_zero_page() -> Result<(), CpuError> {
        let mut cpu = setup_test_cpu(&[OP_CODE, ZERO_PAGE_ADDR])?;

        populate_zero_page(cpu.memory.as_mut(), &[EXPECTED])?;

        let res = cpu.get_effective_operand(AddressingMode::ZeroPageX)?;
        assert_eq!(res, EXPECTED);
        assert_final_pc(&cpu, 2);
        Ok(())
    }

    #[test]
    fn get_effective_operand_zero_page_indexed_x() -> Result<(), CpuError> {
        let mut cpu = setup_test_cpu(&[OP_CODE, ZERO_PAGE_ADDR])?;

        populate_zero_page(cpu.memory.as_mut(), &[0xaa, 0xbb, 0xcc, EXPECTED])?;

        cpu.index_x = 3;
        cpu.index_y = 2; // should be ignored, since testing Indexed_X
        let res = cpu.get_effective_operand(AddressingMode::ZeroPageX)?;
        assert_eq!(res, EXPECTED);
        assert_final_pc(&cpu, 2);
        Ok(())
    }

    #[test]
    fn get_effective_operand_zero_page_indexed_y() -> Result<(), CpuError> {
        let mut cpu = setup_test_cpu(&[OP_CODE, ZERO_PAGE_ADDR])?;

        populate_zero_page(cpu.memory.as_mut(), &[0xaa, 0xbb, EXPECTED])?;

        cpu.index_y = 2;
        cpu.index_x = 3; // should be ignored, since testing Indexed_Y
        let res = cpu.get_effective_operand(AddressingMode::ZeroPageY)?;
        assert_eq!(res, EXPECTED);
        assert_final_pc(&cpu, 2);
        Ok(())
    }

    #[test]
    fn get_effective_operand_absolute() -> Result<(), CpuError> {
        // abs address at offset 0x08 from start:
        let mut cpu = setup_test_cpu(&[
            // abs address at offset 0x08 from start:
            //  0,   1,    2,    3,    4,    5,    6,    7,    8,        9
            OP_CODE, 0x08, 0x03, 0xEA, 0xEA, 0xEA, 0xEA, 0xEA, EXPECTED, 0xEA,
        ])?;
        let res = cpu.get_effective_operand(AddressingMode::Absolute)?;
        assert_eq!(res, EXPECTED);
        assert_final_pc(&cpu, 3);
        Ok(())
    }

    #[test]
    fn get_effective_operand_absolute_indexed_x() -> Result<(), CpuError> {
        let mut cpu = setup_test_cpu(&[
            // abs address at offset 0x06 from start, then indexed by X:
            //  0,   1,    2,    3,    4,    5,    6,    7,    8,        9
            OP_CODE, 0x06, 0x03, 0xEA, 0xEA, 0xEA, 0xEA, 0xEA, EXPECTED, 0xEA,
        ])?;
        cpu.index_x = 2;
        let res = cpu.get_effective_operand(AddressingMode::AbsoluteX)?;
        assert_eq!(res, EXPECTED);
        assert_final_pc(&cpu, 3);
        Ok(())
    }

    #[test]
    fn get_effective_operand_absolute_indexed_y() -> Result<(), CpuError> {
        let mut cpu = setup_test_cpu(&[
            // abs address at offset 0x06 from start, then indexed by X:
            //  0,   1,    2,    3,    4,    5,    6,    7,    8,        9
            OP_CODE, 0x06, 0x03, 0xEA, 0xEA, 0xEA, 0xEA, 0xEA, EXPECTED, 0xEA,
        ])?;
        cpu.index_y = 2;
        let res = cpu.get_effective_operand(AddressingMode::AbsoluteY)?;
        assert_eq!(res, EXPECTED);
        assert_final_pc(&cpu, 3);
        Ok(())
    }

    #[test]
    fn get_effective_operand_indexed_x_indirect() -> Result<(), CpuError> {
        let mut cpu = setup_test_cpu(&[OP_CODE, ZERO_PAGE_ADDR])?;
        populate_zero_page(cpu.memory.as_mut(), &[0xaa, 0xbb, 0x21, 0x03])?;
        cpu.memory.write(0x0321, EXPECTED)?;

        cpu.index_x = 2;
        let res = cpu.get_effective_operand(AddressingMode::IndexedXIndirect)?;
        assert_eq!(res, EXPECTED);
        assert_final_pc(&cpu, 2);
        Ok(())
    }

    #[test]
    fn get_effective_operand_indirect_indexed_y() -> Result<(), CpuError> {
        let mut cpu = setup_test_cpu(&[OP_CODE, ZERO_PAGE_ADDR])?;
        populate_zero_page(cpu.memory.as_mut(), &[0x1F, 0x03])?;
        cpu.memory.write(0x0321, EXPECTED)?;

        cpu.index_y = 2;
        let res = cpu.get_effective_operand(AddressingMode::IndirectIndexedY)?;
        assert_eq!(res, EXPECTED);
        assert_final_pc(&cpu, 2);
        Ok(())
    }

    //============= get_effective_address tests =============
    // these addressing modes cannot be tested with a full operand handling
    #[test]
    fn get_effective_address_relative_forward() -> Result<(), CpuError> {
        let offset: i8 = 42;
        let mut cpu = setup_test_cpu(&[OP_CODE, offset as u8])?;

        let res = cpu.get_effective_address(AddressingMode::Relative)?;
        assert_eq!(res, START_ADDR + 2 + offset as u16);
        assert_final_pc(&cpu, 2);
        Ok(())
    }

    #[test]
    fn get_effective_address_relative_backward() -> Result<(), CpuError> {
        let offset: i8 = 42;
        // negate offset:
        let mut cpu = setup_test_cpu(&[OP_CODE, -offset as u8])?;

        let res = cpu.get_effective_address(AddressingMode::Relative)?;
        let expected_addr = START_ADDR + 2 - offset as u16;
        assert_eq!(res, expected_addr);
        assert_final_pc(&cpu, 2);
        Ok(())
    }

    #[test]
    fn get_effective_address_indirect() -> Result<(), CpuError> {
        let mut cpu = setup_test_cpu(&[
            // indirect address at offset 0x08 from start:
            //  0,   1,    2,    3,    4,    5,    6,    7,    8,        9
            OP_CODE, 0x08, 0x03, 0xEA, 0xEA, 0xEA, 0xEA, 0xEA, 0x34, 0x12,
        ])?;
        let res = cpu.get_effective_address(AddressingMode::Indirect)?;
        assert_eq!(res, 0x1234);
        assert_final_pc(&cpu, 3);
        Ok(())
    }

    #[test]
    fn get_effective_address_indirect_6502bug() -> Result<(), CpuError> {
        let mut cpu = setup_test_cpu(&[
            // indirect address at offset 0x08 from start:
            //  0,   1,    2,    3,    4,    5,    6,    7,    8,        9
            OP_CODE, 0xFF, 0x02, 0xEA, 0xEA, 0xEA, 0xEA, 0xEA, 0x34, 0x12,
        ])?;
        cpu.memory.write(0x02FF, 0x34)?;
        cpu.memory.write(0x0200, 0x12)?;
        let res = cpu.get_effective_address(AddressingMode::Indirect)?;
        assert_eq!(res, 0x1234);
        assert_final_pc(&cpu, 3);
        Ok(())
    }
}
