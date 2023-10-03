use crate::{cpu_impl::AddressingMode, engine::decoder, CpuError, CpuImpl};

pub fn disassemble(cpu: &CpuImpl, address: u16) -> Result<(String, u16), CpuError> {
    let decoded_instr = decoder::decode(cpu.get_byte_at(address)?)?;
    let mut operand_bytes: [u8; 2] = [0; 2];

    for i in 0..decoded_instr.extra_bytes {
        operand_bytes[i as usize] = cpu.get_byte_at(address.wrapping_add(i as u16 + 1))?;
    }
    // for (i, val) in operand_bytes.iter_mut().enumerate() {
    //     *val = cpu.get_byte_at(address.wrapping_add(i as u16 + 1))?;
    // }
    let disassembly = format!(
        "{:04X} {} {}",
        address,
        decoded_instr.get_mnemonic(),
        get_operand(decoded_instr.mode, address, operand_bytes)?
    )
    .trim_end()
    .to_string();

    Ok((
        disassembly,
        address.wrapping_add((decoded_instr.extra_bytes + 1) as u16),
    ))
}

fn get_operand(
    mode: AddressingMode,
    address: u16,
    extra_bytes: [u8; 2],
) -> Result<String, CpuError> {
    match mode {
        AddressingMode::Implied => Ok("".to_string()),
        AddressingMode::Accumulator => Ok("A".to_string()),
        AddressingMode::Immediate => Ok(format!("#${:02X}", extra_bytes[0])),
        AddressingMode::ZeroPage => Ok(format!("${:02X}", extra_bytes[0])),
        AddressingMode::ZeroPageX => Ok(format!("${:02X},X", extra_bytes[0])),
        AddressingMode::ZeroPageY => Ok(format!("${:02X},Y", extra_bytes[0])),
        AddressingMode::Relative => Ok(format!(
            "${:02X} ({:04X})",
            extra_bytes[0],
            // branch instruction offsets are relative to the next instruction
            addr_from_offset(extra_bytes[0], address + 2)
        )),
        AddressingMode::Absolute => Ok(format!("${:04X}", as_word(extra_bytes))),
        AddressingMode::AbsoluteX => Ok(format!("${:04X},X", as_word(extra_bytes))),
        AddressingMode::AbsoluteY => Ok(format!("${:04X},Y", as_word(extra_bytes))),
        AddressingMode::Indirect => Ok(format!("(${:04X})", as_word(extra_bytes))),
        AddressingMode::IndexedXIndirect => Ok(format!("(${:02X},X)", extra_bytes[0])),
        AddressingMode::IndirectIndexedY => Ok(format!("(${:02X}),Y", extra_bytes[0])),
    }
}

fn as_word(extra_bytes: [u8; 2]) -> u16 {
    (extra_bytes[1] as u16) << 8u8 | (extra_bytes[0] as u16)
}

fn addr_from_offset(offset: u8, pc: u16) -> u16 {
    if offset & 0x80 == 0x80 {
        // two's complement
        pc.wrapping_sub((!offset + 1) as u16)
    } else {
        pc.wrapping_add(offset as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disassemble_legal_opcode() -> Result<(), CpuError> {
        let mut cpu = CpuImpl::new();
        cpu.load_program(
            0x0600,
            &[
                0xA9, 0x42, // LDA #$42
                0x85, 0x0F, // STA $0F
                0x30, 0x04, // BMI swap
                0xF0, 0xFA, // BEQ done
                0x4C, 0x00, 0x06, // JMP start
                0xEA, // NOP
                0x00, // BRK
            ],
            true,
        )?;

        let (mut result, mut next_addr) = disassemble(&cpu, 0x0600)?;
        assert_eq!(result, "0600 LDA #$42");
        assert_eq!(next_addr, 0x0602);

        (result, next_addr) = disassemble(&cpu, next_addr)?;
        assert_eq!(result, "0602 STA $0F");
        assert_eq!(next_addr, 0x0604);

        (result, next_addr) = disassemble(&cpu, next_addr)?;
        assert_eq!(result, "0604 BMI $04 (060A)");
        assert_eq!(next_addr, 0x0606);

        (result, next_addr) = disassemble(&cpu, next_addr)?;
        assert_eq!(result, "0606 BEQ $FA (0602)");
        assert_eq!(next_addr, 0x0608);

        (result, next_addr) = disassemble(&cpu, next_addr)?;
        assert_eq!(result, "0608 JMP $0600");
        assert_eq!(next_addr, 0x060B);

        (result, next_addr) = disassemble(&cpu, next_addr)?;
        assert_eq!(result, "060B NOP");
        assert_eq!(next_addr, 0x060C);

        Ok(())
    }

    #[test]
    fn disassemble_illegal_opcode() -> Result<(), CpuError> {
        let mut cpu = CpuImpl::new();
        cpu.load_program(
            0x0600,
            &[
                0xA9, 0x42, // LDA #$42
                0xFA, 0xFF, // illegal opcode
                0x00, // BRK
            ],
            true,
        )?;

        let (mut result, mut next_addr) = disassemble(&cpu, 0x0600)?;
        assert_eq!(result, "0600 LDA #$42");
        assert_eq!(next_addr, 0x0602);

        (result, next_addr) = disassemble(&cpu, next_addr)?;
        assert_eq!(result, "0602 ILL(FA)");
        assert_eq!(next_addr, 0x0603);

        (result, next_addr) = disassemble(&cpu, next_addr)?;
        assert_eq!(result, "0603 ILL(FF)");
        assert_eq!(next_addr, 0x0604);
        Ok(())
    }
}
