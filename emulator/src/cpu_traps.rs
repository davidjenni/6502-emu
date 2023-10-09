use std::fmt;

use crate::{
    engine::decoder::{self, DecodedInstruction},
    CpuError,
};

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
            CpuTrap::ByAddress(addr) => write!(f, "Address trap: 0x{:04X}", addr),
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

#[derive(Debug, Clone, PartialEq)]
pub struct TrapResult {
    pub accumulator: u8,
    pub index_x: u8,
    pub index_y: u8,
    pub status: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrapOutcome {
    pub status: TrapOutcomeStatus,
    pub triggered_by: Option<CpuTrap>,
    pub result: Option<TrapResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Trap {
    pub cpu_trap: CpuTrap,
    pub requested_outcome: TrapOutcomeStatus,
}

#[derive(Debug)]
pub struct TrapDoor {
    address_traps: Vec<Trap>,
    opcode_traps: Vec<Trap>,
}

impl TrapDoor {
    pub fn new() -> TrapDoor {
        let mut td = TrapDoor {
            address_traps: vec![],
            opcode_traps: vec![],
        };
        td.add_brk_trap();
        td
    }

    #[allow(dead_code)]
    pub fn add_addr_trap(&mut self, trap: Trap) {
        self.address_traps.push(trap);
    }

    pub fn add_opcode_trap(&mut self, trap: Trap) {
        self.opcode_traps.push(trap);
    }

    pub fn pre_execute(
        &self,
        decoded: DecodedInstruction,
        address: u16,
    ) -> Result<TrapOutcome, CpuError> {
        // precedence order: by address, then by instruction
        let try_address = CpuTrap::ByAddress(address);
        // TODO: looping over all traps is ok for small number of traps
        for trap in &self.address_traps {
            if try_address == trap.cpu_trap {
                return Ok(TrapOutcome {
                    // TODO: stop is correct for break points, but need to rethink when adding trap handlers
                    // for Handled: set result and Continue
                    status: trap.requested_outcome,
                    triggered_by: Some(try_address),
                    result: None,
                });
            }
        }

        let try_op_code = CpuTrap::ByInstruction(decoded.hex_opcode);
        for trap in &self.opcode_traps {
            if try_op_code == trap.cpu_trap {
                return Ok(TrapOutcome {
                    status: TrapOutcomeStatus::StopAfter,
                    triggered_by: Some(try_op_code),
                    result: None,
                });
            }
        }
        Ok(TrapOutcome {
            status: TrapOutcomeStatus::Continue,
            triggered_by: None,
            result: None,
        })
    }

    fn add_brk_trap(&mut self) {
        self.add_opcode_trap(Trap {
            cpu_trap: CpuTrap::ByInstruction(0x00), // BRK
            requested_outcome: TrapOutcomeStatus::StopAfter,
        });
    }

    #[allow(dead_code)]
    pub fn add_address_trap(&mut self, address: u16) {
        self.add_addr_trap(Trap {
            cpu_trap: CpuTrap::ByAddress(address), // BRK
            requested_outcome: TrapOutcomeStatus::Stop,
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

        let trap = CpuTrap::ByAddress(0x0400);
        assert_eq!(trap.to_string(), "Address trap: 0x0400");
    }

    #[test]
    fn can_trap_on_brk_opcode() -> Result<(), CpuError> {
        let td = TrapDoor::new();
        let decoded = decoder::decode(0x00)?;
        let outcome = td.pre_execute(decoded, 0x0000)?;
        assert_eq!(outcome.status, TrapOutcomeStatus::StopAfter);
        assert_eq!(outcome.triggered_by, Some(CpuTrap::ByInstruction(0x00)));
        Ok(())
    }

    #[test]
    fn can_trap_on_address() -> Result<(), CpuError> {
        let mut td = TrapDoor::new();
        let decoded = decoder::decode(0xA9)?;
        let address = 0x0400;
        td.add_address_trap(address);
        // try non-matching address
        let outcome = td.pre_execute(decoded, 0x1234)?;
        assert_eq!(outcome.status, TrapOutcomeStatus::Continue);
        assert_eq!(outcome.triggered_by, None);

        let decoded = decoder::decode(0x85)?;
        let outcome = td.pre_execute(decoded, address)?;
        assert_eq!(outcome.status, TrapOutcomeStatus::Stop);
        assert_eq!(outcome.triggered_by, Some(CpuTrap::ByAddress(address)));
        Ok(())
    }

    #[test]
    fn can_trap_precedence() -> Result<(), CpuError> {
        let mut td = TrapDoor::new();
        let decoded = decoder::decode(0x00)?;
        let address = 0x0400;
        td.add_address_trap(address);
        // try non-matching address
        let outcome = td.pre_execute(decoded.clone(), address)?;
        println!("outcome: {:?}", outcome);
        assert_eq!(outcome.status, TrapOutcomeStatus::Stop);
        assert_eq!(outcome.triggered_by, Some(CpuTrap::ByAddress(address)));

        let outcome = td.pre_execute(decoded, address + 1)?;
        assert_eq!(outcome.status, TrapOutcomeStatus::StopAfter);
        assert_eq!(outcome.triggered_by, Some(CpuTrap::ByInstruction(0x00)));
        Ok(())
    }
}
