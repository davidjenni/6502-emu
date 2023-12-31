use std::cmp;
use std::io;

use crate::console_io::StdIo;
use crate::dbg_cmd_parser::{parse_cmd, AddressRange, DebugCmdError, DebugCommand};
use mos6502_emulator::{Cpu, CpuError, CpuRegisterSnapshot};

pub struct Debugger<'a> {
    stdio: &'a mut dyn StdIo,
    last_cmd: DebugCommand,
    last_prog_addr: Option<u16>,
    last_mem_addr: Option<u16>,
}

impl Debugger<'_> {
    pub fn new(stdio: &mut dyn StdIo) -> Debugger {
        Debugger {
            stdio,
            last_cmd: DebugCommand::Invalid,
            last_prog_addr: None,
            last_mem_addr: None,
        }
    }

    pub fn debug_loop(
        &mut self,
        cpu: &mut Box<dyn Cpu>,
    ) -> Result<CpuRegisterSnapshot, DebugCmdError> {
        self.print_snapshot(cpu, cpu.get_register_snapshot())?;
        loop {
            let cmd = self.get_user_input()?;
            match cmd {
                DebugCommand::Step => {
                    let snapshot = cpu.step()?;
                    self.print_snapshot(cpu, snapshot)?;
                    self.last_prog_addr = None;
                }
                DebugCommand::Continue => {
                    let snapshot = cpu.run(Some(cpu.get_pc()))?;
                    self.print_snapshot(cpu, snapshot)?;
                    self.last_prog_addr = None;
                }
                DebugCommand::Disassemble(addr_range) => {
                    let (start, end, line_cnt) =
                        calculate_range(self.last_prog_addr, addr_range, cpu);
                    let mut next_addr = start;
                    loop {
                        let (lines, next) = cpu.disassemble(next_addr, line_cnt.unwrap_or(10))?;
                        for line in lines {
                            self.writeln(format!("  {}", line).as_str());
                        }
                        next_addr = next;
                        if next_addr >= end {
                            break;
                        }
                    }
                    self.last_prog_addr = Some(next_addr);
                }
                DebugCommand::Memory(addr_range) => {
                    let (start, end, _) = calculate_range(self.last_mem_addr, addr_range, cpu);
                    let mut next_addr = start;
                    while let Ok((line, next)) =
                        self.format_memory_ln(cpu, next_addr, cmp::min(end - next_addr, 16))
                    {
                        self.writeln(line.as_str());
                        next_addr = next;
                        if next_addr >= end {
                            break;
                        }
                    }
                    self.last_mem_addr = Some(next_addr);
                }
                DebugCommand::Help | DebugCommand::Invalid => {
                    self.show_usage();
                }
                DebugCommand::Quit => {
                    self.writeln("Exiting...");
                    break;
                }
                DebugCommand::Repeat => panic!("not reachable"),
            }
        }
        Ok(cpu.get_register_snapshot())
    }

    fn format_memory_ln(
        &self,
        cpu: &mut Box<dyn Cpu>,
        addr: u16,
        cnt: u16,
    ) -> Result<(String, u16), DebugCmdError> {
        let mut msg = format!("  {:04X}:", addr);
        let mut next_addr = addr;
        for i in 0..cnt {
            msg.push_str(format!(" {:02X}", cpu.get_byte_at(addr + i)?).as_str());
            next_addr = next_addr.wrapping_add(1);
        }
        Ok((msg, next_addr))
    }

    fn writeln(&mut self, msg: &str) {
        self.write(msg);
        self.write("\n");
    }

    fn write(&mut self, msg: &str) {
        let _ = self.stdio.write(msg);
    }

    fn get_user_input(&mut self) -> Result<DebugCommand, DebugCmdError> {
        self.write("(dbg)> ");
        let mut input = String::new();
        let _ = self.stdio.read_line(&mut input);
        let mut cmd = match parse_cmd(input.trim()) {
            Ok(cmd) => cmd,
            Err(e) => match e {
                DebugCmdError::InvalidCommand(_) => {
                    self.writeln(e.to_string().as_str());
                    DebugCommand::Invalid
                }
                DebugCmdError::InvalidAddressRange(_) => {
                    self.writeln(e.to_string().as_str());
                    DebugCommand::Invalid
                }
                DebugCmdError::CpuError(_) => {
                    self.writeln(e.to_string().as_str());
                    DebugCommand::Invalid
                }
            },
        };
        if cmd == DebugCommand::Repeat {
            cmd = match self.last_cmd {
                DebugCommand::Memory(_) => DebugCommand::Memory(AddressRange::Default),
                DebugCommand::Disassemble(_) => DebugCommand::Disassemble(AddressRange::Default),
                _ => self.last_cmd,
            }
        } else {
            self.last_cmd = cmd;
        }
        Ok(cmd)
    }

    fn print_snapshot(
        &mut self,
        cpu: &mut Box<dyn Cpu>,
        snapshot: CpuRegisterSnapshot,
    ) -> Result<(), DebugCmdError> {
        print_register(&mut self.stdio.get_writer(), snapshot);
        let (current_op, _) = cpu.disassemble(cpu.get_pc(), 1)?;
        self.writeln(format!("    {}", current_op[0]).as_str());
        Ok(())
    }

    fn show_usage(&mut self) {
        self.writeln("Usage:");
        self.writeln("  <empty line>                  - repeat last command");
        self.writeln("  step (s)                      - step one instruction");
        self.writeln("  continue (c)                  - continue execution");
        self.writeln("  disassemble (di) [addr_range] - disassemble instructions at address range");
        self.writeln("  memory (m) [addr_range]       - print memory at address range");
        self.writeln("  quit (q)                      - quit debugger");
        self.writeln("");
        self.writeln("  addr_range:");
        self.writeln("  <empty>                     - current PC or last address with increment");
        self.writeln("  <start_addr>                - start_addr dec or hex prefix '0x' or '$'");
        self.writeln("  <start_addr>..<end_addr>    - exclusive range from start_addr to end_addr");
        self.writeln("  <start_addr>..=<end_addr>   - inclusive range from start_addr to end_addr");
        self.writeln("  <start_addr>,<line_cnt>     - range from start_addr for line_cnt lines");
    }
}

