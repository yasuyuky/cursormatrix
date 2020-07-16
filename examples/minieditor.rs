use cursormatrix::{Direction, Event, Term};

fn handle_event(ev: &Event, term: &mut Term) -> bool {
    match ev {
        &Event::Ctrl('C') | &Event::TimeOut => return false,
        &Event::Ctrl('L') => term.clear().unwrap(),
        &Event::Ctrl('A') => term.move_home().unwrap(),
        &Event::Ctrl('E') => term.move_end().unwrap(),
        &Event::Arrow(Direction::Up) => term.move_up().unwrap(),
        &Event::Arrow(Direction::Down) => term.move_down().unwrap(),
        &Event::Arrow(Direction::Left) => term.move_left().unwrap(),
        &Event::Arrow(Direction::Right) => term.move_right().unwrap(),
        &Event::Ctrl('D') => term.cursor.delete_char().unwrap(),
        &Event::Return => {
            term.print("↩︎").unwrap();
            term.move_home().unwrap();
            term.move_down().unwrap();
        },
        &Event::Delete => term.cursor.backspace().unwrap(),
        &Event::BackSpace => term.cursor.backspace().unwrap(),
        &Event::Chars(ref s) => {
            use unicode_normalization::UnicodeNormalization;
            term.print(&format!("{}", s.nfkc().collect::<String>())).unwrap();
        },
        e => {
            let pos = term.cursor.get_pos();
            term.print(format!("e: {:?}, pos{:?}", e, pos).as_str()).unwrap();
        },
    }
    true
}

fn main() {
    let (mut term, erx) = Term::with_input(true).expect("term");

    term.print("edit").unwrap();
    loop {
        if match erx.recv() {
            Ok(ev) => !handle_event(&ev, &mut term),
            Err(_) => false,
        } {
            break;
        }
    }
}
