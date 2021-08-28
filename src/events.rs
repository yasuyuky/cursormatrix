use lazy_static::lazy_static;
use std::clone::Clone;
use std::collections::BTreeMap;
use std::fmt;
use std::hash::Hash;
use std::io;
use std::str::FromStr;

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Event {
    Raw(Input),
    Ctrl(Input),
    Meta(Input),
    Shift(Input),
    TimeOut,
    TermSize(usize, usize),
}

impl FromStr for Event {
    type Err = io::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("ctrl+") {
            Ok(Self::Ctrl(Input::from_str(&s[5..])?))
        } else if s.starts_with("meta+") {
            Ok(Self::Meta(Input::from_str(&s[5..])?))
        } else if s.starts_with("shift+") {
            Ok(Self::Shift(Input::from_str(&s[6..])?))
        } else {
            Ok(Self::Raw(Input::from_str(s)?))
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raw(i) => write!(f, "{}", i),
            Self::Ctrl(i) => write!(f, "ctrl+{}", i),
            Self::Meta(i) => write!(f, "meta+{}", i),
            Self::Shift(i) => write!(f, "shift+{}", i),
            Self::TimeOut => write!(f, "timeout"),
            Self::TermSize(x, y) => write!(f, "({},{})", x, y),
        }
    }
}

#[test]
fn test_deserialize_event() {
    let ctrl_s = Event::from_str("ctrl+s").unwrap();
    assert_eq!(ctrl_s, Event::Ctrl(Input::Chars("s".to_owned())));
    let meta_up = Event::from_str("meta+up").unwrap();
    assert_eq!(meta_up, Event::Meta(Input::Arrow(Direction::Up)));
}

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Input {
    Chars(String),
    Function(u8),
    Arrow(Direction),
    Scroll(Direction),
    Page(Direction),
    Return,
    Enter,
    Tab,
    BackSpace,
    Delete,
    Escape,
    Home,
    End,
}

impl FromStr for Input {
    type Err = io::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "return" => Ok(Self::Return),
            "enter" => Ok(Self::Enter),
            "tab" => Ok(Self::Tab),
            "backspace" => Ok(Self::BackSpace),
            "delete" => Ok(Self::Delete),
            "escape" => Ok(Self::Escape),
            "home" => Ok(Self::Home),
            "end" => Ok(Self::End),
            "pageup" => Ok(Self::Page(Direction::Up)),
            "pagedown" => Ok(Self::Page(Direction::Down)),
            "scrollup" => Ok(Self::Scroll(Direction::Up)),
            "scrolldown" => Ok(Self::Scroll(Direction::Down)),
            s => {
                if Direction::from_str(s).is_ok() {
                    Ok(Self::Arrow(Direction::from_str(s)?))
                } else if s.len() > 1 && s.starts_with("f") {
                    Ok(Self::Function(s[1..].parse::<u8>().unwrap_or_default()))
                } else {
                    Ok(Self::Chars(s.to_owned()))
                }
            },
        }
    }
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Chars(s) => write!(f, "{}", s),
            Self::Function(n) => write!(f, "f{}", n),
            Self::Arrow(d) => write!(f, "{}", d),
            Self::Scroll(d) => write!(f, "scroll{}", d),
            Self::Page(d) => write!(f, "page{}", d),
            Self::Return => write!(f, "return"),
            Self::Enter => write!(f, "enter"),
            Self::Tab => write!(f, "tab"),
            Self::BackSpace => write!(f, "backspace"),
            Self::Delete => write!(f, "delete"),
            Self::Escape => write!(f, "escape"),
            Self::Home => write!(f, "home"),
            Self::End => write!(f, "end"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl FromStr for Direction {
    type Err = io::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "up" => Ok(Self::Up),
            "down" => Ok(Self::Down),
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "cannot parse")),
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Left => "left",
            Self::Right => "right",
        })
    }
}