impl From<CpuError> for DebugCmdError {
    fn from(e: CpuError) -> Self {
        DebugCmdError::CpuError(e.to_string())
    }
}

pub fn print_register(writer: &mut dyn io::Write, snapshot: CpuRegisterSnapshot) {
    let msg = format!(
        "PC: {:04X}: A: {:02X} X: {:02X} Y: {:02X} S: {:08b} SP: {:04X}\n",
        snapshot.program_counter,
        snapshot.accumulator,
        snapshot.x_register,
        snapshot.y_register,
        snapshot.status,
        snapshot.stack_pointer,
    );
    writer.write_all(msg.as_bytes()).unwrap();
}

fn calculate_range(
    last_addr: Option<u16>,
    addr_range: AddressRange,
    cpu: &mut Box<dyn Cpu>,
) -> (u16, u16, Option<usize>) {
    let pc = cpu.get_pc();
    let (start, end, line_cnt) = match addr_range {
        AddressRange::StartEnd((start, end)) => (start, end, None),
        AddressRange::StartLines((start, line_cnt)) => {
            (start, start.wrapping_add(line_cnt as u16), Some(line_cnt))
        }
        AddressRange::Default => match last_addr {
            Some(addr) => (addr, addr.wrapping_add(16), None),
            None => (pc, pc.wrapping_add(16), None),
        },
    };
    (start, end, line_cnt)
}

#[cfg(test)]
mod tests {
    use crate::console_io::tests::Spy;

    use super::*;

    fn create_debugger(spy: &mut Spy) -> Debugger {
        Debugger {
            stdio: spy,
            last_cmd: DebugCommand::Invalid,
            last_prog_addr: None,
            last_mem_addr: None,
        }
    }

