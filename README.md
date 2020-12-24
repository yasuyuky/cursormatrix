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
            _ => (),
        },
        &Event::Raw(Input::Arrow(Direction::Up)) => term.move_up().unwrap(),
        &Event::Raw(Input::Arrow(Direction::Down)) => term.move_down().unwrap(),
        &Event::Raw(Input::BackSpace) => term.cursor.backspace().unwrap(),
        &Event::Raw(Input::Chars(ref s)) => term.print(&s).unwrap(),
        &Event::TermSize(w, h) => term.matrix.refresh(w, h),
        _ => (),
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
