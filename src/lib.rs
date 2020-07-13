#[macro_use]
extern crate lazy_static;

mod cmx;
mod core;
mod events;
pub use cmx::Term;
pub use events::{Direction, Event};

#[cfg(test)]
mod tests {

    use crate::Term;
    use term::terminfo::TermInfo;

    #[test]
    fn test_show_info() {
        let term = match Term::new() {
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
