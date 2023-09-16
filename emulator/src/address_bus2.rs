use crate::CpuError;

pub trait AddressBus2 {
    fn fetch_byte_at_pc(&mut self) -> Result<u8, CpuError>;
    fn fetch_word_at_pc(&mut self) -> Result<u16, CpuError>;
    fn set_pc(&mut self, address: u16) -> Result<(), CpuError>;
    fn get_pc(&self) -> u16;
    fn load_program(&mut self, start_addr: u16, program: &[u8]) -> Result<(), CpuError>;
}
