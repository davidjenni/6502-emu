use crate::{Cpu, CpuError, CpuRegisterSnapshot};

pub trait Debugger<'a> {
    // fn reset(&mut self) -> Result<(), CpuError>;
    fn set_pc(&mut self, addr: u16) -> Result<(), CpuError>;
    fn get_register_snapshot(&self) -> CpuRegisterSnapshot;
    fn step(&mut self) -> Result<CpuRegisterSnapshot, CpuError>;
    fn run(&mut self, start_addr: Option<u16>) -> Result<CpuRegisterSnapshot, CpuError>;
    fn list(&self, start_addr: u16, end_addr: u16) -> Result<&[&str], CpuError>;
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

    // fn list(&self, start_addr: u16, end_addr: u16) -> Result<&[&str], CpuError> {
    fn list(&self, _: u16, _: u16) -> Result<&[&str], CpuError> {
        // let mut addr = start_addr;
        // while addr <= end_addr {
        //     let opcode = self.memory.read(addr).unwrap();
        //     let instruction = self.decode_instruction(opcode).unwrap();
        //     println!("{:04X}  {:02X}  {}", addr, opcode, instruction);
        //     addr += instruction.size();
        // }
        Ok(&["BRK"])
    }
}
