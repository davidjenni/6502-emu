use std::fs::File;
use std::io::*;
use std::path::Path;
use std::{env, io};

// Convert CSV file wilt 6502 op codes info to a match {} map:
fn main() -> io::Result<()> {
    let src_dir_path = env::var("CARGO_MANIFEST_DIR").unwrap();
    let opcodes_file = Path::new(&src_dir_path)
        .join("src")
        .join("engine")
        .join("opcodes-mos6502.csv");

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_file_path = Path::new(&out_dir).join("opcodes_mos6502.rs");

    convert_csv_to_opcodes(&opcodes_file, &out_file_path)?;

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/engine/opcodes-mos6502.csv");
    Ok(())
}

fn convert_csv_to_opcodes(opcodes_file: &Path, out_file_path: &Path) -> io::Result<()> {
    let mut out_file = LineWriter::new(File::create(out_file_path)?);
    println!("cargo:warning=generating: {}...", out_file_path.display());

    out_file.write_all(b"    match opcode{\n")?;

    let mut line_cnt = 0;
    if let Ok(lines) = read_lines(opcodes_file) {
        for line in lines {
            const COLUMNS: usize = 5;
            //   opcode,mnemonic,addressing mode,bytes,cycles,flags
            //   0x69,ADC,IMM,2,2,<flags>
            let [hex_opcode, mnemonic, mode, bytes, cycles] = <[String; COLUMNS]>::try_from(
                line?
                    .split(',')
                    .map(String::from)
                    .take(COLUMNS)
                    .collect::<Vec<String>>(),
            )
            .unwrap();
            // skip header line
            if hex_opcode == *"opcode" {
                continue;
            }

            writeln!(
                out_file,
                //     0x69 => Ok(DecodedInstruction { opcode: OpCode::ADC, mode: AddressingMode::Immediate, execute: execute_adc, extra_bytes: 2, cycles: 2, }),
                "        {} => Ok(DecodedInstruction {{ opcode: OpCode::{}, mode: AddressingMode::{}, execute: execute_{}, extra_bytes: {}, cycles: {}, }}),",
                hex_opcode,
                mnemonic.to_ascii_uppercase(),
                to_addressing_mode(&mode),
                mnemonic.to_ascii_lowercase(),
                bytes.parse::<u8>().unwrap() - 1,   // TODO need better error handling for number parsing
                cycles.parse::<u8>().unwrap(),
            )?;

            line_cnt += 1;
        }
    } else {
        println!("cargo:warning=failed to read i{}", opcodes_file.display());
    }
    writeln!(
        out_file,
        "        _ => Err(CpuError::InvalidOpcode(opcode)),"
    )?;
    out_file.write_all(b"    }\n")?;

    out_file.flush()?;
    println!(
        "cargo:warning=converted {} opcodes from {}",
        line_cnt,
        opcodes_file.display()
    );
    Ok(())
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn to_addressing_mode(csv_mode: &str) -> String {
    match csv_mode {
        "IMP" => "Implied",
        "ACC" => "Accumulator",
        "IMM" => "Immediate",
        "ZP" => "ZeroPage",
        "ZPX" => "ZeroPageX",
        "ZPY" => "ZeroPageY",
        "REL" => "Relative",
        "ABS" => "Absolute",
        "ABSX" => "AbsoluteX",
        "ABSY" => "AbsoluteY",
        "IND" => "Indirect",
        "INDX" => "IndexedXIndirect",
        "INDY" => "IndirectIndexedY",
        _ => panic!("unknown addressing mode: {}", csv_mode),
    }
    .to_string()
}
