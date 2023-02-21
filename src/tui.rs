use crate::game_state::GameState;
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor};

fn display_game_state<W: Write>(game_state: &GameState, stdout: &mut RawTerminal<W>) {
    let (init_col, init_row) = (3u16, 2u16);
    let mut col = init_col;
    for column in &game_state.columns {
        let mut row = init_row;
        for (card, card_state) in &column.0 {
            writeln!(
                stdout,
                "{}{}{}{}",
                cursor::Goto(col, row),
                cursor::Hide,
                color::Fg(color::Green),
                card
            )
            .unwrap();

            row += 1;
        }
        col += 5;
    }
}

pub fn run_ui(game_state: &mut GameState) {
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

    display_game_state(game_state, &mut stdout);

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
