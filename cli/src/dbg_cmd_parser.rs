use std::num::ParseIntError;

use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DebugCommand {
    Continue,
    Disassemble(AddressRange),
    Help,
    Invalid,
    Memory(AddressRange),
    Quit,
    Repeat,
    Step,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AddressRange {
    StartEnd((u16, u16)),
    StartLines((u16, usize)),
    Default,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DebugCmdError {
    InvalidCommand(String),
    InvalidAddressRange(u16),
    CpuError(String),
}

impl std::fmt::Display for DebugCmdError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DebugCmdError::InvalidCommand(s) => write!(f, "Invalid command: {}", s),
            DebugCmdError::InvalidAddressRange(s) => {
                write!(f, "Invalid address: must be between 0 and {}", s)
            }
            DebugCmdError::CpuError(s) => write!(f, "Cpu Error: {}", s),
        }
    }
}

impl From<ParseIntError> for DebugCmdError {
    fn from(_: ParseIntError) -> Self {
        DebugCmdError::InvalidAddressRange(u16::MAX)
    }
}

#[derive(Parser)]
#[grammar = "dbg_cmd.pest"]
struct DbgCmdParser;

pub fn parse_cmd(input: &str) -> Result<DebugCommand, DebugCmdError> {
    if input.is_empty() {
        return Ok(DebugCommand::Repeat);
    }

    let mut parsed_cmd = match DbgCmdParser::parse(Rule::cmd, input) {
        Ok(pairs) => pairs,
        Err(e) => {
            return Err(DebugCmdError::InvalidCommand(e.to_string()));
        }
    };

    let mut dbg_cmd = DebugCommand::Invalid;
    for verb in parsed_cmd.next().unwrap().into_inner() {
        match verb.as_rule() {
            Rule::continue_run => dbg_cmd = DebugCommand::Continue,
            Rule::disassemble => dbg_cmd = DebugCommand::Disassemble(process_addr_range(verb)?),
            Rule::help_verb => dbg_cmd = DebugCommand::Help,
            Rule::memory => dbg_cmd = DebugCommand::Memory(process_addr_range(verb)?),
            Rule::quit_verb => dbg_cmd = DebugCommand::Quit,
            Rule::step_verb => dbg_cmd = DebugCommand::Step,
            Rule::EOI => {}
            _ => unreachable!(),
        };
    }
    Ok(dbg_cmd)
}

fn process_addr_range(pair: Pair<Rule>) -> Result<AddressRange, DebugCmdError> {
    let mut b = AddressRangeBuilder::new();
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::dec_address => b.add_addr(inner_pair.as_str().parse::<u16>()?),
            Rule::hex_address => b.add_addr(u16::from_str_radix(inner_pair.as_str(), 16)?),
            Rule::inclusive => b.is_exclusive_range(false),
            Rule::exclusive => b.is_exclusive_range(true),
            Rule::line_cnt => b.add_line_cnt(inner_pair.as_str().parse::<usize>().unwrap()),
            _ => unreachable!(),
        };
    }
    Ok(b.build())
}

struct AddressRangeBuilder {
    start_addr: Option<u16>,
    end_addr: Option<u16>,
    is_exclusive_range: bool,
    line_cnt: Option<usize>,
}

impl AddressRangeBuilder {
    pub fn new() -> AddressRangeBuilder {
        AddressRangeBuilder {
            start_addr: None,
            end_addr: None,
            is_exclusive_range: true,
            line_cnt: None,
        }
    }

    pub fn add_addr(&mut self, addr: u16) -> &mut AddressRangeBuilder {
        if self.start_addr.is_none() {
            self.start_addr = Some(addr);
        } else {
            self.end_addr = Some(addr);
        }
        self
    }

    pub fn add_line_cnt(&mut self, line_cnt: usize) -> &mut AddressRangeBuilder {
        self.line_cnt = Some(line_cnt);
        self
    }

    pub fn is_exclusive_range(&mut self, is_exclusive: bool) -> &mut AddressRangeBuilder {
        self.is_exclusive_range = is_exclusive;
        self
    }

