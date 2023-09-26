mod args;
mod bin_file;
mod debug_loop;

use std::process;
use std::result::Result::Ok;

use anyhow::{Context, Error, Result};
use clap::Parser;

use args::CliArgs;
use debug_loop::{show_usage, DebuggerCommand, DebuggerLoop};
use mos6502_emulator::{create, CpuController, CpuRegisterSnapshot, CpuType};

fn main() {
    let args = CliArgs::parse();

    let outcome = match args.command {
        args::Command::Run => run(&args),
        args::Command::Debug => debug(&args),
    };
    match outcome {
        Ok(snapshot) => {
            println!();
            print_snapshot(snapshot);
            println!("done.");
        }
        Err(e) => {
            eprintln!("Program finished with error:\n {:#}", e);
            process::exit(1);
        }
    }
}

fn run(args: &CliArgs) -> Result<CpuRegisterSnapshot> {
    let (mut cpu, load_addr) = create_cpu(args)?;

    // For now, start address == load address
    anyhow::Ok(cpu.run(Some(load_addr))?)
}

fn debug(args: &CliArgs) -> Result<CpuRegisterSnapshot> {
    let (mut cpu, load_addr) = create_cpu(args)?;

    let mut dbg = cpu.as_debugger();
    dbg.set_pc(load_addr)?;

    print_register(dbg.get_register_snapshot());
    let mut dbg_loop = DebuggerLoop::new();
    loop {
        let cmd = dbg_loop.get_user_input();
        match cmd {
            DebuggerCommand::Step => {
                let snapshot = dbg.step()?;
                print_register(snapshot);
            }
            DebuggerCommand::Continue => {
                let snapshot = dbg.run(None)?;
                print_register(snapshot);
            }
            DebuggerCommand::Disassemble => {
                let lines = dbg.disassemble(load_addr, 10)?;
                for line in lines {
                    println!("{}", line);
                }
            }
            DebuggerCommand::Invalid => {
                show_usage();
            }
            DebuggerCommand::Quit => {
                println!("Exiting...");
                break;
            }
            DebuggerCommand::Repeat => panic!("not reachable"),
        }
    }
    anyhow::Ok(dbg.get_register_snapshot())
}

fn create_cpu(args: &CliArgs) -> Result<(Box<dyn CpuController>, u16), Error> {
    let mut cpu = create(CpuType::MOS6502)?;
    let load_addr: Option<u16>;

    if args.binary.is_some() {
        let file_name = args.binary.as_ref().unwrap();
        let b = bin_file::load_program(file_name, None)
            .with_context(|| format!("Error loading binary file '{}'", file_name))?;
        load_addr = b.start_addr.or(args.load_address);
        if load_addr.is_none() {
            return Err(anyhow::anyhow!(
                "Start address not specified, and cannot be inferred from file format"
            ));
        }
        cpu.load_program(load_addr.unwrap(), &b.data)?;
        println!(
            "Loaded {} bytes at address {:04X}",
            b.data.len(),
            load_addr.unwrap()
        );
    } else {
        println!("No binary file specified, running empty program with single BRK instruction");
        load_addr = Some(0xFFFE); // RESET vector
    }
    Ok((cpu, load_addr.unwrap()))
}

fn print_register(snapshot: CpuRegisterSnapshot) {
    println!(
        "PC: {:04X}: A: {:02X} X: {:02X} Y: {:02X} S: {:08b} SP: {:04X}",
        snapshot.program_counter,
        snapshot.accumulator,
        snapshot.x_register,
        snapshot.y_register,
        snapshot.status,
        snapshot.stack_pointer
    );
}
fn print_snapshot(snapshot: CpuRegisterSnapshot) {
    print_register(snapshot.clone());
    println!(
        "Instructions: {}; Cycles: {}; Clock speed: {:.3} MHz",
        snapshot.accumulated_instructions,
        snapshot.accumulated_cycles,
        snapshot.approximate_clock_speed / 1_000_000.0
    );
    println!(
        "Program finished after {} μs:",
        snapshot.elapsed_time.as_micros()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn running_simplest_prg() {
        let args = CliArgs::parse_from(["run", "-b=tests/assets/simplest.prg"]);
        let snapshot = run(&args).unwrap();
        assert_eq!(snapshot.program_counter, 0xFFFE);
        assert_eq!(snapshot.accumulated_instructions, 3);
        assert_eq!(snapshot.accumulated_cycles, 12);
        assert_eq!(snapshot.accumulator, 0x42);
    }
}
