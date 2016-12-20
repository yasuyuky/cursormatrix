use std::collections::BTreeMap;
use std::str::FromStr;
use std::clone::Clone;


#[allow(dead_code)]
#[derive(Clone,Debug,Eq,PartialEq)]
pub enum Event {
    Chars(String),
    Ctrl(char),
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
    TimeOut,
}

#[derive(Clone,Debug,Eq,PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}


lazy_static! {

    pub static ref CTRL_KEY_DICT : BTreeMap<Vec<u8>, Event> = {
        (0u8..32).map(|x| (vec![x],Event::Ctrl((x+64) as char)) ).collect()
    };

    pub static ref TERMINFO_KEY_DICT : BTreeMap<String, Event> = {
        [
          ("kcuu1",   Event::Arrow(Direction::Up)),
          ("kcud1",   Event::Arrow(Direction::Down)),
          ("kcub1",   Event::Arrow(Direction::Left)),
          ("kcuf1",   Event::Arrow(Direction::Right)),
          ("dch1",    Event::Delete),
          ("tab",     Event::Tab),
          ("cr",      Event::Return),
          ("khome",   Event::Home),
          ("key_end", Event::End),
          ("kpp",     Event::Page(Direction::Up)),
          ("knp",     Event::Page(Direction::Down)),
          ("key_sr",  Event::Scroll(Direction::Up)),
          ("key_sf",  Event::Scroll(Direction::Down)),
        ].iter().map(|&(k,ref v)| (String::from_str(k).unwrap(),v.clone()))
         .chain((0u8..64).map(|i| (format!("key_f{}",i), Event::Function(i))))
         .collect()
    };

    pub static ref DEFAULT_KEY_DICT : BTreeMap<Vec<u8>, Event> = {
        [
          ("\u{1b}[A", Event::Arrow(Direction::Up)),
          ("\u{1b}[B", Event::Arrow(Direction::Down)),
          ("\u{1b}[D", Event::Arrow(Direction::Left)),
          ("\u{1b}[C", Event::Arrow(Direction::Right)),
          ("\u{1b}[H", Event::Home),
          ("\u{1b}[F", Event::End),
          ("\u{08}",   Event::BackSpace),
          ("\t",       Event::Tab),
          ("\n",       Event::Enter),
          ("\r",       Event::Return),
          ("\u{1b}",   Event::Escape),
          ("\u{7f}",   Event::Delete),
        ].iter().map(|&(k,ref v)| (k.chars().map(|c| c as u8).collect(),v.clone()))
         .collect()
    };


}
