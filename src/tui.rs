use crate::game_state::GameState;
use std::io::{stdin, stdout, Stdin, Stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor};

pub struct Ui {
    stdout: RawTerminal<Stdout>,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            stdout: stdout().into_raw_mode().unwrap(),
        }
    }

    fn display_game_state(&mut self, game_state: &GameState) {
        let (init_col, init_row) = (3u16, 2u16);
        let mut col = init_col;
        for column in &game_state.columns {
            let mut row = init_row;
            for (card, card_state) in &column.0 {
                writeln!(
                    self.stdout,
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

    pub fn run(&mut self, game_state: &mut GameState) {
        let stdin = stdin();
        writeln!(
            self.stdout,
            "{}{}{}{}Solitext",
            clear::All,
            cursor::Goto(1, 1),
            cursor::Hide,
            color::Fg(color::LightYellow),
        )
        .unwrap();

        self.display_game_state(game_state);

        for c in stdin.keys() {
            match c.unwrap() {
                Key::Esc => break,
                Key::Ctrl('c') => break,
                _ => {}
            }
            self.stdout.flush().unwrap();
        }

        write!(self.stdout, "{}", cursor::Show).unwrap();
    }
}
