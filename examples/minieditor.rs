extern crate cursormatrix;

use cursormatrix::events::*;
use cursormatrix::Term;
use std::time::Duration;

fn handle_event(ev: &Event, term: &mut Term) -> bool {
    match ev {
        &Event::Ctrl('C') => return false,
        &Event::TimeOut => return false,
        &Event::Ctrl('L') => term.cursor.clear().unwrap(),
        &Event::Ctrl('R') => term.cursor.reload().unwrap(),
        &Event::Ctrl('A') => term.cursor.move_home().unwrap(),
        &Event::Ctrl('E') => term.cursor.move_end().unwrap(),
        &Event::Arrow(Direction::Up) => term.cursor.move_up().unwrap(),
        &Event::Arrow(Direction::Down) => term.cursor.move_down().unwrap(),
        &Event::Arrow(Direction::Left) => term.cursor.move_left().unwrap(),
        &Event::Arrow(Direction::Right) => term.cursor.move_right().unwrap(),
        &Event::Ctrl('D') => term.cursor.delete_char().unwrap(),
        &Event::Delete => term.cursor.delete_char().unwrap(),
        &Event::BackSpace => term.cursor.backspace().unwrap(),
        &Event::Chars(ref s) => {
            use unicode_normalization::UnicodeNormalization;
            term.cursor
                .print_here(&format!("{}", s.nfkc().collect::<String>()))
                .unwrap();
        },
        e => {
            let pos = term.cursor.get_pos();
            term.cursor
                .print_here(format!("e: {:?}, pos{:?}", e, pos).as_str())
                .unwrap();
        },
    }
    true
}

fn main() {
    let (mut term, erx) = Term::with_input(Some(Duration::from_secs(10)), true).expect("term");

    term.cursor.print_fill_here("async!!!", 10).unwrap();
    loop {
        if match erx.recv() {
            Ok(ev) => !handle_event(&ev, &mut term),
            Err(_) => false,
        } {
            break;
        }
    }
}