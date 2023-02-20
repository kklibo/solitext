use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, color, cursor};

pub fn run_ui() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    writeln!(
        stdout,
        "{}{}{}{}Solitext",
        clear::All,
        cursor::Goto(1, 1),
        cursor::Hide,
        color::Fg(color::LightYellow),
    )
    .unwrap();

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Esc => break,
            Key::Ctrl('c') => break,
            _ => {}
        }
        stdout.flush().unwrap();
    }

    write!(stdout, "{}", cursor::Show).unwrap();
}
