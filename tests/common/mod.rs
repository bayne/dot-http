use std::io;
use std::io::Write;
use std::str::from_utf8;
use tempfile::{NamedTempFile, TempPath};

pub fn create_file(contents: &str) -> TempPath {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "{}", contents).unwrap();
    file.into_temp_path()
}

pub struct DebugWriter(pub String);
impl Write for DebugWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let DebugWriter(inner) = self;
        let buf = from_utf8(buf).unwrap();
        inner.push_str(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