    #[test]
    fn debug_loop_disassemble() -> Result<(), DebugCmdError> {
        let mut spy = Spy::new("disassemble\n\nstep\n\nquit\n");
        let mut debugger = create_debugger(&mut spy);
        let mut cpu = mos6502_emulator::create_cpu(mos6502_emulator::CpuType::MOS6502)?;
        cpu.load_program(0x0300, &[0xA9, 0x42, 0x85, 0x0F, 0x00], true)?;
        cpu.set_pc(0x0300)?;
        let snapshot = debugger.debug_loop(&mut cpu)?;

        assert_eq!(snapshot.program_counter, 0x0304);
        assert_eq!(debugger.last_cmd, DebugCommand::Quit);
        let stdout = spy.get_stdout();
        // println!("{}", stdout);
        assert!(stdout.contains("PC: 0300: A: 00 X: 00 Y: 00 S: 00000010 SP: 01FF"));
        assert!(stdout.contains("0300 LDA #$42"));
        assert!(stdout.contains("PC: 0302: A: 42"));
        Ok(())
    }

    #[test]
    fn debug_loop_disassemble_by_lines() -> Result<(), DebugCmdError> {
        let mut spy = Spy::new("di $0300,4\n\nquit\n");
        let mut debugger = create_debugger(&mut spy);
        let mut cpu = mos6502_emulator::create_cpu(mos6502_emulator::CpuType::MOS6502)?;
        cpu.load_program(0x0300, &[0xA9, 0x42, 0x85, 0x0F, 0x00], true)?;
        cpu.set_pc(0x0300)?;
        debugger.debug_loop(&mut cpu)?;

        let stdout = spy.get_stdout();
        // println!("{}", stdout);
        assert!(stdout.contains("PC: 0300: A: 00 X: 00 Y: 00 S: 00000010 SP: 01FF"));
        assert!(stdout.contains("0300 LDA #$42"));
        Ok(())
    }

    #[test]
    fn debug_loop_memory() -> Result<(), DebugCmdError> {
        let mut spy = Spy::new("memory 0x020..0x42\n\nquit\n");
        let mut debugger = create_debugger(&mut spy);
        let mut cpu = mos6502_emulator::create_cpu(mos6502_emulator::CpuType::MOS6502)?;
        cpu.set_pc(0x0300)?;
        debugger.debug_loop(&mut cpu)?;

        let stdout = spy.get_stdout();
        // println!("{}", stdout);
        assert!(stdout.contains("0020: 00 00 00 00"));
        assert!(stdout.contains("0030: 00 00 00 00"));
        assert!(stdout.contains("0040: 00"));
        assert!(stdout.contains("0041: 00 00 00 00"));
        Ok(())
    }

    #[test]
    fn debug_loop_memory_illegal_address() -> Result<(), DebugCmdError> {
        let mut spy = Spy::new("memory 0xA2042\nquit\n");
        let mut debugger = create_debugger(&mut spy);
        let mut cpu = mos6502_emulator::create_cpu(mos6502_emulator::CpuType::MOS6502)?;
        cpu.set_pc(0x0300)?;
        debugger.debug_loop(&mut cpu)?;

        let stdout = spy.get_stdout();
        // println!("{}", stdout);
        assert!(stdout.contains("Invalid address: must be between 0 and 65535"));
        assert!(stdout.contains("Usage:"));
        Ok(())
    }

    #[test]
    fn usage() -> Result<(), DebugCmdError> {
        let mut spy = Spy::new("help\nquit\n");
        let mut debugger = create_debugger(&mut spy);
        let mut cpu = mos6502_emulator::create_cpu(mos6502_emulator::CpuType::MOS6502)?;
        cpu.load_program(0x0300, &[0x00], true)?;
        cpu.set_pc(0x0300)?;
        debugger.debug_loop(&mut cpu)?;

        let stdout = spy.get_stdout();
        // println!("{}", stdout);
        assert!(stdout.contains("Usage:"));
        assert!(stdout
            .contains("disassemble (di) [addr_range] - disassemble instructions at address range"));
        Ok(())
    }

    #[test]
    fn cpu_error_format() {
        let err = CpuError::InvalidOpcode(0x42);
        let dbg_err = DebugCmdError::from(err);
        assert!(dbg_err.to_string().contains("Cpu Error: illegal op code"));
    }
}
