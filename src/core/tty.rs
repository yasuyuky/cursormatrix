use std::fs::{File, OpenOptions};
use std::io::{Error, Read};
use std::os::unix::io::{AsRawFd, RawFd};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Tty {
    file: File,
}

#[allow(dead_code)]
impl Tty {
    pub fn new() -> Result<Self, std::io::Error> {
        Ok(Tty { file: OpenOptions::new().write(true).read(true).open("/dev/tty")? })
    }

    pub fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, Error> {
        self.file.read_to_end(buf)
    }
}

impl AsRawFd for Tty {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}
