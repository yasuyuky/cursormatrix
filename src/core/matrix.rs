use crate::core::Tty;
use libc;
use std::io::Error;
use std::mem;
use std::os::unix::io::AsRawFd;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Matrix {
    tty: Tty,
    pub width: usize,
    pub height: usize,
}

#[allow(dead_code)]
impl Matrix {
    pub fn from_tty(tty: &Tty) -> Result<Self, Error> {
        let ws = Self::load_winsize(tty)?;
        Ok(Self { tty: tty.clone(),
                  width: ws.ws_col as usize,
                  height: ws.ws_row as usize })
    }

    pub fn refresh(&mut self) -> Result<(), Error> {
        let ws = Self::load_winsize(&self.tty)?;
        self.width = ws.ws_col as usize;
        self.height = ws.ws_row as usize;
        Ok(())
    }

    fn load_winsize(tty: &Tty) -> Result<libc::winsize, Error> {
        let mut ws: libc::winsize = unsafe { mem::MaybeUninit::uninit().assume_init() };
        let res = unsafe { libc::ioctl(tty.as_raw_fd(), libc::TIOCGWINSZ, &mut ws) };
        if res != 0 {
            return Err(Error::last_os_error());
        }
        Ok(ws)
    }
}
