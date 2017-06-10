use std::mem;
use std::ptr;
use std::io::{Error, Write, stdout};
use core::{Matrix, TermInfo, Tty};
use std::sync::atomic::{ATOMIC_BOOL_INIT, AtomicBool, Ordering};

use libc;
use unicode_width::UnicodeWidthStr;

static SIGWINCH_RECIEVED: AtomicBool = ATOMIC_BOOL_INIT;

#[derive(Clone,Debug)]
struct CursorCommand {
    pub address: String,
    pub up: String,
    pub down: String,
    pub left: String,
    pub right: String,
    pub clear: String,
    pub delete_char: String,
    pub delete_line: String,
}

#[allow(dead_code)]
impl CursorCommand {
    pub fn from_terminfo(terminfo: &TermInfo) -> Self {
        CursorCommand { address: terminfo.get_string("cup"),
                        up: terminfo.get_string("cuu1"),
                        down: terminfo.get_string("cud1"),
                        left: terminfo.get_string("cub1"),
                        right: terminfo.get_string("cuf1"),
                        clear: terminfo.get_string("clear"),
                        delete_char: terminfo.get_string("dch1"),
                        delete_line: terminfo.get_string("dl1"),
                    }
    }
}

#[derive(Clone,Debug)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
    commands: CursorCommand,
    pub matrix: Matrix,
}

#[allow(dead_code)]
impl Cursor {
    pub fn new(terminfo: &TermInfo, tty: &Tty, cjk: bool) -> Result<Self, Error> {
        Self::setup_sighandler()?;
        Ok(Cursor { x: 0,
                    y: 0,
                    commands: CursorCommand::from_terminfo(terminfo),
                    matrix: Matrix::from_tty(tty, cjk)?, })
    }

    fn setup_sighandler() -> Result<(), Error> {
        let mut sigaction_winch: libc::sigaction = unsafe { mem::zeroed() };
        sigaction_winch.sa_sigaction = sigwinch_handler as libc::size_t;
        let res = unsafe { libc::sigaction(libc::SIGWINCH, &sigaction_winch, ptr::null_mut()) };
        if res != 0 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), Error> {
        self.check_winch()?;
        self.move_to(0, 0)?;
        Self::write_raw_command(&self.commands.clear)?;
        self.matrix.clear()
    }

    pub fn reload(&mut self) -> Result<(), Error> {
        self.check_winch()
    }

    fn check_winch(&mut self) -> Result<(), Error> {
        if SIGWINCH_RECIEVED.load(Ordering::SeqCst) {
            SIGWINCH_RECIEVED.store(false, Ordering::SeqCst);
            let (x, y) = self.get_pos();
            self.matrix.refresh()?;
            self.rewrite_matrix()?;
            self.move_to(x, y)?
        }
        Ok(())
    }

    fn rewrite_matrix(&mut self) -> Result<(), Error> {
        Self::write_raw_command(&self.commands.clear)?;
        let w = self.matrix.range.width;
        for (i, l) in self.matrix.get_lines().iter().enumerate() {
            self.print_fill(0, i, l, w)?
        }
        Ok(())
    }

    fn print_fill(&mut self, x: usize, y: usize, s: &str, w: usize) -> Result<(), Error> {
        let rs = s.replace(|c| ['\n', '\r'].iter().any(|r| c == *r), "");
        let (end, final_s) = self.matrix.put_buffer(x, y, w, &rs);
        stdout().write_fmt(format_args!("{}", final_s))?;
        self.move_to(end, y)?;
        stdout().flush()
    }

    pub fn print_fill_here(&mut self, s: &str, w: usize) -> Result<(), Error> {
        self.check_winch()?;
        let (x, y) = self.get_pos();
        self.print_fill(x, y, s, w)
    }

    pub fn print_here(&mut self, s: &str) -> Result<(), Error> {
        let w = UnicodeWidthStr::width(s);
        self.print_fill_here(s, w)
    }

    pub fn get_pos(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    pub fn move_to(&mut self, x: usize, y: usize) -> Result<(), Error> {
        self.x = *[x, self.matrix.range.width - 1].iter().min().unwrap();
        self.y = *[y, self.matrix.range.height - 1].iter().min().unwrap();
        Self::write_command_with_args(&self.commands.address, &vec![self.y, self.x])
    }

    pub fn move_up(&mut self) -> Result<(), Error> {
        self.y = if self.y == 0 { 0 } else { self.y - 1 };
        Self::write_raw_command(&self.commands.up)
    }

    pub fn move_down(&mut self) -> Result<(), Error> {
        Ok(self.y = if self.y < self.matrix.range.height - 1 {
            Self::write_raw_command(&self.commands.down)?;
            self.y + 1
        } else {
            self.y
        })
    }

    pub fn move_left(&mut self) -> Result<(), Error> {
        self.x = if self.x == 0 { 0 } else { self.x - 1 };
        Self::write_raw_command(&self.commands.left)
    }

    pub fn move_right(&mut self) -> Result<(), Error> {
        Ok(self.x = if self.x < self.matrix.range.width - 1 {
            Self::write_raw_command(&self.commands.right)?;
            self.x + 1
        } else {
            self.x
        })
    }

    pub fn move_home(&mut self) -> Result<(), Error> {
        let y = self.y;
        self.move_to(0, y)
    }

    pub fn move_end(&mut self) -> Result<(), Error> {
        let x = self.matrix.range.width - 1;
        let y = self.y;
        self.move_to(x, y)
    }

    pub fn delete_char(&mut self) -> Result<(), Error> {
        Self::write_raw_command(&self.commands.delete_char)
    }

    pub fn delete_line(&mut self) -> Result<(), Error> {
        Self::write_raw_command(&self.commands.delete_line)
    }

    fn write_raw_command(command: &String) -> Result<(), Error> {
        stdout().write_fmt(format_args!("{}", command))?;
        stdout().flush()
    }

    fn write_command_with_args(command: &String, args: &Vec<usize>) -> Result<(), Error> {
        let s = TermInfo::format(command, args);
        stdout().write_fmt(format_args!("{}", s))?;
        stdout().flush()
    }
}

#[allow(dead_code)]
extern "C" fn sigwinch_handler(_: i32) {
    SIGWINCH_RECIEVED.store(true, Ordering::SeqCst);
}