    pub fn build(&mut self) -> AddressRange {
        if self.start_addr.is_none() {
            return AddressRange::Default;
        }
        if self.line_cnt.is_some() {
            return AddressRange::StartLines((self.start_addr.unwrap(), self.line_cnt.unwrap()));
        }
        if self.end_addr.is_none() {
            return AddressRange::StartEnd((
                self.start_addr.unwrap(),
                self.start_addr.unwrap() + 16,
            ));
        }
        if self.is_exclusive_range {
            self.end_addr = Some(self.end_addr.unwrap().wrapping_sub(1));
        }
        AddressRange::StartEnd((self.start_addr.unwrap(), self.end_addr.unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ======== disassemble commands
    #[test]
    fn parse_disassemble_no_args() -> Result<(), DebugCmdError> {
        let cmd = parse_cmd("dIsasSemble")?;
        assert_eq!(DebugCommand::Disassemble(AddressRange::Default), cmd);
        Ok(())
    }

    // #[test]
    // fn parse_disassemble_pc() -> Result<(), DbgCmdError> {
    //     let cmd = parse_cmd("dI pC")?;
    //     assert_eq!(DbgCmd::Disassemble(AddressRange::Default), cmd);
    //     Ok(())
    // }

    #[test]
    fn parse_disassemble_start_addr_only_dec() -> Result<(), DebugCmdError> {
        let cmd = parse_cmd("di 1234")?;
        assert_eq!(
            DebugCommand::Disassemble(AddressRange::StartEnd((1234, 1250))),
            cmd
        );
        Ok(())
    }

    #[test]
    fn parse_disassemble_start_addr_only_hex() -> Result<(), DebugCmdError> {
        let cmd = parse_cmd("di 0Xa0")?;
        assert_eq!(
            DebugCommand::Disassemble(AddressRange::StartEnd((0xA0, 0xB0))),
            cmd
        );
        Ok(())
    }

    #[test]
    fn parse_disassemble_start_addr_lines_hex() -> Result<(), DebugCmdError> {
        let cmd = parse_cmd("dI 0x1234,22")?;
        assert_eq!(
            DebugCommand::Disassemble(AddressRange::StartLines((0x1234, 22))),
            cmd
        );
        Ok(())
    }

    // ======== memory commands
    #[test]
    fn parse_memory_no_args() -> Result<(), DebugCmdError> {
        let cmd = parse_cmd("MeMory")?;
        assert_eq!(DebugCommand::Memory(AddressRange::Default), cmd);
        Ok(())
    }

    #[test]
    fn parse_memory_start_addr_only_dec() -> Result<(), DebugCmdError> {
        let cmd = parse_cmd("m 1234")?;
        assert_eq!(
            DebugCommand::Memory(AddressRange::StartEnd((1234, 1250))),
            cmd
        );
        Ok(())
    }

    #[test]
    fn parse_memory_start_addr_only_hex() -> Result<(), DebugCmdError> {
        let cmd = parse_cmd("m 0Xa0")?;
        assert_eq!(
            DebugCommand::Memory(AddressRange::StartEnd((0xA0, 0xB0))),
            cmd
        );
        Ok(())
    }

    #[test]
    fn parse_memory_start_addr_end_excl_range_hex() -> Result<(), DebugCmdError> {
        let cmd = parse_cmd("m 0X1234 ..0x1240")?;
        assert_eq!(
            DebugCommand::Memory(AddressRange::StartEnd((0x1234, 0x123F))),
            cmd
        );
        Ok(())
    }

    #[test]
    fn parse_memory_start_addr_lines_hex() -> Result<(), DebugCmdError> {
        let cmd = parse_cmd("M 0x1234,22")?;
        assert_eq!(
            DebugCommand::Memory(AddressRange::StartLines((0x1234, 22))),
            cmd
        );
        Ok(())
    }

    // ======== error handling
    #[test]
    fn parse_error_unknown_cmd() -> Result<(), DebugCmdError> {
        let res = parse_cmd("nonsense ");
        assert!(res.is_err());
        let err = res.err().unwrap();
        match err {
            DebugCmdError::InvalidCommand(_) => {}
            _ => unreachable!("Unexpected error type"),
        };
        assert!(err.to_string().contains("Invalid command:"));
        assert!(err.to_string().contains("nonsense"));

        Ok(())
    }

    #[test]
    fn parse_error_bad_address_range() -> Result<(), DebugCmdError> {
        let res = parse_cmd("m  0x1234, ");
        assert!(res.is_err());
        let err = res.err().unwrap();
        match err {
            DebugCmdError::InvalidCommand(_) => {}
            _ => unreachable!("Unexpected error type"),
        };
        assert!(err.to_string().contains("Invalid command:"));
        assert!(err.to_string().contains("expected line_cnt"));

        Ok(())
    }

    #[test]
    fn parse_error_bad_hex_address() -> Result<(), DebugCmdError> {
        let res = parse_cmd("m 0x0EFG");
        assert!(res.is_err());
        let err = res.err().unwrap();
        match err {
            DebugCmdError::InvalidCommand(_) => {}
            _ => unreachable!("Unexpected error type"),
        };
        assert!(err.to_string().contains("Invalid command:"));
        assert!(
            err.to_string()
                .contains("expected EOI, exclusive, or inclusive")
        );

        Ok(())
    }

    #[test]
    fn parse_error_bad_range_indicator() -> Result<(), DebugCmdError> {
        let res = parse_cmd("m 0x0EF.=1234");
        assert!(res.is_err());
        let err = res.err().unwrap();
        match err {
            DebugCmdError::InvalidCommand(_) => {}
            _ => unreachable!("Unexpected error type"),
        };
        println!("err: {}", err);
        assert!(err.to_string().contains("Invalid command:"));
        assert!(
            err.to_string()
                .contains("expected EOI, exclusive, or inclusive")
        );

        Ok(())
    }

    #[test]
    fn parse_error_dec_address_overflow() -> Result<(), DebugCmdError> {
        let res = parse_cmd("m 66000");
        assert!(res.is_err());
        let err = res.err().unwrap();
        match err {
            DebugCmdError::InvalidAddressRange(_) => {}
            _ => unreachable!("Unexpected error type"),
        };
        println!("err: {}", err);
        assert!(err.to_string().contains("Invalid address:"));
        assert!(err.to_string().contains("between 0 and 65535"));

        Ok(())
    }

    #[test]
    fn parse_error_hex_address_overflow() -> Result<(), DebugCmdError> {
        let res = parse_cmd("m 0x1E234");
        assert!(res.is_err());
        let err = res.err().unwrap();
        match err {
            DebugCmdError::InvalidAddressRange(_) => {}
            _ => unreachable!("Unexpected error type"),
        };
        println!("err: {}", err);
        assert!(err.to_string().contains("Invalid address:"));
        assert!(err.to_string().contains("between 0 and 65535"));

        Ok(())
    }

    // ======== simple commands
    #[test]
    fn parse_continue() -> Result<(), DebugCmdError> {
        let cmd = parse_cmd("  c ")?;
        assert_eq!(DebugCommand::Continue, cmd);
        Ok(())
    }

    #[test]
    fn parse_help() -> Result<(), DebugCmdError> {
        let cmd = parse_cmd("  help ")?;
        assert_eq!(DebugCommand::Help, cmd);
        Ok(())
    }

    #[test]
    fn parse_quit() -> Result<(), DebugCmdError> {
        let cmd = parse_cmd("  qUit ")?;
        assert_eq!(DebugCommand::Quit, cmd);
        Ok(())
    }

    #[test]
    fn parse_repeat() -> Result<(), DebugCmdError> {
        let cmd = parse_cmd("")?;
        assert_eq!(DebugCommand::Repeat, cmd);
        Ok(())
    }

    #[test]
    fn parse_step() -> Result<(), DebugCmdError> {
        let cmd = parse_cmd("  S ")?;
        assert_eq!(DebugCommand::Step, cmd);
        Ok(())
    }
}
