use std::io;

use crate::args::FileFormat;

pub struct BinFileBuffer {
    pub load_addr: Option<u16>,
    pub data: Vec<u8>,
}

pub fn load_program(file: &str, format: Option<FileFormat>) -> Result<BinFileBuffer, io::Error> {
    let format = resolve_format(file, format)?;
    let mut file = std::fs::File::open(file)?;
    read_program(&mut file, format)
}

fn resolve_format(
    file: &str,
    candidate_format: Option<FileFormat>,
) -> Result<FileFormat, io::Error> {
    let ext = std::path::Path::new(file)
        .extension()
        .unwrap()
        .to_str()
        .unwrap();
    let format = match ext {
        "bin" => Some(FileFormat::Bin),
        "prg" => Some(FileFormat::Prg),
        _ => None,
    };

    // error out if format cannot be determined:
    if candidate_format.is_none() && format.is_none() {
        let msg = format_args!(
            "File format not specified, and cannot be inferred from file extension '{}'",
            ext
        )
        .to_string();
        return Err(io::Error::new(io::ErrorKind::InvalidInput, msg));
    }
    // precedence to user-specified format:
    Ok(candidate_format.unwrap_or(format.unwrap()))
}

fn read_program(
    file: &mut dyn io::Read,
    format: FileFormat,
) -> Result<BinFileBuffer, std::io::Error> {
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    match format {
        FileFormat::Bin => Ok(BinFileBuffer {
            load_addr: None,
            data: buffer,
        }),
        FileFormat::Prg => Ok(BinFileBuffer {
            load_addr: Some(u16::from_le_bytes([buffer[0], buffer[1]])),
            data: buffer[2..].to_vec(),
        }),
    }
}

#[allow(dead_code)]
pub trait BinFile {
    fn read(&mut self, format: FileFormat) -> Result<BinFileBuffer, std::io::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_format_user_precedence() -> Result<(), io::Error> {
        let file = "foo.prg";
        let format = Some(FileFormat::Bin);
        let result = resolve_format(file, format)?;
        assert_eq!(result, FileFormat::Bin);
        Ok(())
    }

    #[test]
    fn resolve_format_extension_only() -> Result<(), io::Error> {
        let file = "foo.prg";
        let result = resolve_format(file, None)?;
        assert_eq!(result, FileFormat::Prg);
        Ok(())
    }

    #[test]
    fn resolve_format_unknown_extension_only() -> Result<(), io::Error> {
        let file = "foo.bla";
        let result = resolve_format(file, None);
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err
            .to_string()
            .contains("cannot be inferred from file extension 'bla'"));
        Ok(())
    }
}
