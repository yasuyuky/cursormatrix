extern crate unicode_width;
use self::unicode_width::{UnicodeWidthChar as UWChar, UnicodeWidthStr as UWStr};
use crate::core::{TermSize, Tty};
use std::collections::VecDeque;
use std::io::Error;
use std::iter;
use std::iter::FromIterator;
use std::str::FromStr;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum PadStr {
    UStr(String, usize),
    Pad,
}

#[allow(dead_code)]
impl PadStr {
    pub fn from_str(s: &str, cjk: bool) -> PadStr {
        if cjk {
            PadStr::UStr(String::from_str(s).unwrap(), UWStr::width_cjk(s))
        } else {
            PadStr::UStr(String::from_str(s).unwrap(), UWStr::width(s))
        }
    }

    pub fn push_str(&mut self, s: &str, cjk: bool) -> PadStr {
        match *self {
            PadStr::UStr(ref mut os, _) => {
                os.push_str(s);
                Self::from_str(os.as_str(), cjk)
            },
            PadStr::Pad => PadStr::Pad,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Matrix {
    data: Vec<Vec<PadStr>>,
    pub range: TermSize,
    cjk: bool,
}

#[allow(dead_code)]
impl Matrix {
    pub fn from_tty(tty: &Tty, cjk: bool) -> Result<Matrix, Error> {
        let mut matrix = Matrix { data: Vec::new(),
                                  range: TermSize::from_tty(tty)?,
                                  cjk };
        matrix.refresh()?;
        Ok(matrix)
    }

    pub fn refresh(&mut self) -> Result<(), Error> {
        self.range.refresh()?;
        self.data.resize(self.range.height, Vec::new());
        for d in self.data.iter_mut() {
            d.resize(self.range.width, PadStr::from_str(" ", self.cjk))
        }
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), Error> {
        self.data.clear();
        self.refresh()
    }

    fn get_width(&self, c: char) -> usize {
        if self.cjk {
            UWChar::width_cjk(c).unwrap_or(0)
        } else {
            UWChar::width(c).unwrap_or(0)
        }
    }

    pub fn put_buffer(&mut self, x: usize, y: usize, w: usize, s: &str) -> (usize, String) {
        let ws = self.fill_line(s, w, ' ');
        let replace_data = self.create_padstr(&ws);
        let end = *[x + replace_data.len(), self.range.width].iter().min().unwrap();
        let new_vecpadstr = self.data[y].iter()
                                        .enumerate()
                                        .filter_map(|(i, pad_uc)| {
                                            if i < x {
                                                Some(pad_uc.clone())
                                            } else if i < x + replace_data.len() && i < self.range.width {
                                                Some(replace_data[i - x].clone())
                                            } else if i < self.range.width {
                                                Some(pad_uc.clone())
                                            } else {
                                                None
                                            }
                                        })
                                        .collect::<Vec<PadStr>>();
        self.data[y] = new_vecpadstr;
        (end, Self::subpadstr(&self.data[y], x, end))
    }

    pub fn lines(&self) -> Vec<String> {
        self.data
            .iter()
            .map(|l| Self::subpadstr(l, 0, self.range.width))
            .collect()
    }

    fn create_padstr(&self, s: &str) -> Vec<PadStr> {
        let s_with_w = s.chars()
                        .map(|c| (c.to_string(), self.get_width(c)))
                        .collect::<Vec<(String, usize)>>();
        let mut deq: VecDeque<PadStr> = VecDeque::new();
        for &(ref s, ref w) in s_with_w.iter() {
            match *w {
                0 => match deq.pop_back() {
                    Some(PadStr::UStr(ref us, _)) => {
                        deq.push_back(PadStr::from_str(&(us.clone() + s), self.cjk));
                    },
                    Some(PadStr::Pad) => {
                        let i = deq.len() - 2;
                        deq[i] = deq[i].push_str(s.as_str(), self.cjk);
                        deq.push_back(PadStr::Pad)
                    },
                    None => deq.push_back(PadStr::from_str(s, self.cjk)),
                },
                n => {
                    deq.push_back(PadStr::from_str(s, self.cjk));
                    for _ in 1..n {
                        deq.push_back(PadStr::Pad)
                    }
                },
            }
        }
        deq.into_iter().collect()
    }

    fn subpadstr(padstrs: &[PadStr], start: usize, end: usize) -> String {
        let end_ = match padstrs[end - 1] {
            PadStr::UStr(_, 2) => end - 1,
            _ => end,
        };
        padstrs[start..end_].iter()
                            .filter_map(|iu| match *iu {
                                PadStr::UStr(ref u, _) => Some(u.clone()),
                                PadStr::Pad => None,
                            })
                            .collect()
    }

    pub fn fill_line(&self, s: &str, w: usize, c: char) -> String {
        let mut pos = 0usize;
        for c in s.chars() {
            if pos < w {
                pos += self.get_width(c);
            } else {
                break;
            }
        }
        [s, String::from_iter(iter::repeat(c).take(w - pos)).as_str()].join("")
    }
}
