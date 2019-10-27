use std::str::FromStr;
use unicode_width::UnicodeWidthStr as UWStr;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Rune {
    UStr(String, usize),
    Pad,
}

#[allow(dead_code)]
impl Rune {
    pub fn from_str(s: &str, cjk: bool) -> Self {
        if cjk {
            Rune::UStr(String::from_str(s).unwrap(), UWStr::width_cjk(s))
        } else {
            Rune::UStr(String::from_str(s).unwrap(), UWStr::width(s))
        }
    }

    pub fn push_str(&mut self, s: &str, cjk: bool) -> Self {
        match *self {
            Rune::UStr(ref mut os, _) => {
                os.push_str(s);
                Self::from_str(os.as_str(), cjk)
            },
            Rune::Pad => Rune::Pad,
        }
    }
}
