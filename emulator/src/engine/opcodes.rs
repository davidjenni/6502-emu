#[allow(dead_code)] // TODO remove
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Copy, Clone)]
pub enum OpCode {
    ADC, // Add Memory to Accumulator with Carry
    AND, // "AND" Memory with Accumulator
    ASL, // Shift Left One Bit (Memory or Accumulator)
    BCC, // Branch on Carry Clear
    BCS, // Branch on Carry Set
    BEQ, // Branch on Equal (Zero Set)
    BIT, // Test Bits in Memory with Accumulator
    BMI, // Branch on Result Minus
    BNE, // Branch on Not Equal (Zero Clear)
    BPL, // Branch on Result Plus
    BRK, // Force Break
    BVC, // Branch on Overflow Clear
    BVS, // Branch on Overflow Set
    CLC, // Clear Carry Flag
    CLD, // Clear Decimal Mode
    CLI, // Clear interrupt Disable Bit
    CLV, // Clear Overflow Flag
    CMP, // Compare Memory to Accumulator
    CPX, // Compare Memory and Index X
    CPY, // Compare Memory and Index Y
    DEC, // Decrement Memory by One
    DEX, // Decrement Index X by One
    DEY, // Decrement Index Y by One
    EOR, // "Exclusive-Or" Memory with Accumulator
    INC, // Increment Memory by One
    INX, // Increment Index X by One
    INY, // Increment Index Y by One
    JMP, // Jump to address
    JSR, // Jump to Subroutine, Saving Return Address
    LDA, // Load Accumulator with Memory
    LDX, // Load Index X with Memory
    LDY, // Load Index Y with Memory
    LSR, // Shift Right One Bit (Memory or Accumulator)
    NOP, // No Operation
    ORA, // "OR" Memory with Accumulator
    PHA, // Push Accumulator to Stack
    PHP, // Push Processor Status on Stack
    PLA, // Pull Accumulator from Stack
    PLP, // Pull Processor Status from Stack
    ROL, // Rotate One Bit Left (Memory or Accumulator)
    ROR, // Rotate One Bit Right (Memory or Accumulator)
    RTI, // Return from Interrupt
    RTS, // Return from Subroutine
    SBC, // Subtract Memory from Accumulator with Borrow
    SEC, // Set Carry Flag
    SED, // Set Decimal Mode
    SEI, // Set Interrupt Disable Status
    STA, // Store Accumulator in Memory
    STX, // Store Index X in Memory
    STY, // Store Index Y in Memory
    TAX, // Transfer Accumulator to Index X
    TAY, // Transfer Accumulator to Index Y
    TSX, // Transfer Stack pointer to Index X
    TXA, // Transfer Index X to Accumulator
    TXS, // Transfer Index X to Stack pointer
    TYA, // Transfer Index Y to Accumulator
}