lazy_static! {
    pub static ref CTRL_KEY_DICT: BTreeMap<Vec<u8>, Event> =
        (0u8..32).map(|x| (vec![x], Event::Ctrl(Input::Chars(((x + 64) as char).to_string()))))
                 .collect();
    pub static ref META_KEY_DICT: BTreeMap<Vec<u8>, Event> =
        (32u8..128).map(|x| (vec![0x1b, x], Event::Meta(Input::Chars((x as char).to_string()))))
                   .collect();
    pub static ref TERMINFO_KEY_DICT: BTreeMap<String, Event> = {
        [("kcuu1", Event::Raw(Input::Arrow(Direction::Up))),
         ("kcud1", Event::Raw(Input::Arrow(Direction::Down))),
         ("kcub1", Event::Raw(Input::Arrow(Direction::Left))),
         ("kcuf1", Event::Raw(Input::Arrow(Direction::Right))),
         ("dch1", Event::Raw(Input::Delete)),
         ("tab", Event::Raw(Input::Tab)),
         ("cr", Event::Raw(Input::Return)),
         ("khome", Event::Raw(Input::Home)),
         ("kend", Event::Raw(Input::End)),
         ("kpp", Event::Raw(Input::Page(Direction::Up))),
         ("knp", Event::Raw(Input::Page(Direction::Down))),
         ("key_sr", Event::Raw(Input::Scroll(Direction::Up))),
         ("key_sf", Event::Raw(Input::Scroll(Direction::Down)))].iter()
                                                                .map(|&(k, ref v)| {
                                                                    (String::from_str(k).unwrap(), v.clone())
                                                                })
                                                                .chain((0u8..64).map(|i| {
                                                                                    (format!("key_f{}", i),
                                                                                     Event::Raw(Input::Function(i)))
                                                                                }))
                                                                .collect()
    };
    pub static ref DEFAULT_KEY_DICT: BTreeMap<Vec<u8>, Event> = {
        [("\u{1b}[A", Input::Arrow(Direction::Up)),
         ("\u{1b}[B", Input::Arrow(Direction::Down)),
         ("\u{1b}[D", Input::Arrow(Direction::Left)),
         ("\u{1b}[C", Input::Arrow(Direction::Right)),
         ("\u{1b}[H", Input::Home),
         ("\u{1b}[F", Input::End),
         ("\u{08}", Input::BackSpace),
         ("\t", Input::Tab),
         ("\n", Input::Enter),
         ("\r", Input::Return),
         ("\u{1b}", Input::Escape),
         ("\u{7f}", Input::Delete)].iter()
                                   .map(|(k, v)| {
                                       vec![(k.chars().map(|c| c as u8).collect(), Event::Raw(v.clone())),
                                            ((String::from("\u{1b}") + k).chars().map(|c| c as u8).collect(),
                                             Event::Meta(v.clone())),]
                                   })
                                   .flatten()
                                   .collect()
    };
    pub static ref MOD_ARROW_KEY_DICT: BTreeMap<Vec<u8>, Event> = {
        let arrows = [("\u{1b}[1;5A", Event::Ctrl(Input::Arrow(Direction::Up))),
                      ("\u{1b}[1;5B", Event::Ctrl(Input::Arrow(Direction::Down))),
                      ("\u{1b}[1;5D", Event::Ctrl(Input::Arrow(Direction::Left))),
                      ("\u{1b}[1;5C", Event::Ctrl(Input::Arrow(Direction::Right))),
                      ("\u{1b}[1;2A", Event::Shift(Input::Arrow(Direction::Up))),
                      ("\u{1b}[1;2B", Event::Shift(Input::Arrow(Direction::Down))),
                      ("\u{1b}[1;2D", Event::Shift(Input::Arrow(Direction::Left))),
                      ("\u{1b}[1;2C", Event::Shift(Input::Arrow(Direction::Right)))];
        arrows.iter()
              .map(|(k, v)| (k.chars().map(|c| c as u8).collect(), v.clone()))
              .collect()
    };
}
