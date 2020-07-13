use crate::core::TermInfo;
use std::io::{stdout, Error, Write};

#[derive(Clone, Debug)]
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
                        delete_line: terminfo.get_string("dl1") }
    }
}

#[derive(Clone, Debug)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
    commands: CursorCommand,
}

#[allow(dead_code)]
impl Cursor {
    pub fn new(terminfo: &TermInfo) -> Result<Self, Error> {
        Ok(Cursor { x: 0,
                    y: 0,
                    commands: CursorCommand::from_terminfo(terminfo) })
    }

    pub fn clear(&mut self) -> Result<(), Error> {
        self.move_to((0, 0))?;
        Self::write_raw_command(&self.commands.clear)
    }

    pub fn print(&mut self, s: &str) -> Result<(), Error> {
        stdout().write_fmt(format_args!("{}", s))?;
        stdout().flush()
    }

    pub fn get_pos(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    pub fn move_to(&mut self, (x, y): (usize, usize)) -> Result<(), Error> {
        self.x = x;
        self.y = y;
        Self::write_command_with_args(&self.commands.address, &[self.y, self.x])
    }

    pub fn move_up(&mut self) -> Result<(), Error> {
        self.y = if self.y == 0 { 0 } else { self.y - 1 };
        Self::write_raw_command(&self.commands.up)
    }

    pub fn move_down(&mut self, ylimit: usize) -> Result<(), Error> {
        if self.y < ylimit {
            Self::write_raw_command(&self.commands.down)?;
            self.y += 1
        };
        Ok(())
    }

    pub fn move_left(&mut self) -> Result<(), Error> {
        self.x = if self.x == 0 { 0 } else { self.x - 1 };
        Self::write_raw_command(&self.commands.left)
    }

    pub fn move_right(&mut self, xlimit: usize) -> Result<(), Error> {
        if self.x < xlimit {
            Self::write_raw_command(&self.commands.right)?;
            self.x += 1
        };
        Ok(())
    }

    pub fn delete_char(&mut self) -> Result<(), Error> {
        Self::write_raw_command(&self.commands.delete_char)
    }

    pub fn delete_line(&mut self) -> Result<(), Error> {
        Self::write_raw_command(&self.commands.delete_line)
    }

    pub fn backspace(&mut self) -> Result<(), Error> {
        if self.x == 0 {
            return Ok(());
        }
        self.move_left()?;
        self.delete_char()
    }

    pub fn write_raw_command(command: &str) -> Result<(), Error> {
        stdout().write_fmt(format_args!("{}", command))?;
        stdout().flush()
    }

    pub fn write_command_with_args(command: &str, args: &[usize]) -> Result<(), Error> {
        let s = TermInfo::format(command, args);
        stdout().write_fmt(format_args!("{}", s))?;
        stdout().flush()
    }
}
