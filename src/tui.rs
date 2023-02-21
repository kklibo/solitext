use crate::game_state::GameState;
use std::io::{stdin, stdout, Stdin, Stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Selection {
    Deck,
    Column { index: u8, card_count: u8 },
    Pile { index: u8 },
}

impl Selection {
    pub fn move_right(&mut self) {
        *self = match self {
            Self::Deck => Self::Column {
                index: 0,
                card_count: 0,
            },
            Self::Column { index, card_count } if *index < GameState::COLUMN_COUNT - 1 => {
                Self::Column {
                    index: *index + 1,
                    card_count: *card_count,
                }
            }
            Self::Column { .. } => Self::Pile { index: 0 },
            Self::Pile { index } => Self::Pile { index: *index },
        };
    }
}

pub struct Ui {
    stdout: RawTerminal<Stdout>,
    cursor: Selection,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            stdout: stdout().into_raw_mode().unwrap(),
            cursor: Selection::Deck,
        }
    }

    fn display_game_state(&mut self, game_state: &GameState) {
        writeln!(self.stdout, "{}", clear::All,).unwrap();

        self.display_deck(game_state);
        self.display_columns(game_state);
        self.display_piles(game_state);
        self.display_selection_cursor();
    }

    const CURSOR_ROW: u16 = 10;
    fn display_selection_cursor(&mut self) {
        let col = match self.cursor {
            Selection::Deck => Self::DECK_INIT_COL,
            Selection::Column { index, .. } => {
                Self::COLUMNS_INIT_COL + (index as u16) * Self::COLUMNS_COL_STEP
            }
            Selection::Pile { .. } => Self::PILES_INIT_COL,
        };

        writeln!(
            self.stdout,
            "{}{}{}█↑█",
            cursor::Goto(col, Self::CURSOR_ROW),
            cursor::Hide,
            color::Fg(color::Blue),
        )
        .unwrap();
    }

    const COLUMNS_INIT_COL: u16 = 8;
    const COLUMNS_INIT_ROW: u16 = 2;
    const COLUMNS_COL_STEP: u16 = 5;
    fn display_columns(&mut self, game_state: &GameState) {
        let (init_col, init_row) = (Self::COLUMNS_INIT_COL, Self::COLUMNS_INIT_ROW);
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
            col += Self::COLUMNS_COL_STEP;
        }
    }

    const PILES_INIT_COL: u16 = 48;
    const PILES_INIT_ROW: u16 = 2;
    fn display_piles(&mut self, game_state: &GameState) {
        let (init_col, init_row) = (Self::PILES_INIT_COL, Self::PILES_INIT_ROW);
        let mut row = init_row;
        for pile in &game_state.card_piles {
            let top = if let Some(card) = pile.0.last() {
                card.to_string()
            } else {
                "_".to_string()
            };

            writeln!(
                self.stdout,
                "{}{}{}{}",
                cursor::Goto(init_col, row),
                cursor::Hide,
                color::Fg(color::Green),
                top
            )
            .unwrap();

            row += 2;
        }
    }

    const DECK_INIT_COL: u16 = 2;
    const DECK_INIT_ROW: u16 = 2;
    fn display_deck(&mut self, game_state: &GameState) {
        let (init_col, init_row) = (Self::DECK_INIT_COL, Self::DECK_INIT_ROW);

        let top = if let Some(card) = game_state.deck.last() {
            card.to_string()
        } else {
            "_".to_string()
        };

        writeln!(
            self.stdout,
            "{}{}{}{}",
            cursor::Goto(init_col, init_row),
            cursor::Hide,
            color::Fg(color::Green),
            top
        )
        .unwrap();
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
                Key::Right => self.cursor.move_right(),
                Key::Esc => break,
                Key::Ctrl('c') => break,
                _ => {}
            }
            self.display_game_state(game_state);
            self.stdout.flush().unwrap();
        }

        write!(self.stdout, "{}", cursor::Show).unwrap();
    }
}
