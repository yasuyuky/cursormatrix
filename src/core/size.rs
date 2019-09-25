use crate::core::Tty;
use libc;
use std::io::Error;
use std::mem;
use std::os::unix::io::AsRawFd;

#[derive(Clone, Debug)]
pub struct TermSize {
    tty: Tty,
    pub width: usize,
    pub height: usize,
}

#[allow(dead_code)]
impl TermSize {
    pub fn from_tty(tty: &Tty) -> Result<TermSize, Error> {
        let mut ws: libc::winsize = unsafe { mem::uninitialized() };
        let res = unsafe { libc::ioctl(tty.as_raw_fd(), libc::TIOCGWINSZ, &mut ws) };
        if res != 0 {
            return Err(Error::last_os_error());
        }
        Ok(TermSize { tty: tty.clone(),
                      width: ws.ws_col as usize,
                      height: ws.ws_row as usize })
    }

    pub fn refresh(&mut self) -> Result<(), Error> {
        let mut ws: libc::winsize = unsafe { mem::uninitialized() };
        let res = unsafe { libc::ioctl(self.tty.as_raw_fd(), libc::TIOCGWINSZ, &mut ws) };
        if res != 0 {
            return Err(Error::last_os_error());
        }
        self.width = ws.ws_col as usize;
        self.height = ws.ws_row as usize;
        Ok(())
    }
}
