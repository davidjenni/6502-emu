mod args;
mod bin_file;
mod console_io;
mod dbg_cmd_parser;
mod debugger;

use std::process;
use std::result::Result::Ok;

use anyhow::{Context, Error, Result};
use clap::Parser;
use console_io::StdIo;
use dbg_cmd_parser::DebugCmdError;

use crate::console_io::ConsoleIo;
use crate::debugger::{print_register, Debugger};
use args::CliArgs;
use mos6502_emulator::{create_cpu, Cpu, CpuRegisterSnapshot, CpuType};

fn main() {
    let args = CliArgs::parse();

    let mut console = ConsoleIo::default();
    let main = Main {
        stdio: &mut console,
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

struct Main<'a> {
    stdio: &'a mut dyn StdIo,
}

impl Main<'_> {
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
        self.stdio.write(msg).unwrap();
    }

    fn err(&mut self, msg: &str) {
        self.stdio.write_err(msg).unwrap();
    }

    fn run(&mut self, args: &CliArgs) -> Result<CpuRegisterSnapshot> {
        let (mut cpu, start_addr) = self.init_cpu(args)?;

        anyhow::Ok(cpu.run(Some(start_addr))?)
    }

    fn debug(&mut self, args: &CliArgs) -> Result<CpuRegisterSnapshot> {
        let (mut cpu, start_addr) = self.init_cpu(args)?;

        cpu.set_pc(start_addr)?;
        let mut dbg = Debugger::new(self.stdio);
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
            cpu.load_program(load_addr, &b.data, args.read_only)?;
            self.writeln(
                format!(
                    "Loaded {} bytes at address {:04X}; read-only mem={}",
                    b.data.len(),
                    load_addr,
                    args.read_only
                )
                .as_str(),
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
        print_register(&mut self.stdio.get_writer(), snapshot.clone());
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

impl From<DebugCmdError> for Error {
    fn from(e: DebugCmdError) -> Self {
        anyhow::anyhow!("{}", e)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Ok;
    use std::time;

    use super::*;
    use crate::console_io::tests::Spy;

    fn prepare_main(spy: &'_ mut Spy) -> Main<'_> {
        Main { stdio: spy }
    }

    #[test]
    fn try_main_run() -> Result<(), Error> {
        let args = CliArgs::parse_from(["run"]);

        let mut spy = Spy::new("");
        let m = prepare_main(&mut spy);

        m.try_main(&args)?;

        let stdout = spy.get_stdout();
        println!("{}", stdout);
        assert!(stdout.contains("Start execution at address FFFE"));
        assert_eq!(spy.get_stderr().len(), 0);
        Ok(())
    }

    #[test]
    fn try_main_debug() -> Result<(), Error> {
        let args = CliArgs::parse_from(["r6502.exe", "debug"]);
        let mut spy = Spy::new("quit\n");
        let m = prepare_main(&mut spy);
        m.try_main(&args)?;
        let stdout = spy.get_stdout();
        println!("{}", stdout);
        assert!(stdout.contains("Start execution at address FFFE"));
        assert_eq!(spy.get_stderr().len(), 0);
        Ok(())
    }

    #[test]
    fn try_main_unknown_file_error() -> Result<(), Error> {
        let args = CliArgs::parse_from(["run", "-b=unknown.prg"]);

        let mut spy = Spy::new("");
        let m = prepare_main(&mut spy);

        let r = m.try_main(&args);
        assert!(r.is_err());
        let stderr = spy.get_stderr();
        // println!("ERR: {}", stderr);
        assert!(stderr.contains("Program finished with error:"));
        assert!(stderr.contains("Error loading binary file 'unknown.prg'"));

        let stdout = spy.get_stdout();
        // println!("{}", stdout);
        assert_eq!(stdout.len(), 0);

        Ok(())
    }

    #[test]
    fn try_main_unknown_load_address_error() -> Result<(), Error> {
        let args = CliArgs::parse_from(["run", "-b=tests/assets/simplest.bin"]);

        let mut spy = Spy::new("");
        let mut m = prepare_main(&mut spy);

        let r = m.run(&args);
        assert!(r.is_err());
        let stderr = r.err().unwrap().to_string();
        // println!("ERR: {}", stderr);
        assert!(
            stderr.contains("Load address not specified, and cannot be inferred from file format")
        );

        Ok(())
    }

    #[test]
    fn main_running_simplest_prg() -> Result<(), Error> {
        let args = CliArgs::parse_from(["run", "-b=tests/assets/simplest.prg"]);

        let mut spy = Spy::new("");
        let mut m = prepare_main(&mut spy);

        let snapshot = m.run(&args)?;
        assert_eq!(snapshot.program_counter, 0xFFFE);
        assert_eq!(snapshot.accumulated_instructions, 3);
        assert_eq!(snapshot.accumulated_cycles, 12);
        assert_eq!(snapshot.accumulator, 0x42);

        let stdout = spy.get_stdout();
        println!("{}", stdout);
        assert!(stdout.contains("Start execution at address 0600"));
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

        let mut spy = Spy::new("");
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
