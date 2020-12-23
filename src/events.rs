use lazy_static::lazy_static;
use std::clone::Clone;
use std::collections::BTreeMap;
use std::str::FromStr;

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    Raw(Input),
    Ctrl(Input),
    Meta(Input),
    Shift(Input),
    TimeOut,
    TermSize(usize, usize),
}

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
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
