use crate::core::Tty;
use std::os::unix::io::AsRawFd;
use termios::*;

pub struct TermiosCond {
    original_termios: Termios,
    tty: Tty,
}

#[allow(dead_code)]
impl TermiosCond {
    pub fn from_tty(tty: Tty) -> Self {
        let mut termios = Termios::from_fd(tty.as_raw_fd()).unwrap();
        let termioscond = TermiosCond { original_termios: termios,
                                        tty };
        termios.c_cflag &= !(CSIZE | PARENB);
        termios.c_cflag |= CS8;
        termios.c_lflag &= !(ICANON | ECHO | ECHOE | ECHOK | ECHONL | ISIG | IEXTEN);
        termios.c_oflag &= !OPOST;
        termios.c_iflag &= !(IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON);
        termios.c_cc[VMIN] = 0;
        termios.c_cc[VTIME] = 0;
        tcsetattr(termioscond.tty.as_raw_fd(), TCSANOW, &termios).unwrap();
        termioscond
    }
}

impl Drop for TermiosCond {
    fn drop(&mut self) {
        let _ = tcsetattr(self.tty.as_raw_fd(), TCSANOW, &self.original_termios);
    }
}
