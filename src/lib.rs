extern crate crossbeam;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate term;
extern crate termios;
extern crate unicode_normalization;
extern crate unicode_width;

mod core;
mod cursormatrix;
mod events;

#[cfg(test)]
mod tests {

    use crate::cursormatrix;
    use crate::events::*;
    use std::time::Duration;
    use term::terminfo::TermInfo;

    #[test]
    fn test_term() -> Result<(), term::Error> {
        fn handle_event(ev: &Event, term: &mut cursormatrix::Term) -> bool {
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

        let (mut term, erx) = cursormatrix::Term::with_input(Some(Duration::from_secs(30)), true)?;

        term.cursor.print_fill_here("async!!!", 10).unwrap();
        loop {
            if match erx.recv() {
                Ok(ev) => !handle_event(&ev, &mut term),
                Err(_) => false,
            } {
                break;
            }
        }

        Ok(assert!(true))
    }

    // #[test]
    fn test_show_info() {
        let term = match cursormatrix::Term::new() {
            Ok(term) => term,
            Err(_) => return,
        };

        let terminfo = term.terminfo.clone();
        let dic = term.pattern_dict.clone();
        drop(term);

        view_terminfo(&terminfo.info);

        for (k, v) in dic {
            println!("{:?}:{:?}", k, v);
        }
        assert!(true)
    }

    #[allow(dead_code)]
    fn view_terminfo(terminfo: &TermInfo) {
        println!("names:");
        for element in &terminfo.names {
            println!("names:{:?}", element);
        }
        println!("bools:");
        for element in &terminfo.bools {
            println!("bools:{:?}", element);
        }
        println!("numbers:");
        for element in &terminfo.numbers {
            println!("numbers:{:?}", element);
        }
        println!("strings:");
        for element in &terminfo.strings {
            println!("strings:{}\t{:?}",
                     element.0,
                     String::from_utf8(element.1.clone()).unwrap());
        }
    }

}
