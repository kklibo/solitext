use crate::game_state::GameState;
use std::io::{stdin, stdout, Stdout, Write};
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
    /// for the Left key
    pub fn move_left(&mut self) {
        #[allow(clippy::assertions_on_constants)]
        {
            assert!(GameState::COLUMN_COUNT > 0);
        }
        *self = match *self {
            x @ Self::Deck => x,
            Self::Column { index, card_count } if index > 0 => Self::Column {
                index: index - 1,
                card_count,
            },
            Self::Column { .. } => Self::Deck,
            Self::Pile { .. } => Selection::Column {
                index: GameState::COLUMN_COUNT - 1,
                card_count: 0,
            },
        };
    }

    /// for the Right key
    pub fn move_right(&mut self) {
        #[allow(clippy::assertions_on_constants)]
        {
            assert!(GameState::COLUMN_COUNT > 0);
        }
        *self = match *self {
            Self::Deck => Self::Column {
                index: 0,
                card_count: 0,
            },
            Self::Column { index, card_count } if index < GameState::COLUMN_COUNT - 1 => {
                Self::Column {
                    index: index + 1,
                    card_count,
                }
            }
            Self::Column { .. } => Self::Pile { index: 0 },
            x @ Self::Pile { .. } => x,
        };
    }

    /// for the Up key
    pub fn select_up(&mut self, game_state: &GameState) {
        *self = match *self {
            x @ Self::Deck => x,
            Self::Column { index, card_count }
                if (card_count as usize) < game_state.columns[index as usize].0.len() =>
            {
                Self::Column {
                    index,
                    card_count: card_count + 1,
                }
            }
            x @ Self::Column { .. } => x,
            Self::Pile { index } if index > 0 => Self::Pile { index: index - 1 },
            x @ Self::Pile { .. } => x,
        }
    }

    /// for the Down key
    pub fn select_down(&mut self, game_state: &GameState) {
        *self = match *self {
            x @ Self::Deck => x,
            Self::Column { index, card_count } if (card_count as usize) > 0 => Self::Column {
                index,
                card_count: card_count - 1,
            },
            x @ Self::Column { .. } => x,
            Self::Pile { index } if (index as usize) < game_state.card_piles.len() - 1 => {
                Self::Pile { index: index + 1 }
            }
            x @ Self::Pile { .. } => x,
        }
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

        self.display_info(game_state);
        self.display_deck(game_state);
        self.display_columns(game_state);
        self.display_piles(game_state);
        self.display_column_selection_cursor();
        self.display_card_selection_cursor(game_state);
    }

    fn selection_col(&mut self) -> u16 {
        match self.cursor {
            Selection::Deck => Self::DECK_INIT_COL,
            Selection::Column { index, .. } => {
                Self::COLUMNS_INIT_COL + (index as u16) * Self::COLUMNS_COL_STEP
            }
            Selection::Pile { .. } => Self::PILES_INIT_COL,
        }
    }

    const CURSOR_ROW: u16 = 10;
    fn display_column_selection_cursor(&mut self) {
        let col = self.selection_col();

        writeln!(
            self.stdout,
            "{}{}{}█↑█",
            cursor::Goto(col, Self::CURSOR_ROW),
            cursor::Hide,
            color::Fg(color::Blue),
        )
        .unwrap();
    }

    fn display_card_selection_cursor(&mut self, game_state: &GameState) {
        let col = self.selection_col();

        match self.cursor {
            Selection::Deck => self.draw_deck_selection_cursor(col, Self::DECK_INIT_ROW),
            Selection::Column { index, card_count } => {
                self.draw_card_column_selection_cursor(game_state, col, index, card_count)
            }
            Selection::Pile { index } => self.draw_pile_selection_cursor(col, index),
        };
        writeln!(self.stdout, "{}", color::Bg(color::Reset),).unwrap();
    }

    fn draw_deck_selection_cursor(&mut self, col: u16, row: u16) {
        self.draw_selection_char(col + 2, row, "◂");
        self.draw_selection_char(col - 2, row, "▸");
    }

    fn draw_card_column_selection_cursor(
        &mut self,
        game_state: &GameState,
        col: u16,
        index: u8,
        card_count: u8,
    ) {
        let upper = Self::COLUMNS_INIT_ROW
            + Self::COLUMNS_ROW_STEP * (game_state.columns[index as usize].0.len()) as u16;
        let lower = upper
            .checked_sub(card_count as u16)
            .expect("should not select nonexistent cards");

        for row in lower..upper {
            self.draw_selection_char(col - 1, row, "[");
            self.draw_selection_char(col + 3, row, "]");
        }
    }

    fn draw_pile_selection_cursor(&mut self, col: u16, index: u8) {
        let row = Self::PILES_INIT_ROW + Self::PILES_ROW_STEP * index as u16;
        self.draw_selection_char(col - 1, row, "[");
        self.draw_selection_char(col + 3, row, "]");
    }

    fn draw_selection_char(&mut self, col: u16, row: u16, ch: &str) {
        writeln!(
            self.stdout,
            "{}{}{}{ch}",
            cursor::Goto(col, row),
            color::Fg(color::Reset),
            color::Bg(color::LightGreen),
        )
        .unwrap();
    }

    const COLUMNS_INIT_COL: u16 = 8;
    const COLUMNS_INIT_ROW: u16 = 2;
    const COLUMNS_COL_STEP: u16 = 5;
    const COLUMNS_ROW_STEP: u16 = 1;
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

                row += Self::COLUMNS_ROW_STEP;
            }
            col += Self::COLUMNS_COL_STEP;
        }
    }

    const PILES_INIT_COL: u16 = 48;
    const PILES_INIT_ROW: u16 = 2;
    const PILES_ROW_STEP: u16 = 2;
    fn display_piles(&mut self, game_state: &GameState) {
        let (init_col, init_row) = (Self::PILES_INIT_COL, Self::PILES_INIT_ROW);
        let mut row = init_row;
        for pile in &game_state.card_piles {
            let top = if let Some(card) = pile.0.last() {
                card.to_string()
            } else {
                " _".to_string()
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

            row += Self::PILES_ROW_STEP;
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

    fn display_info(&mut self, game_state: &GameState) {
        writeln!(
            self.stdout,
            "{}{}{}Solitext",
            cursor::Goto(1, 1),
            cursor::Hide,
            color::Fg(color::LightYellow),
        )
        .unwrap();
    }

    fn restore_terminal(&mut self) {
        write!(
            self.stdout,
            "{}{}{}",
            clear::All,
            cursor::Goto(1, 1),
            cursor::Show,
        )
        .unwrap();
    }

    pub fn run(&mut self, game_state: &mut GameState) {
        let stdin = stdin();
        self.display_game_state(game_state);

        for c in stdin.keys() {
            match c.unwrap() {
                Key::Left => self.cursor.move_left(),
                Key::Right => self.cursor.move_right(),
                Key::Up => self.cursor.select_up(game_state),
                Key::Down => self.cursor.select_down(game_state),
                Key::Esc => break,
                Key::Ctrl('c') => break,
                _ => {}
            }
            self.display_game_state(game_state);
            self.stdout.flush().unwrap();
        }

        self.restore_terminal();
    }
}
