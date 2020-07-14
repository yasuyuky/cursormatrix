use cursormatrix::{Direction, Event, Term};
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
        let (term, erx) = Term::with_input(None, true).unwrap();
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
                Ok(Event::Ctrl('C')) => break,
                Ok(Event::Tab) => self.select_line(),
                Ok(Event::Arrow(Direction::Up)) => self.line = std::cmp::max(self.line - 1, 0),
                Ok(Event::Arrow(Direction::Down)) => self.line = std::cmp::min(self.line + 1, height as isize - 1),
                Ok(Event::Chars(ref s)) => self.query.push_str(s),
                Ok(Event::BackSpace) | Ok(Event::Delete) => {
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
                        .iter()
                        .enumerate()
                        .flat_map(|(i, e)| {
                            if e.data.find(&self.query).is_some() {
                                Some((i, e.clone()))
                            } else {
                                None
                            }
                        })
                        .collect();
        self.term.move_to(0, 0)?;
        self.term.print(&format!("> {}{}", self.query, blank))?;
        for l in 0..self.term.matrix.height - 1 {
            self.term.move_to(0, l + 1)?;
            match self.view.get(l) {
                Some((_, e)) => {
                    if l == self.line as usize {
                        self.term.print_to_color(width,
                                                  &format!("{}{}", e.data, blank),
                                                  0xffffff,
                                                  if e.selected { 0x9acd32 } else { 0x008080 })?;
                    } else {
                        if e.selected {
                            self.term
                                .print_to_color(width, &format!("{}{}", e.data, blank), 0xffffff, 0xff6347)?;
                        }
                        self.term.print_to(width, &format!("{}{}", e.data, blank))?;
                    }
                },
                None => {
                    self.term.print_to(width, &blank)?;
                },
            }
        }
        self.term.move_to(self.term.width_str(&self.query) + 2, 0)
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
