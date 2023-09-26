use crate::disassembler::disassemble;
use crate::{Cpu, CpuError, CpuRegisterSnapshot};

pub trait Debugger<'a> {
    // fn reset(&mut self) -> Result<(), CpuError>;
    fn set_pc(&mut self, addr: u16) -> Result<(), CpuError>;
    fn get_register_snapshot(&self) -> CpuRegisterSnapshot;
    fn step(&mut self) -> Result<CpuRegisterSnapshot, CpuError>;
    fn run(&mut self, start_addr: Option<u16>) -> Result<CpuRegisterSnapshot, CpuError>;
    fn disassemble(&self, start_addr: u16, lines: usize) -> Result<Vec<String>, CpuError>;
}

pub struct DebuggerImpl<'a> {
    cpu: &'a mut Cpu,
}

impl<'a> DebuggerImpl<'a> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(cpu: &mut Cpu) -> Box<dyn Debugger + '_> {
        Box::new(DebuggerImpl { cpu })
    }
}

impl<'a> Debugger<'_> for DebuggerImpl<'a> {
    fn set_pc(&mut self, addr: u16) -> Result<(), CpuError> {
        self.cpu.set_pc(addr)?;
        Ok(())
    }

    fn get_register_snapshot(&self) -> CpuRegisterSnapshot {
        self.cpu.get_register_snapshot()
    }

    fn step(&mut self) -> Result<CpuRegisterSnapshot, CpuError> {
        self.cpu.step()?;
        Ok(self.cpu.get_register_snapshot())
    }

    fn run(&mut self, start_addr: Option<u16>) -> Result<CpuRegisterSnapshot, CpuError> {
        self.cpu.run(start_addr)?;
        Ok(self.cpu.get_register_snapshot())
    }

    fn disassemble(&self, start_addr: u16, lines: usize) -> Result<Vec<String>, CpuError> {
        let mut disassembled_lines = Vec::new();
        let mut cnt = lines;
        let mut next_addr = start_addr;
        while next_addr != 0 && cnt > 0 {
            let line: String;
            (line, next_addr) = disassemble(self.cpu, next_addr)?;
            disassembled_lines.push(line);
            cnt -= 1;
        }
        Ok(disassembled_lines)
    }
}
