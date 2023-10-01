mod args;
mod bin_file;
mod debugger;

use std::io;
use std::process;
use std::result::Result::Ok;

use anyhow::{Context, Error, Result};
use clap::Parser;

use crate::debugger::{print_register, Debugger};
use args::CliArgs;
use mos6502_emulator::{create_cpu, Cpu, CpuRegisterSnapshot, CpuType};

fn main() {
    let args = CliArgs::parse();

    let main = Main {
        stdin: io::BufReader::new(io::stdin()),
        stdout: io::stdout(),
        stderr: io::stderr(),
    };
    let outcome = main.try_main(&args);

    match outcome {
        Ok(()) => {
            process::exit(0);
        }
        Err(_) => {
            process::exit(1);
        }
    }
}

struct Main<R, W, E> {
    #[allow(dead_code)]
    stdin: R,
    stdout: W,
    stderr: E,
}

impl<R, W, E> Main<R, W, E>
where
    R: io::BufRead,
    W: io::Write,
    E: io::Write,
{
    pub fn try_main(mut self, args: &CliArgs) -> Result<()> {
        let outcome = match args.command {
            args::Command::Run => self.run(args),
            args::Command::Debug => self.debug(args),
        };
        match outcome {
            Ok(snapshot) => {
                self.writeln("");
                self.print_snapshot(snapshot);
                self.writeln("done.");
                Ok(())
            }
            Err(e) => {
                self.err(format!("Program finished with error:\n {:#}", e).as_str());
                Err(e)
            }
        }
    }

    fn writeln(&mut self, msg: &str) {
        self.write(msg);
        self.write("\n");
    }

    fn write(&mut self, msg: &str) {
        self.stdout.write_all(msg.as_bytes()).unwrap();
    }

    fn err(&mut self, msg: &str) {
        self.stderr.write_all(msg.as_bytes()).unwrap();
    }

    fn run(&mut self, args: &CliArgs) -> Result<CpuRegisterSnapshot> {
        let (mut cpu, start_addr) = self.init_cpu(args)?;

        anyhow::Ok(cpu.run(Some(start_addr))?)
    }

    fn debug(&mut self, args: &CliArgs) -> Result<CpuRegisterSnapshot> {
        let (mut cpu, start_addr) = self.init_cpu(args)?;

        cpu.set_pc(start_addr)?;
        let mut dbg = Debugger::<R, W, E>::new();
        dbg.debug_loop(&mut cpu)?;

        anyhow::Ok(cpu.get_register_snapshot())
    }

    fn init_cpu(&mut self, args: &CliArgs) -> Result<(Box<dyn Cpu>, u16), Error> {
        let mut cpu = create_cpu(CpuType::MOS6502)?;
        let load_addr: Option<u16>;

        if args.binary.is_some() {
            let file_name = args.binary.as_ref().unwrap();
            let b = bin_file::load_program(file_name, None)
                .with_context(|| format!("Error loading binary file '{}'", file_name))?;
            load_addr = b.load_addr.or(args.load_address);
            if load_addr.is_none() {
                return Err(anyhow::anyhow!(
                    "Load address not specified, and cannot be inferred from file format"
                ));
            }
            let load_addr = load_addr.unwrap();
            cpu.load_program(load_addr, &b.data)?;
            self.writeln(
                format!("Loaded {} bytes at address {:04X}", b.data.len(), load_addr).as_str(),
            );
        } else {
            self.writeln(
                "No binary file specified, running empty program with single BRK instruction",
            );
            load_addr = Some(0xFFFE); // RESET vector
        }

        let start_addr = if args.start_address.is_some() {
            args.start_address.unwrap()
        } else {
            load_addr.unwrap()
        };
        self.writeln(format!("Start execution at address {:04X}", start_addr).as_str());
        Ok((cpu, start_addr))
    }

    fn print_snapshot(&mut self, snapshot: CpuRegisterSnapshot) {
        print_register(&mut self.stdout, snapshot.clone());
        self.writeln(
            format!(
                "Instructions: {}; Cycles: {}; Clock speed: {:.3} MHz",
                snapshot.accumulated_instructions,
                snapshot.accumulated_cycles,
                snapshot.approximate_clock_speed / 1_000_000.0,
            )
            .as_str(),
        );
        self.writeln(
            format!(
                "Program finished after {} μs:",
                snapshot.elapsed_time.as_micros(),
            )
            .as_str(),
        );
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Ok;
    use std::str;
    use std::time;

    use super::*;

    struct Spy<'a> {
        stdin: &'a [u8],
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    }

    impl<'a> Spy<'a> {
        pub fn default() -> Spy<'a> {
            Spy {
                stdin: "".as_bytes(),
                stdout: vec![],
                stderr: vec![],
            }
        }

        #[allow(dead_code)]
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

        pub fn get_stderr(&self) -> String {
            str::from_utf8(&self.stderr).unwrap().to_string()
        }
    }

    fn prepare_main<'a>(spy: &'a mut Spy) -> Main<&'a [u8], &'a mut Vec<u8>, &'a mut Vec<u8>> {
        Main {
            stdin: spy.stdin,
            stdout: &mut spy.stdout,
            stderr: &mut spy.stderr,
        }
    }

    #[test]
    fn main_running_simplest_prg() -> Result<(), Error> {
        let args = CliArgs::parse_from(["run", "-b=tests/assets/simplest.prg"]);

        let mut spy = Spy::default();
        let mut m = prepare_main(&mut spy);

        let snapshot = m.run(&args)?;
        assert_eq!(snapshot.program_counter, 0xFFFE);
        assert_eq!(snapshot.accumulated_instructions, 3);
        assert_eq!(snapshot.accumulated_cycles, 12);
        assert_eq!(snapshot.accumulator, 0x42);

        assert!(spy.get_stdout().contains("Start execution at address 0600"));
        assert_eq!(spy.get_stderr().len(), 0);
        Ok(())
    }

    #[test]
    fn main_print_snapshot() -> Result<(), Error> {
        #[allow(unused_variables)]
        let snapshot = CpuRegisterSnapshot {
            accumulator: 0x42,
            x_register: 0x43,
            y_register: 0x44,
            stack_pointer: 0x45,
            program_counter: 0x4711,
            status: 0x47,
            elapsed_time: time::Duration::from_micros(123456),
            accumulated_cycles: 123456,
            accumulated_instructions: 123456,
            approximate_clock_speed: 123456.0,
        };

        let mut spy: Spy = Spy::default();
        let mut m = prepare_main(&mut spy);
        m.print_snapshot(snapshot);

        let stdout = spy.get_stdout();
        assert!(stdout.contains("PC: 4711"));
        assert!(stdout.contains("A: 42"));
        assert!(stdout.contains("Instructions: 123456"));

        Ok(())
    }

    #[test]
    fn main_help() {
        let args = CliArgs::parse_from(["-h"]);
        assert!(args.format.is_none());
        assert_eq!(args.command, args::Command::Run);
    }
}
