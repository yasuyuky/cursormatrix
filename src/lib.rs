mod core;
mod events;
mod terminal;
pub use events::{Direction, Event, Input};
pub use terminal::Term;

#[cfg(test)]
mod tests {

    use crate::Term;
    use term::terminfo::TermInfo;

    #[test]
    fn test_show_info() {
        let term = match Term::from_cjk(true) {
            Ok(term) => term,
            Err(_) => return,
        };
        let terminfo = term.terminfo.clone();
        drop(term);
        view_terminfo(&terminfo.info);
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
