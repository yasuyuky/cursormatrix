use crate::core::{Cursor, Matrix, TermInfo, TermiosCond, Tty};
use crate::events::{Event, CTRL_KEY_DICT, DEFAULT_KEY_DICT, TERMINFO_KEY_DICT};
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
use unicode_width::UnicodeWidthStr;

static SIGWINCH_RECIEVED: AtomicBool = AtomicBool::new(false);

#[allow(dead_code)]
#[derive(Clone)]
pub struct Term {
    pub pattern_dict: BTreeMap<Vec<u8>, Event>,
    pub cursor: Cursor,
    pub matrix: Matrix,
    pub terminfo: TermInfo,
    termioscond: TermiosCond,
    tty: Tty,
}

#[allow(dead_code)]
impl Term {
    pub fn new() -> Result<Term, Error> {
        Self::from_cjk(true)
    }

    pub fn from_cjk(cjk: bool) -> Result<Term, Error> {
        Self::setup_sighandler()?;
        let terminfo = TermInfo::new();
        let tty = Tty::new().expect("open tty");
        let mut term = Term { pattern_dict: Self::create_pattern_dict(&terminfo),
                              cursor: Cursor::new(&terminfo)?,
                              matrix: Matrix::from_tty(&tty, cjk)?,
                              terminfo,
                              termioscond: TermiosCond::from_tty(&tty),
                              tty };
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

    pub fn with_input(duration: Option<Duration>, cjk: bool) -> Result<(Term, Receiver<Event>), Error> {
        let term = Self::from_cjk(cjk)?;

        let (etx, erx) = channel::<Event>();
        let mut t2 = term.clone();
        thread::spawn(move || t2.get_input(duration, etx));
        Ok((term, erx))
    }

    pub fn clear(&mut self) -> Result<(), std::io::Error> {
        self.reload()?;
        self.cursor.clear()?;
        self.matrix.clear()
    }

    pub fn reload(&mut self) -> Result<(), Error> {
        self.check_winch()
    }

    pub fn print_fill(&mut self, (x, y): (usize, usize), s: &str, w: usize) -> Result<(), Error> {
        let rs = s.replace(|c| ['\n', '\r'].iter().any(|r| c == *r), "");
        let (end, ss) = self.matrix.put_buffer(x, y, w, &rs);
        self.cursor.print_fill((end, y), &ss)
    }

    fn width_str(&self, s: &str) -> usize {
        if self.matrix.cjk {
            UnicodeWidthStr::width(s)
        } else {
            UnicodeWidthStr::width_cjk(s)
        }
    }

    pub fn print(&mut self, s: &str) -> Result<(), Error> {
        let w = self.width_str(s);
        self.print_fill(self.cursor.get_pos(), s, w)
    }

    pub fn move_to(&mut self, (x, y): (usize, usize)) -> Result<(), Error> {
        let x = std::cmp::min(x, self.matrix.range.width - 1);
        let y = std::cmp::min(y, self.matrix.range.height - 1);
        self.cursor.move_to((x, y))
    }

    pub fn move_up(&mut self) -> Result<(), Error> {
        self.cursor.move_up()
    }

    pub fn move_down(&mut self) -> Result<(), Error> {
        self.cursor.move_down(self.matrix.range.height - 1)
    }

    pub fn move_left(&mut self) -> Result<(), Error> {
        self.cursor.move_left()
    }

    pub fn move_right(&mut self) -> Result<(), Error> {
        self.cursor.move_right(self.matrix.range.width - 1)
    }

    pub fn move_home(&mut self) -> Result<(), Error> {
        self.move_to((0, self.cursor.y))
    }

    pub fn move_end(&mut self) -> Result<(), Error> {
        self.move_to((self.matrix.range.width - 1, self.cursor.y))
    }

    pub fn move_top(&mut self) -> Result<(), Error> {
        self.move_to((self.cursor.x, 0))
    }

    pub fn move_bottom(&mut self) -> Result<(), Error> {
        self.move_to((self.cursor.x, self.matrix.range.height - 1))
    }

    fn rewrite_matrix(&mut self) -> Result<(), Error> {
        self.cursor.clear()?;
        let w = self.matrix.range.width;
        for (i, l) in self.matrix.lines().iter().enumerate() {
            self.print_fill((0, i), l, w)?
        }
        Ok(())
    }

    fn check_winch(&mut self) -> Result<(), Error> {
        if SIGWINCH_RECIEVED.load(Ordering::SeqCst) {
            SIGWINCH_RECIEVED.store(false, Ordering::SeqCst);
            let (x, y) = self.cursor.get_pos();
            self.matrix.refresh()?;
            self.rewrite_matrix()?;
            self.cursor.move_to((x, y))?
        }
        Ok(())
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

    pub fn get_input(&mut self, maybe_timeout: Option<Duration>, etx: Sender<Event>) -> Result<(), Error> {
        crossbeam::scope(|scope| {
            let (btx, brx) = channel::<u8>();
            let etx_clone = etx.clone();
            let dic = self.pattern_dict.clone();
            scope.spawn(move |_| Self::recieve_to_convert(&dic, brx, etx_clone));

            let timeout: *mut libc::timeval = match maybe_timeout {
                None => ptr::null_mut(),
                Some(to) => &mut libc::timeval { tv_sec: to.as_secs() as libc::time_t,
                                                 tv_usec: (to.subsec_nanos() as libc::suseconds_t) / 1000 },
            };

            let rawfd = self.tty.as_raw_fd();
            let mut readfds: libc::fd_set = unsafe { mem::zeroed() };
            unsafe { libc::FD_SET(rawfd, &mut readfds) };
            loop {
                match unsafe { libc::select(rawfd + 1, &mut readfds, ptr::null_mut(), ptr::null_mut(), timeout) } {
                    -1 => {
                        let err = Error::last_os_error();
                        match err.kind() {
                            ErrorKind::Interrupted => continue,
                            _ => return Err(err),
                        }
                    },
                    0 => {
                        etx.send(Event::TimeOut).unwrap();
                        return Ok(());
                    },
                    _ => {
                        let mut buf = Vec::<u8>::new();
                        self.tty.read_to_end(&mut buf)?;
                        for b in buf.iter() {
                            btx.send(b.clone()).unwrap()
                        }
                        buf.clear();
                    },
                }
            }
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
            Ok(ref s) => Ok(Event::Chars(s.clone())),
            Err(e) => Err(e),
        }
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
