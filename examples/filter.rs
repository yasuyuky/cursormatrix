use cursormatrix::{Direction, Event, Input, Term};
use std::io::{self, Read};
use std::sync::mpsc::Receiver;

#[derive(Clone)]
struct Item {
    selected: bool,
    data: String,
}

impl Item {
    pub fn new(s: &str) -> Self {
        Self { selected: false,
               data: String::from(s) }
    }
}

struct InteractiveFilter {
    data: Vec<Item>,
    view: Vec<(usize, Item)>,
    term: Term,
    erx: Receiver<Event>,
    query: String,
    line: isize,
}

impl InteractiveFilter {
    pub fn new(data: &[String]) -> Self {
        let (term, erx) = Term::with_input(true).unwrap();
        Self { data: data.iter().map(|s| Item::new(s)).collect(),
               view: Vec::new(),
               term,
               erx,
               query: String::new(),
               line: 0 }
    }

    pub fn select_line(&mut self) {
        if let Some(selection) = self.view.get(self.line as usize) {
            if let Some(item) = self.data.get_mut(selection.0) {
                item.selected = !item.selected;
            }
        }
    }

    pub fn filter(&mut self) -> Vec<String> {
        self.draw().expect("first draw");
        loop {
            let height = std::cmp::min(self.view.len(), self.term.matrix.height - 1);
            match self.erx.recv() {
                Ok(Event::TermSize(w, h)) => self.term.matrix.refresh(w, h),
                Ok(Event::Ctrl(Input::Chars(s))) => match s.as_str() {
                    "C" => break,
                    _ => continue,
                },
                Ok(Event::Raw(Input::Tab)) => self.select_line(),
                Ok(Event::Raw(Input::Arrow(Direction::Up))) => self.line = std::cmp::max(self.line - 1, 0),
                Ok(Event::Raw(Input::Arrow(Direction::Down))) => {
                    self.line = std::cmp::min(self.line + 1, height as isize - 1)
                },
                Ok(Event::Raw(Input::Chars(ref s))) => self.query.push_str(s),
                Ok(Event::Raw(Input::BackSpace)) | Ok(Event::Raw(Input::Delete)) => {
                    self.query.pop().unwrap_or_default();
                },
                _ => continue,
            }
            self.draw().expect("draw");
        }

        self.data
            .iter()
            .filter(|i| i.selected)
            .map(|i| i.data.clone())
            .collect()
    }

    pub fn draw(&mut self) -> Result<(), std::io::Error> {
        let width = self.term.matrix.width;
        let blank = " ".repeat(width);
        self.view = self.data
                        .clone()
                        .into_iter()
                        .enumerate()
                        .filter(|(_, e)| e.data.find(&self.query).is_some())
                        .collect();
        self.term.cursor.hide()?;
        self.term.xlimit = Some(width);
        self.term.bg.push((33, 33, 33));
        self.term.move_to(0, 0)?;
        self.term.print(&format!("> {}{}", self.query, blank))?;
        for l in 0..self.term.matrix.height - 1 {
            self.term.move_to(0, l + 1)?;
            match self.view.get(l) {
                Some((_, e)) => {
                    let bg = if l == self.line as usize {
                        Some(if e.selected { (154, 205, 50) } else { (0, 128, 128) })
                    } else {
                        Some(if e.selected { Some((255, 99, 71)) } else { None }).flatten()
                    };
                    self.term.cprint(&format!("{}{}", e.data, blank), None, bg)?
                },
                None => self.term.print(&blank)?,
            }
        }
        self.term.move_to(self.term.width_str(&self.query) + 2, 0)?;
        self.term.cursor.show()
    }
}

fn main() {
    let mut buf = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut buf).expect("read");
    let vs: Vec<String> = buf.lines().map(String::from).collect();

    let mut f = InteractiveFilter::new(&vs);
    let rs = f.filter();
    drop(f);
    println!("{}", rs.join("\n"));
}
