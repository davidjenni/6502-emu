use std::io;
use std::io::Write;

pub trait StdIo {
    fn read_line(&mut self, buf: &mut String) -> io::Result<usize>;
    fn write(&mut self, buf: &str) -> io::Result<usize>;
    fn write_err(&mut self, buf: &str) -> io::Result<usize>;
    fn get_writer(&mut self) -> &mut dyn Write;
}

pub struct ConsoleIo {}

impl StdIo for ConsoleIo {
    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        io::stdin().read_line(buf)
    }

    fn write(&mut self, msg: &str) -> io::Result<usize> {
        let cnt = io::stdout().write(msg.as_bytes())?;
        io::stdout().flush()?;
        Ok(cnt)
    }

    fn write_err(&mut self, msg: &str) -> io::Result<usize> {
        let cnt = io::stderr().write(msg.as_bytes())?;
        io::stderr().flush()?;
        Ok(cnt)
    }

    fn get_writer(&mut self) -> &mut dyn Write {
        self
    }
}

impl Write for ConsoleIo {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        io::stdout().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stdout().flush()
    }
}

impl ConsoleIo {
    pub fn default() -> ConsoleIo {
        ConsoleIo {}
    }
}

#[cfg(test)]
pub mod tests {
    use std::io::Write;
    use std::str;

    use super::*;
    use crate::StdIo;

    pub struct Spy {
        stdin: String,
        current_offset: usize,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    }

    impl StdIo for Spy {
        fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
            let lines = self.stdin.split('\n').collect::<Vec<&str>>();
            if lines.len() <= self.current_offset {
                return Ok(0);
            }
            *buf = lines[self.current_offset].to_string().clone();
            self.current_offset += 1;
            Ok(buf.len())
        }

        fn write(&mut self, buf: &str) -> io::Result<usize> {
            self.stdout.write(buf.as_bytes())
        }

        fn write_err(&mut self, buf: &str) -> io::Result<usize> {
            self.stderr.write(buf.as_bytes())
        }

        fn get_writer(&mut self) -> &mut dyn Write {
            self
        }
    }

    impl Write for Spy {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.stdout.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl Spy {
        pub fn new(input: &str) -> Spy {
            Spy {
                stdin: input.to_string(),
                current_offset: 0,
                stdout: vec![],
                stderr: vec![],
            }
        }

        pub fn get_stdout(&self) -> String {
            str::from_utf8(&self.stdout).unwrap().to_string()
        }

        pub fn get_stderr(&self) -> String {
            str::from_utf8(&self.stderr).unwrap().to_string()
        }
    }

    #[test]
    fn test_spy_read_line() {
        let mut spy = Spy::new("foo\nbar\nbaz");
        let mut buf = String::new();
        spy.read_line(&mut buf).unwrap();
        assert_eq!(buf, "foo");
        spy.read_line(&mut buf).unwrap();
        assert_eq!(buf, "bar");
        spy.read_line(&mut buf).unwrap();
        assert_eq!(buf, "baz");
    }

    #[test]
    fn test_spy_stdout() {
        let mut spy = Spy::new("");
        StdIo::write(&mut spy, "foo").unwrap();
        std::io::Write::write(&mut spy, "bar".as_bytes()).unwrap();
        assert_eq!(spy.get_stdout(), "foobar");
    }

    #[test]
    fn test_spy_stderr() {
        let mut spy = Spy::new("");
        spy.write_err("foo").unwrap();
        spy.write_err("bar").unwrap();
        assert_eq!(spy.get_stderr(), "foobar");
    }

    #[test]
    fn test_console_stdout_stderr() {
        let mut io = ConsoleIo::default();
        let written = StdIo::write(&mut io, "foo");
        assert_eq!(written.unwrap(), 3);
        let written = StdIo::write_err(&mut io, "bla");
        assert_eq!(written.unwrap(), 3);
        let written = std::io::Write::write(&mut io, "bar".as_bytes());
        std::io::Write::flush(&mut io).unwrap();
        assert_eq!(written.unwrap(), 3);
    }
}
