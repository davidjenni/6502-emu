#[allow(dead_code)] // TODO remove
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Copy, Clone)]
pub enum OpCode {
    BRK,
    NOP,
    LDA,
    STA,
}
