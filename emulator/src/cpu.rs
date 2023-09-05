use crate::address_bus::AddressBus;
use crate::status_register::StatusRegister;

#[derive(Debug)]
pub struct Cpu {
    pub accumulator: u8,
    pub index_x: u8,
    pub index_y: u8,
    pub status: StatusRegister,
    pub address_bus: AddressBus,
}
