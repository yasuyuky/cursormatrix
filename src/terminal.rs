use crate::core::{Cursor, Matrix, TermInfo, TermiosCond, Tty};
use crate::events::{Event, Input, CTRL_KEY_DICT, DEFAULT_KEY_DICT, TERMINFO_KEY_DICT};
use colored::Colorize;
use crossbeam;
use libc;
use std::collections::BTreeMap;
use std::collections::Bound::*;
use std::io::{stdout, Error, ErrorKind, Write};
use std::mem;
use std::os::unix::io::AsRawFd;
use std::ptr;
use std::string::FromUtf8Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

static SIGWINCH_RECIEVED: AtomicBool = AtomicBool::new(false);

#[allow(dead_code)]
pub struct Term {
    pub cursor: Cursor,
    pub matrix: Matrix,
    pub terminfo: TermInfo,
    pub xlimit: Option<usize>,
    pub fg: Vec<(u8, u8, u8)>,
    pub bg: Vec<(u8, u8, u8)>,
    termioscond: TermiosCond,
    cjk: bool,
}

#[allow(dead_code)]
impl Term {
    pub fn from_cjk(cjk: bool) -> Result<Self, Error> {
        Self::setup_sighandler()?;
        let terminfo = TermInfo::new();
        let tty = Tty::new().expect("open tty");
        let (w, h) = Self::load_winsize(&tty)?;
        let mut term = Term { cursor: Cursor::new(&terminfo)?,
                              matrix: Matrix::new(w, h),
                              terminfo,
                              xlimit: None,
                              fg: Vec::new(),
                              bg: Vec::new(),
                              termioscond: TermiosCond::from_tty(tty),
                              cjk };
        term.write_raw_command("smcup")?;
        term.cursor.clear()?;
        Ok(term)
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

    pub fn with_input(cjk: bool) -> Result<(Self, Receiver<Event>), Error> {
        let term = Self::from_cjk(cjk)?;

        let patterns = Self::create_pattern_dict(&term.terminfo);
        let (etx, erx) = channel::<Event>();
        thread::spawn(move || Self::get_input(patterns, etx));
        Ok((term, erx))
    }

    pub fn clear(&mut self) -> Result<(), std::io::Error> {
        self.cursor.clear()
    }

    pub fn width_char(&self, c: char) -> usize {
        if self.cjk {
            UnicodeWidthChar::width(c).unwrap_or_default()
        } else {
            UnicodeWidthChar::width_cjk(c).unwrap_or_default()
        }
    }

    pub fn width_str(&self, s: &str) -> usize {
        if self.cjk {
            UnicodeWidthStr::width(s)
        } else {
            UnicodeWidthStr::width_cjk(s)
        }
    }

    fn limit_string(&self, s: &str, limit: usize) -> (String, usize) {
        let mut w = 0;
        (s.chars()
          .take_while(|c| {
              w += self.width_char(*c);
              w <= limit
          })
          .collect(),
         w)
    }

    pub fn print(&mut self, s: &str) -> Result<(), Error> {
        let (x, y) = self.cursor.get_pos();
        let (mut s, w) = match self.xlimit {
            Some(limit) => self.limit_string(s, std::cmp::max(limit as isize - x as isize, 0) as usize),
            None => (String::from(s), self.width_str(s)),
        };
        if let Some((r, g, b)) = self.bg.last() {
            s = format!("{}", s.on_truecolor(*r, *g, *b));
        }
        if let Some((r, g, b)) = self.fg.last() {
            s = format!("{}", s.truecolor(*r, *g, *b));
        }
        self.cursor.print(&s)?;
        self.cursor.move_to(x + w, y)
    }

    pub fn cprint(&mut self, s: &str, fg: Option<(u8, u8, u8)>, bg: Option<(u8, u8, u8)>) -> Result<(), Error> {
        bg.and_then(|c| Some(self.bg.push(c)));
        fg.and_then(|c| Some(self.fg.push(c)));
        self.print(s)?;
        fg.and_then(|_| self.fg.pop());
        bg.and_then(|_| self.bg.pop());
        Ok(())
    }

    pub fn move_to(&mut self, x: usize, y: usize) -> Result<(), Error> {
        let x = std::cmp::min(x, self.matrix.width - 1);
        let y = std::cmp::min(y, self.matrix.height - 1);
        self.cursor.move_to(x, y)
    }

    pub fn move_up(&mut self) -> Result<(), Error> {
        self.cursor.move_up()
    }

    pub fn move_down(&mut self) -> Result<(), Error> {
        self.cursor.move_down(self.matrix.height - 1)
    }

    pub fn move_left(&mut self) -> Result<(), Error> {
        self.cursor.move_left()
    }

    pub fn move_right(&mut self) -> Result<(), Error> {
        self.cursor.move_right(self.matrix.width - 1)
    }

    pub fn move_home(&mut self) -> Result<(), Error> {
        self.move_to(0, self.cursor.y)
    }

    pub fn move_end(&mut self) -> Result<(), Error> {
        self.move_to(self.matrix.width - 1, self.cursor.y)
    }

    pub fn move_top(&mut self) -> Result<(), Error> {
        self.move_to(self.cursor.x, 0)
    }

    pub fn move_bottom(&mut self) -> Result<(), Error> {
        self.move_to(self.cursor.x, self.matrix.height - 1)
    }

    fn write_raw_command(&mut self, command: &str) -> Result<(), Error> {
        stdout().write_fmt(format_args!("{}", self.terminfo.get_string(command)))?;
        stdout().flush()
    }

    fn write_command_with_args(&mut self, command: &str, args: &[usize]) -> Result<(), Error> {
        let s = TermInfo::format(&self.terminfo.get_string(command), args);
        stdout().write_fmt(format_args!("{}", s))?;
        stdout().flush()
    }

    fn check_resizing(tty: &mut Tty, etx: &Sender<Event>) -> Result<(), Error> {
        if SIGWINCH_RECIEVED.load(Ordering::SeqCst) {
            SIGWINCH_RECIEVED.store(false, Ordering::SeqCst);
            let (w, h) = Self::load_winsize(tty)?;
            etx.send(Event::TermSize(w, h)).unwrap()
        }
        Ok(())
    }

    fn send_buffer(tty: &mut Tty, btx: &Sender<u8>) -> Result<(), Error> {
        let mut buf = Vec::<u8>::new();
        tty.read_to_end(&mut buf)?;
        for b in buf.iter() {
            btx.send(*b).unwrap()
        }
        Ok(buf.clear())
    }

    fn loop_select(tty: &mut Tty, btx: Sender<u8>, etx: Sender<Event>) -> Result<(), Error> {
        let timeout: *mut libc::timeval = &mut libc::timeval { tv_sec: 0,
                                                               tv_usec: 100000 };
        let rawfd = tty.as_raw_fd();
        let mut readfds: libc::fd_set = unsafe { mem::zeroed() };
        loop {
            Self::check_resizing(tty, &etx)?;
            unsafe { libc::FD_SET(rawfd, &mut readfds) };
            match unsafe { libc::select(rawfd + 1, &mut readfds, ptr::null_mut(), ptr::null_mut(), timeout) } {
                -1 => {
                    let err = Error::last_os_error();
                    match Error::last_os_error().kind() {
                        ErrorKind::Interrupted => continue,
                        _ => return Err(err),
                    }
                },
                0 => continue,
                _ => Self::send_buffer(tty, &btx)?,
            }
        }
    }

    fn get_input(patterns: BTreeMap<Vec<u8>, Event>, etx: Sender<Event>) -> Result<(), Error> {
        crossbeam::scope(|scope| {
            let (btx, brx) = channel::<u8>();
            let etx_input = etx.clone();
            scope.spawn(move |_| Self::recieve_to_convert(&patterns, brx, etx_input));
            let mut tty = Tty::new()?;
            Self::loop_select(&mut tty, btx, etx)
        }).unwrap()
    }

    fn recieve_to_convert(patterns: &BTreeMap<Vec<u8>, Event>, brx: Receiver<u8>, etx: Sender<Event>) {
        let mut timeout = Duration::from_millis(1000);
        let mut buf = Vec::<u8>::new();

        'recv_byte: loop {
            match brx.recv_timeout(timeout) {
                Ok(b) => {
                    buf.push(b);
                    match patterns.get(&buf) {
                        Some(_) => match patterns.range::<Vec<u8>, _>((Excluded(&buf), Unbounded::<&Vec<u8>>))
                                                 .next()
                        {
                            Some((ref k, _)) => {
                                if buf.iter().enumerate().all(|(i, &x)| x == k[i]) {
                                    timeout = Duration::from_millis(1);
                                    continue 'recv_byte;
                                } else {
                                    etx.send(Self::convert_to_event(&patterns, &buf).unwrap()).unwrap();
                                    buf.clear();
                                    timeout = Duration::from_millis(1000);
                                }
                            },
                            None => {
                                timeout = Duration::from_millis(1);
                                continue 'recv_byte;
                            },
                        },
                        None => {
                            timeout = Duration::from_millis(1);
                            continue 'recv_byte;
                        },
                    }
                },
                Err(_) => match buf.len() {
                    0 => continue 'recv_byte,
                    _ => {
                        match Self::convert_to_event(patterns, &buf) {
                            Ok(e) => etx.send(e).unwrap(),
                            Err(_) => return,
                        }
                        buf.clear();
                        timeout = Duration::from_millis(1000);
                    },
                },
            }
        }
    }

