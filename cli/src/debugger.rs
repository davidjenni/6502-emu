use std::io;

use crate::dbg_cmd_parser::{parse_cmd, DebugCmdError, DebugCommand};
use mos6502_emulator::{Cpu, CpuError, CpuRegisterSnapshot};

pub struct Debugger<R, W, E> {
    stdin: R,
    stdout: W,
    #[allow(dead_code)]
    stderr: E,
    last_cmd: DebugCommand,
    last_addr: Option<u16>,
}

impl<R, W, E> Debugger<R, W, E>
where
    R: io::BufRead,
    W: io::Write,
    E: io::Write,
{
    pub fn new() -> Debugger<Box<dyn io::BufRead>, Box<dyn io::Write>, Box<dyn io::Write>> {
        Debugger {
            stdin: Box::new(io::BufReader::new(io::stdin())),
            stdout: Box::new(io::stdout()),
            stderr: Box::new(io::stderr()),
            last_cmd: DebugCommand::Invalid,
            last_addr: None,
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
                    self.last_addr = None;
                }
                DebugCommand::Continue => {
                    let snapshot = cpu.run(Some(cpu.get_pc()))?;
                    self.print_snapshot(cpu, snapshot)?;
                    self.last_addr = None;
                }
                DebugCommand::Disassemble(_) => {
                    // TODO: consume range
                    let addr = match self.last_addr {
                        Some(addr) => addr,
                        None => cpu.get_pc(),
                    };
                    let (lines, next_addr) = cpu.disassemble(addr, 10)?;
                    for line in lines {
                        self.writeln(format!("  {}", line).as_str());
                    }
                    self.last_addr = Some(next_addr);
                }
                DebugCommand::Memory(_) => {
                    // TODO: consume range
                    let addr = match self.last_addr {
                        Some(addr) => addr,
                        None => cpu.get_pc(),
                    };
                    let mut msg = format!("  {:04X}:", addr);
                    for i in 0..16 {
                        msg.push_str(
                            format!(" {:02X}", cpu.get_byte_at(addr.wrapping_add(i + 1))?).as_str(),
                        );
                    }
                    self.writeln(msg.as_str());
                    self.last_addr = Some(addr + 16);
                }
                DebugCommand::Invalid => {
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

    fn writeln(&mut self, msg: &str) {
        self.write(msg);
        self.write("\n");
    }

    fn write(&mut self, msg: &str) {
        self.stdout.write_all(msg.as_bytes()).unwrap();
        self.stdout.flush().expect("Failed to flush stdout");
    }

    fn get_user_input(&mut self) -> Result<DebugCommand, DebugCmdError> {
        self.write("(dbg)> ");
        let mut input = String::new();
        self.stdin
            .read_line(&mut input)
            .expect("Failed to read user input");
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
            cmd = self.last_cmd
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
        print_register(&mut self.stdout, snapshot);
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

#[cfg(test)]
mod tests {
    // use anyhow::Ok;
    use std::str;

    use super::*;

    struct Spy<'a> {
        stdin: &'a [u8],
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    }

    impl<'a> Spy<'a> {
        pub fn new(input: &'a str) -> Spy<'a> {
            Spy {
                stdin: input.as_bytes(),
                stdout: vec![],
                stderr: vec![],
            }
        }

        pub fn get_stdout(&'a self) -> String {
            str::from_utf8(&self.stdout).unwrap().to_string()
        }

        #[allow(dead_code)]
        pub fn get_stderr(&self) -> String {
            str::from_utf8(&self.stderr).unwrap().to_string()
        }
    }

    #[test]
    fn debug_loop() -> Result<(), DebugCmdError> {
        let mut spy = Spy::new("disassemble\nstep\n\nquit\n");
        let mut debugger = Debugger {
            stdin: Box::new(spy.stdin),
            stdout: &mut spy.stdout,
            stderr: &mut spy.stderr,
            last_cmd: DebugCommand::Invalid,
            last_addr: None,
        };
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
    fn usage() -> Result<(), DebugCmdError> {
        let mut spy = Spy::new("help\nquit\n");
        let mut debugger = Debugger {
            stdin: Box::new(spy.stdin),
            stdout: &mut spy.stdout,
            stderr: &mut spy.stderr,
            last_cmd: DebugCommand::Invalid,
            last_addr: None,
        };
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
}
