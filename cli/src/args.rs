use clap::{Parser, ValueEnum};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Command {
    Run,
    Debug,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum FileFormat {
    /// Plain binary with no header, little endian byte order
    Bin,
    /// Like a bin file, but with a 16 byte header that indicates the load address
    Prg,
    // Mon, // WozMon like format: text file with hex bytes, and a 16 byte header that indices the start address
    // Mos, // MOS object file format https://en.wikipedia.org/wiki/MOS_Technology_file_format
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    #[arg(value_enum, ignore_case = true, default_value = "run")]
    pub command: Command,

    #[arg(short, long)]
    /// Path to binary file to load and run
    pub binary: Option<String>,

    #[arg(value_enum, ignore_case = true, short)]
    /// File format of the binary file to load
    pub format: Option<FileFormat>,

    #[arg(short, long, required = false, value_parser = clap::value_parser!(u16))]
    /// Load address (u16) for binary to be loaded to (inferred for .prg); if no start_addr it is also used as start address
    pub load_address: Option<u16>,

    #[arg(short, long, required = false, value_parser = clap::value_parser!(u16))]
    /// Start address (u16) for binary to be started with
    pub start_address: Option<u16>,
}
