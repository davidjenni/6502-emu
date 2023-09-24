use clap::{Parser, ValueEnum};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Command {
    Run,
    Debug,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    #[arg(short, long, required = false)]
    /// Path to binary file to load and run
    pub file: Option<String>,

    #[arg(value_enum, default_value = "run")]
    pub command: Command,
}
