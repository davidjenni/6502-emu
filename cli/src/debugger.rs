use anyhow::Result;
use std::io;

use mos6502_emulator::{Cpu, CpuRegisterSnapshot};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DebuggerCommand {
    Step,
    Disassemble,
    // Memory,
    Continue,
    Quit,
    Repeat,
    Invalid,
}

pub struct Debugger<R, W, E> {
    #[allow(dead_code)]
    stdin: R,
    stdout: W,
    #[allow(dead_code)]
    stderr: E,
    last_cmd: DebuggerCommand,
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
            last_cmd: DebuggerCommand::Invalid,
        }
    }

    pub fn debug_loop(&mut self, cpu: &mut Box<dyn Cpu>) -> Result<CpuRegisterSnapshot> {
        print_register(&mut self.stdout, cpu.get_register_snapshot());
        loop {
            let cmd = self.get_user_input();
            match cmd {
                DebuggerCommand::Step => {
                    let snapshot = cpu.step()?;
                    print_register(&mut self.stdout, snapshot);
                }
                DebuggerCommand::Continue => {
                    let snapshot = cpu.run(None)?;
                    print_register(&mut self.stdout, snapshot);
                }
                DebuggerCommand::Disassemble => {
                    let lines = cpu.disassemble(cpu.get_pc(), 10)?;
                    for line in lines {
                        self.writeln(format!("  {}", line).as_str());
                    }
                }
                DebuggerCommand::Invalid => {
                    self.show_usage();
                }
                DebuggerCommand::Quit => {
                    self.writeln("Exiting...");
                    break;
                }
                DebuggerCommand::Repeat => panic!("not reachable"),
            }
        }
        anyhow::Ok(cpu.get_register_snapshot())
    }

    fn writeln(&mut self, msg: &str) {
        self.write(msg);
        self.write("\n");
    }

    fn write(&mut self, msg: &str) {
        self.stdout.write_all(msg.as_bytes()).unwrap();
        self.stdout.flush().expect("Failed to flush stdout");
    }

    pub fn get_user_input(&mut self) -> DebuggerCommand {
        self.write("(dbg)> ");
        let mut input = String::new();
        self.stdin
            .read_line(&mut input)
            .expect("Failed to read user input");
        let mut cmd = parse_command(input.trim());
        if cmd == DebuggerCommand::Repeat {
            cmd = self.last_cmd
        } else {
            self.last_cmd = cmd;
        }
        cmd
    }

    pub fn show_usage(&mut self) {
        self.writeln("Usage:");
        self.writeln("  step (s)          - step one instruction");
        self.writeln("  disassemble (di)  - disassemble instructions from current PC");
        self.writeln("  continue (c)      - continue execution");
        self.writeln("  <empty line>      - repeat last command");
        self.writeln("  quit (q)          - quit debugger");
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

fn parse_command(input: &str) -> DebuggerCommand {
    let input = input.trim().to_ascii_lowercase();
    let mut iter = input.split_whitespace();
    match iter.next() {
        Some("step") | Some("s") => DebuggerCommand::Step,
        Some("disassemble") | Some("di") => DebuggerCommand::Disassemble,
        // Some("memory")  | Some("m")=> DebuggerCommand::Memory,
        Some("continue") | Some("c") => DebuggerCommand::Continue,
        Some("quit") | Some("q") => DebuggerCommand::Quit,
        None => DebuggerCommand::Repeat,
        _ => DebuggerCommand::Invalid,
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Ok;
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
    fn parse_command_upper_lower() {
        assert_eq!(parse_command("step"), DebuggerCommand::Step);
        assert_eq!(parse_command("s"), DebuggerCommand::Step);
        assert_eq!(parse_command("sTeP"), DebuggerCommand::Step);

        assert_eq!(parse_command("continue"), DebuggerCommand::Continue);
        assert_eq!(parse_command("C"), DebuggerCommand::Continue);
        assert_eq!(parse_command("ConTinUE"), DebuggerCommand::Continue);

        assert_eq!(parse_command("disasseMBle"), DebuggerCommand::Disassemble);
        assert_eq!(parse_command("di"), DebuggerCommand::Disassemble);

        assert_eq!(parse_command("quit"), DebuggerCommand::Quit);
        assert_eq!(parse_command("Q"), DebuggerCommand::Quit);

        assert_eq!(parse_command(""), DebuggerCommand::Repeat);

        assert_eq!(parse_command("BlA"), DebuggerCommand::Invalid);
    }

    #[test]
    fn _loop() -> Result<()> {
        let mut spy = Spy::new("disassemble\nstep\nstep\nquit\n");
        let mut debugger = Debugger {
            stdin: Box::new(spy.stdin),
            stdout: &mut spy.stdout,
            stderr: &mut spy.stderr,
            last_cmd: DebuggerCommand::Invalid,
        };
        let mut cpu = mos6502_emulator::create_cpu(mos6502_emulator::CpuType::MOS6502)?;
        cpu.load_program(0x0300, &[0xA9, 0x42, 0x85, 0x0F, 0x00])?;
        cpu.set_pc(0x0300)?;
        let snapshot = debugger.debug_loop(&mut cpu)?;

        assert_eq!(snapshot.program_counter, 0x0304);
        assert_eq!(debugger.last_cmd, DebuggerCommand::Quit);
        let stdout = spy.get_stdout();
        // println!("{}", stdout);
        assert!(stdout.contains("PC: 0300: A: 00 X: 00 Y: 00 S: 00000000 SP: 01FF"));
        assert!(stdout.contains("0300 LDA #$42"));
        assert!(stdout.contains("PC: 0302: A: 42"));
        Ok(())
    }

    #[test]
    fn usage() -> Result<()> {
        let mut spy = Spy::new("help\nquit\n");
        let mut debugger = Debugger {
            stdin: Box::new(spy.stdin),
            stdout: &mut spy.stdout,
            stderr: &mut spy.stderr,
            last_cmd: DebuggerCommand::Invalid,
        };
        let mut cpu = mos6502_emulator::create_cpu(mos6502_emulator::CpuType::MOS6502)?;
        cpu.load_program(0x0300, &[0x00])?;
        cpu.set_pc(0x0300)?;
        debugger.debug_loop(&mut cpu)?;

        let stdout = spy.get_stdout();
        // println!("{}", stdout);
        assert!(stdout.contains("Usage:"));
        assert!(stdout.contains("disassemble (di)  - disassemble instructions from current PC"));
        Ok(())
    }
}
