use std::fmt;

use crate::{
    engine::decoder::{self, DecodedInstruction},
    CpuError,
};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum CpuTrap {
    ByInstruction(u8),
    ByAddress(u16),
}

impl fmt::Display for CpuTrap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CpuTrap::ByInstruction(op_code_byte) => {
                let decoded = decoder::decode(*op_code_byte).unwrap().get_mnemonic();
                write!(f, "Opcode trap: 0x{:02X} ({})", op_code_byte, decoded)
            }
            CpuTrap::ByAddress(addr) => write!(f, "Invalid address: 0x{:04X}", addr),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrapOutcomeStatus {
    Continue,  // continue execution
    Handled,   // continue execution, skipped instruction, returning trap result
    Stop,      // stop execution
    StopAfter, // stop execution after this instruction
}

pub struct TrapResult {
    pub accumulator: u8,
    pub index_x: u8,
    pub index_y: u8,
    pub status: u8,
}

pub struct TrapOutcome {
    pub status: TrapOutcomeStatus,
    pub result: Option<TrapResult>,
}

#[derive(Debug)]
pub struct Trap {
    pub trap: CpuTrap,
    pub outcome: TrapOutcomeStatus,
}

#[derive(Debug)]
pub struct TrapDoor {
    traps: Vec<Trap>,
}

impl TrapDoor {
    pub fn new() -> TrapDoor {
        let mut td = TrapDoor { traps: vec![] };
        td.add_brk_trap();
        td
    }

    // #[allow(dead_code)]
    pub fn add_trap(&mut self, trap: Trap) {
        self.traps.push(trap);
    }

    #[allow(unused_variables)]
    pub fn pre_execute(
        &self,
        decoded: DecodedInstruction,
        address: u16,
    ) -> Result<TrapOutcome, CpuError> {
        let current = CpuTrap::ByInstruction(decoded.hex_opcode);
        for trap in &self.traps {
            if current == trap.trap {
                return Ok(TrapOutcome {
                    status: TrapOutcomeStatus::StopAfter,
                    result: None,
                });
            }
        }
        Ok(TrapOutcome {
            status: TrapOutcomeStatus::Continue,
            // trap: None,
            result: None,
        })
    }

    fn add_brk_trap(&mut self) {
        self.add_trap(Trap {
            trap: CpuTrap::ByInstruction(0x00), // BRK
            outcome: TrapOutcomeStatus::StopAfter,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_print_cpu_trap() {
        let trap = CpuTrap::ByInstruction(0x00);
        assert_eq!(trap.to_string(), "Opcode trap: 0x00 (BRK)");
        let trap = CpuTrap::ByInstruction(0xFF);
        assert_eq!(trap.to_string(), "Opcode trap: 0xFF (ILL(FF))");

        let trap = CpuTrap::ByAddress(0x0000);
        assert_eq!(trap.to_string(), "Invalid address: 0x0000");
    }

    #[test]
    fn can_trap_on_brk_opcode() -> Result<(), CpuError> {
        let td = TrapDoor::new();
        let decoded = decoder::decode(0x00)?;
        let outcome = td.pre_execute(decoded, 0x0000)?;
        assert_eq!(outcome.status, TrapOutcomeStatus::StopAfter);
        Ok(())
    }
}
