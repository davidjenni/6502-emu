use crate::disassembler::disassemble;
use crate::{Cpu, CpuType};
use crate::{CpuError, CpuImpl, CpuRegisterSnapshot};

pub struct CpuControllerImpl {
    cpu: CpuImpl,
}

impl CpuControllerImpl {
    pub fn create(kind: CpuType) -> Result<Box<dyn Cpu>, CpuError> {
        let mut cpu = match kind {
            CpuType::MOS6502 => CpuControllerImpl {
                cpu: CpuImpl::default(),
            },
        };
        cpu.reset()?;
        Ok(Box::new(cpu))
    }
}

impl Cpu for CpuControllerImpl {
    fn reset(&mut self) -> Result<(), CpuError> {
        self.cpu.reset()
    }

    fn load_program(&mut self, start_addr: u16, program: &[u8]) -> Result<(), CpuError> {
        self.cpu.load_program(start_addr, program)
    }

    fn set_pc(&mut self, addr: u16) -> Result<(), CpuError> {
        self.cpu.set_pc(addr)?;
        Ok(())
    }

    fn get_pc(&self) -> u16 {
        self.cpu.get_pc()
    }

    fn run(&mut self, start_addr: Option<u16>) -> Result<CpuRegisterSnapshot, CpuError> {
        self.cpu.run(start_addr)?;

        Ok(self.cpu.get_register_snapshot())
    }

    fn step(&mut self) -> Result<CpuRegisterSnapshot, CpuError> {
        self.cpu.step()?;
        Ok(self.cpu.get_register_snapshot())
    }

    fn get_register_snapshot(&self) -> CpuRegisterSnapshot {
        self.cpu.get_register_snapshot()
    }

    fn disassemble(&self, start_addr: u16, lines: usize) -> Result<Vec<String>, CpuError> {
        let mut disassembled_lines = Vec::new();
        let mut cnt = lines;
        let mut next_addr = start_addr;
        while next_addr != 0 && cnt > 0 {
            let line: String;
            (line, next_addr) = disassemble(&self.cpu, next_addr)?;
            disassembled_lines.push(line);
            cnt -= 1;
        }
        Ok(disassembled_lines)
    }

    fn get_byte_at(&self, address: u16) -> Result<u8, CpuError> {
        self.cpu.get_byte_at(address)
    }

    fn set_byte_at(&mut self, address: u16, value: u8) -> Result<(), CpuError> {
        self.cpu.set_byte_at(address, value)
    }
}
