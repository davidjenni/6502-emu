mod args;
mod bin_file;

use std::process;

use anyhow::{Context, Result};
use clap::Parser;

use args::CliArgs;
use mos6502_emulator::{create, CpuRegisterSnapshot, CpuType};

fn main() {
    let args = CliArgs::parse();

    let outcome = match args.command {
        args::Command::Run => run(&args),
        args::Command::Debug => {
            todo!("Debug mode is not implemented yet");
        }
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
    let mut cpu = create(CpuType::MOS6502)?;
    let mut load_addr: Option<u16> = None;

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
    }
    // For now, start address == load address
    Ok(cpu.run(load_addr)?)
}

fn print_snapshot(snapshot: CpuRegisterSnapshot) {
    println!(
        "PC: {:04X}: A: {:02X} X: {:02X} Y: {:02X} S: {:08b} SP: {:04X}",
        snapshot.program_counter,
        snapshot.accumulator,
        snapshot.x_register,
        snapshot.y_register,
        snapshot.status,
        snapshot.stack_pointer
    );
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
