use std::time::Duration;
use std::io::{Error, ErrorKind, Write, stdout};
use std::mem;
use std::os::unix::io::AsRawFd;
use std::ptr;
use std::collections::BTreeMap;
use std::string::FromUtf8Error;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::collections::Bound::*;

use libc;
use crossbeam;

use events::{CTRL_KEY_DICT, DEFAULT_KEY_DICT, Event, TERMINFO_KEY_DICT};
use core::{Cursor, TermInfo, TermiosCond, Tty};

#[allow(dead_code)]
#[derive(Clone)]
pub struct Term {
    pub pattern_dict: BTreeMap<Vec<u8>, Event>,
    pub cursor: Cursor,
    pub terminfo: TermInfo,
    termioscond: TermiosCond,
    tty: Tty,
}

#[allow(dead_code)]
impl Term {
    pub fn new() -> Result<Term, Error> {
        Self::from_cjk(false)
    }

    pub fn cjk() -> Result<Term, Error> {
        Self::from_cjk(true)
    }

    pub fn from_cjk(cjk: bool) -> Result<Term, Error> {
        let terminfo = TermInfo::new();
        let tty = Tty::new();
        let mut term = Term { pattern_dict: Self::create_pattern_dict(&terminfo),
                              cursor: Cursor::new(&terminfo, &tty, cjk)?,
                              terminfo: terminfo,
                              termioscond: TermiosCond::from_tty(&tty),
                              tty: tty, };
        term.write_raw_command("smcup")?;
        term.cursor.clear()?;
        Ok(term)
    }

    fn write_raw_command(&mut self, command: &str) -> Result<(), Error> {
        stdout().write_fmt(format_args!("{}", self.terminfo.get_string(command)))?;
        stdout().flush()
    }

    fn write_command_with_args(&mut self, command: &str, args: &Vec<usize>) -> Result<(), Error> {
        let s = TermInfo::format(&self.terminfo.get_string(command), args);
        stdout().write_fmt(format_args!("{}", s))?;
        stdout().flush()
    }

    pub fn get_input(&mut self, maybe_timeout: Option<Duration>) -> Result<Event, Error> {
        let timeout: *mut libc::timeval = match maybe_timeout {
            None => ptr::null_mut(),
            Some(to) => {
                &mut libc::timeval { tv_sec: to.as_secs() as libc::time_t,
                                     tv_usec: (to.subsec_nanos() as libc::suseconds_t) / 1000, }
            },
        };

        let rawfd = self.tty.as_raw_fd();
        let mut readfds: libc::fd_set = unsafe { mem::zeroed() };
        unsafe { libc::FD_SET(rawfd, &mut readfds) };
        let mut buf = Vec::<u8>::new();
        loop {
            match unsafe {
                libc::select(rawfd + 1, &mut readfds, ptr::null_mut(), ptr::null_mut(), timeout)
            } {
                -1 => {
                    let err = Error::last_os_error();
                    match err.kind() {
                        ErrorKind::Interrupted => continue,
                        _ => return Err(err),
                    }
                },
                0 => {
                    assert!(maybe_timeout.is_some());
                    return Ok(Event::TimeOut);
                },
                _ => {
                    self.tty.read_to_end(&mut buf)?;
                    assert!(buf.len() > 0);
                    match Self::convert_to_event(&self.pattern_dict, &buf) {
                        Ok(e) => return Ok(e),
                        Err(_) => continue,
                    }
                },
            }
        }

    }

    pub fn get_input_async(&mut self, maybe_timeout: Option<Duration>, etx: Sender<Event>)
                           -> Result<(), Error> {

        crossbeam::scope(|scope| {
            let (btx, brx) = channel::<u8>();
            let etx_clone = etx.clone();
            let dic = self.pattern_dict.clone();
            scope.spawn(move || Self::recieve_to_convert(dic, brx, etx_clone));

            let timeout: *mut libc::timeval = match maybe_timeout {
                None => ptr::null_mut(),
                Some(to) => {
                    &mut libc::timeval { tv_sec: to.as_secs() as libc::time_t,
                                         tv_usec: (to.subsec_nanos() as libc::suseconds_t) / 1000, }
                },
            };

            let rawfd = self.tty.as_raw_fd();
            let mut readfds: libc::fd_set = unsafe { mem::zeroed() };
            unsafe { libc::FD_SET(rawfd, &mut readfds) };
            loop {
                match unsafe {
                    libc::select(rawfd + 1, &mut readfds, ptr::null_mut(), ptr::null_mut(), timeout)
                } {
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

        })

    }

    fn recieve_to_convert(patterns: BTreeMap<Vec<u8>, Event>, brx: Receiver<u8>, etx: Sender<Event>) {
        let mut timeout = Duration::from_millis(1000);
        let mut buf = Vec::<u8>::new();

        'recv_byte: loop {
            match brx.recv_timeout(timeout) {
                Ok(b) => {
                    buf.push(b);
                    match patterns.get(&buf) {
                        Some(_) => {
                            match patterns.range::<Vec<u8>, _>((Excluded(&buf),
                                                                Unbounded::<&Vec<u8>>))
                                          .next() {
                                Some((ref k, _)) => {
                                    if buf.iter().enumerate().all(|(i, &x)| x == k[i]) {
                                        timeout = Duration::from_millis(1);
                                        continue 'recv_byte;
                                    } else {
                                        etx.send(Self::convert_to_event(&patterns, &buf).unwrap())
                                           .unwrap();
                                        buf.clear();
                                        timeout = Duration::from_millis(1000);
                                    }
                                },
                                None => {
                                    timeout = Duration::from_millis(1);
                                    continue 'recv_byte;
                                },
                            }
                        },
                        None => {
                            timeout = Duration::from_millis(1);
                            continue 'recv_byte;
                        },
                    }
                },
                Err(_) => {
                    match buf.len() {
                        0 => continue 'recv_byte,
                        _ => {
                            match Self::convert_to_event(&patterns, &buf) {
                                Ok(e) => etx.send(e).unwrap(),
                                Err(_) => return,
                            }
                            buf.clear();
                            timeout = Duration::from_millis(1000);
                        },
                    }
                },
            }
        }
    }

    fn create_pattern_dict(terminfo: &TermInfo) -> BTreeMap<Vec<u8>, Event> {
        let terminfo_dict = terminfo.info
                                    .strings
                                    .iter()
                                    .filter_map(|(k, v)| {
                                        match TERMINFO_KEY_DICT.get(k) {
                                            Some(e) => Some((v.clone(), e.clone())),
                                            None => None,
                                        }
                                    })
                                    .collect::<BTreeMap<Vec<u8>, Event>>();
        CTRL_KEY_DICT.clone()
                     .into_iter()
                     .chain(DEFAULT_KEY_DICT.clone().into_iter())
                     .chain(terminfo_dict.into_iter())
                     .collect()
    }

    fn convert_to_event(pattern_dict: &BTreeMap<Vec<u8>, Event>, buf: &Vec<u8>)
                        -> Result<Event, FromUtf8Error> {
        match pattern_dict.get(buf) {
            Some(e) => return Ok(e.clone()),
            None => (),
        };
        match String::from_utf8(buf.clone()) {
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