    fn create_pattern_dict(terminfo: &TermInfo) -> BTreeMap<Vec<u8>, Event> {
        let terminfo_dict = terminfo.info
                                    .strings
                                    .iter()
                                    .filter_map(|(k, v)| match TERMINFO_KEY_DICT.get(*k) {
                                        Some(e) => Some((v.clone(), e.clone())),
                                        None => None,
                                    })
                                    .collect::<BTreeMap<Vec<u8>, Event>>();
        CTRL_KEY_DICT.clone()
                     .into_iter()
                     .chain(DEFAULT_KEY_DICT.clone().into_iter())
                     .chain(terminfo_dict.into_iter())
                     .collect()
    }

    fn convert_to_event(pattern_dict: &BTreeMap<Vec<u8>, Event>, buf: &[u8]) -> Result<Event, FromUtf8Error> {
        if let Some(e) = pattern_dict.get(buf) {
            return Ok(e.clone());
        };
        match String::from_utf8(buf.to_owned()) {
            Ok(ref s) => Ok(Event::Raw(Input::Chars(s.clone()))),
            Err(e) => Err(e),
        }
    }

    fn load_winsize(tty: &Tty) -> Result<(usize, usize), Error> {
        let mut ws: libc::winsize = unsafe { mem::MaybeUninit::uninit().assume_init() };
        let res = unsafe { libc::ioctl(tty.as_raw_fd(), libc::TIOCGWINSZ, &mut ws) };
        if res != 0 {
            return Err(Error::last_os_error());
        }
        Ok((ws.ws_col as usize, ws.ws_row as usize))
    }
}

impl Drop for Term {
    fn drop(&mut self) {
        let _ = self.write_raw_command("rmcup");
    }
}

#[allow(dead_code)]
extern "C" fn sigwinch_handler(_: i32) {
    SIGWINCH_RECIEVED.store(true, Ordering::SeqCst);
}
