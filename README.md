# cursormatrix

TUI Library for Rust

![ci status](https://github.com/yasuyuky/cursormatrix/workflows/Rust/badge.svg)

## Example usage

```rust
use cursormatrix::{Direction, Event, Input, Term};

fn handle_event(ev: &Event, term: &mut Term) -> bool {
    match ev {
        &Event::Ctrl(Input::Chars(ref s)) => match s.as_str() {
            "C" => return false,
            "L" => term.clear().unwrap(),
            "A" => term.move_home().unwrap(),
            "E" => term.move_end().unwrap(),
            "D" => term.cursor.delete_char().unwrap(),
            k => term.print(&format!("Ctrl+{}", k)).unwrap(),
        },
        &Event::Raw(Input::Arrow(Direction::Up)) => term.move_up().unwrap(),
        &Event::Raw(Input::Arrow(Direction::Down)) => term.move_down().unwrap(),
        &Event::Raw(Input::Arrow(Direction::Left)) => term.move_left().unwrap(),
        &Event::Raw(Input::Arrow(Direction::Right)) => term.move_right().unwrap(),
        &Event::Raw(Input::Return) => {
            term.print("↩︎").unwrap();
            term.move_home().unwrap();
            term.move_down().unwrap();
        },
        &Event::Raw(Input::BackSpace) => term.cursor.backspace().unwrap(),
        &Event::Raw(Input::Chars(ref s)) => {
            let cs: Vec<String> = s.chars().map(|c| format!("{:02x}", c as usize)).collect();
            term.print(&format!("{}", s)).unwrap();
        },
        &Event::TermSize(w, h) => term.matrix.refresh(w, h),
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
```

## Test

```console
cargo test
```

## Example

```console
cargo run example --debug
```

```console
cat FILE | cargo run example --filter
```
