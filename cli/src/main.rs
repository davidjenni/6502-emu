mod args;

use args::CliArgs;
use clap::Parser;

use mos6502_emulator::{create, CpuError, CpuRegisterSnapshot, CpuType};

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
            println!("Program finished with error: {}", e);
        }
    }
}

fn run(args: &CliArgs) -> Result<CpuRegisterSnapshot, CpuError> {
    let mut cpu = create(CpuType::MOS6502)?;
    let start_addr: Option<u16> = None;
    if args.file.is_some() {
        todo!("Loading binary files is not implemented yet");
        // todo!("Assign start_addr from binary file");
    }
    cpu.run(start_addr)
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
        "Instructions: {} Cycles: {} Clock speed: {:.3} MHz",
        snapshot.accumulated_instructions,
        snapshot.accumulated_cycles,
        snapshot.approximate_clock_speed / 1_000_000.0
    );
    println!(
        "Program finished after {} μs:",
        snapshot.elapsed_time.as_micros()
    );
}
