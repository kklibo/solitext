use crate::game_logic;
use crate::game_state::CardCollection;
use crate::game_state::GameState;
use std::io::{stdin, stdout, Stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Selection {
    Deck,
    Column { index: u8, card_count: u8 },
    Pile { index: u8 },
}

impl Selection {
    fn card_column(index: u8, game_state: &GameState) -> Selection {
        let card_count = if game_state.column_is_empty(index) {
            0
        } else {
            1
        };
        Self::Column { index, card_count }
    }

    /// Is this selection in the same Deck, Column, or Pile as `other`?
    /// (I.e.: are variant and index equal?)
    pub fn same_collection(&self, other: Self) -> bool {
        match self {
            Self::Deck => matches!(other, Self::Deck),
            Self::Column {
                index: self_index, ..
            } => {
                if let Self::Column {
                    index: other_index, ..
                } = other
                {
                    *self_index == other_index
                } else {
                    false
                }
            }
            Self::Pile { index: self_index } => {
                if let Self::Pile { index: other_index } = other {
                    *self_index == other_index
                } else {
                    false
                }
            }
        }
    }

    /// Number of cards selected
    pub fn card_count(&self) -> usize {
        match self {
            Self::Column { card_count, .. } => *card_count,
            _ => 1,
        }
        .into()
    }

    /// for the Left key
    pub fn move_left(&mut self, game_state: &GameState) {
        #[allow(clippy::assertions_on_constants)]
        {
            assert!(GameState::COLUMN_COUNT > 0);
        }
        *self = match *self {
            x @ Self::Deck => x,
            Self::Column { index, .. } if index > 0 => Self::card_column(index - 1, game_state),
            Self::Column { .. } => Self::Deck,
            Self::Pile { .. } => Self::card_column(GameState::COLUMN_COUNT - 1, game_state),
        };
    }

    /// for the Right key
    pub fn move_right(&mut self, game_state: &GameState) {
        #[allow(clippy::assertions_on_constants)]
        {
            assert!(GameState::COLUMN_COUNT > 0);
        }
        *self = match *self {
            Self::Deck => Self::card_column(0, game_state),
            Self::Column { index, .. } if index < GameState::COLUMN_COUNT - 1 => {
                Self::card_column(index + 1, game_state)
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

    /// Get the selected card collection
    pub fn selected_collection<'a>(
        &'a self,
        game_state: &'a mut GameState,
    ) -> &mut dyn CardCollection {
        match self {
            Self::Deck => &mut game_state.deck,
            Self::Column { index, .. } => game_state
                .columns
                .get_mut(*index as usize)
                .expect("selected card column should exist"),
            Self::Pile { index } => game_state
                .card_piles
                .get_mut(*index as usize)
                .expect("selected card pile should exist"),
        }
    }
}

pub struct Ui {
    ui_state: UiState,
    stdout: RawTerminal<Stdout>,
    cursor: Selection,
    selected: Option<Selection>,
    debug_message: String,
}

enum UiState {
    Intro,
    Game,
    Victory,
    Quit,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            ui_state: UiState::Game,
            stdout: stdout().into_raw_mode().unwrap(),
            cursor: Selection::Deck,
            selected: None,
            debug_message: "".to_string(),
        }
    }

    fn display_game_state(&mut self, game_state: &GameState) {
        writeln!(self.stdout, "{}", clear::All,).unwrap();

        self.display_info(game_state);
        self.display_deck(game_state);
        self.display_columns(game_state);
        self.display_piles(game_state);

        self.set_colors(color::Blue, color::Reset);
        self.display_column_selection_cursor();

        self.set_colors(color::Reset, color::LightGreen);
        self.display_card_selection_cursor(self.cursor, game_state);

        self.set_colors(color::Reset, color::LightYellow);
        if let Some(selected) = self.selected {
            self.display_card_selection_cursor(selected, game_state);
        }

        self.set_colors(color::Reset, color::Reset);
    }

    fn set_colors(&mut self, foreground: impl color::Color, background: impl color::Color) {
        writeln!(
            self.stdout,
            "{}{}",
            color::Fg(foreground),
            color::Bg(background),
        )
        .unwrap();
    }

    fn selection_col(selection: Selection) -> u16 {
        match selection {
            Selection::Deck => Self::DECK_INIT_COL,
            Selection::Column { index, .. } => {
                Self::COLUMNS_INIT_COL + (index as u16) * Self::COLUMNS_COL_STEP
            }
            Selection::Pile { .. } => Self::PILES_INIT_COL,
        }
    }

    const CURSOR_ROW: u16 = 10;
    fn display_column_selection_cursor(&mut self) {
        let col = Self::selection_col(self.cursor);

        writeln!(self.stdout, "{}█↑█", cursor::Goto(col, Self::CURSOR_ROW),).unwrap();
    }

    fn display_card_selection_cursor(&mut self, selection: Selection, game_state: &GameState) {
        let col = Self::selection_col(selection);

        match selection {
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
        writeln!(self.stdout, "{}{ch}", cursor::Goto(col, row),).unwrap();
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
            "{}{}{}",
            cursor::Goto(init_col, init_row),
            color::Fg(color::Green),
            top
        )
        .unwrap();
    }

    fn display_info(&mut self, game_state: &GameState) {
        writeln!(
            self.stdout,
            "{}{}Solitext",
            cursor::Goto(1, 1),
            color::Fg(color::LightYellow),
        )
        .unwrap();
        writeln!(
            self.stdout,
            "{}{}debug: {}",
            cursor::Goto(2, Self::CURSOR_ROW + 2),
            color::Fg(color::LightBlack),
            self.debug_message
        )
        .unwrap();
    }

    fn set_up_terminal(&mut self) {
        writeln!(self.stdout, "{}", cursor::Hide).unwrap();
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

    fn move_cards(from: Selection, to: Selection, game_state: &mut GameState) -> Result<(), ()> {
        if from.same_collection(to) {
            return Err(());
        }

        let cards = from
            .selected_collection(game_state)
            .take(from.card_count())?;

        to.selected_collection(game_state).receive(cards)?;
        Ok(())
    }

    fn cards_action(&mut self, game_state: &mut GameState) {
        if let (Some(from), to) = (self.selected, self.cursor) {
            self.selected = None;

            if game_logic::valid_move(from, to, game_state).is_ok() {
                match Self::move_cards(from, to, game_state) {
                    Ok(_) => self.debug_message = "move OK".to_string(),
                    Err(_) => self.debug_message = "move attempt failed".to_string(),
                }
            } else {
                self.debug_message = "invalid move".to_string();
            }
        } else {
            self.selected = Some(self.cursor)
        }
    }

    fn debug_unchecked_cards_action(&mut self, game_state: &mut GameState) {
        if let Some(selected) = self.selected {
            self.selected = None;
            let _ = Self::move_cards(selected, self.cursor, game_state);
        } else {
            self.selected = Some(self.cursor)
        }
    }

    fn debug_check_valid(&mut self, game_state: &mut GameState) {
        if let (Some(from), to) = (self.selected, self.cursor) {
            self.debug_message = format!("{:?}", game_logic::valid_move(from, to, game_state));
        } else {
            self.debug_message = "".to_string();
        }
    }

    fn run_game(&mut self, game_state: &mut GameState) {
        self.display_game_state(game_state);

        let stdin = stdin();
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Left => self.cursor.move_left(game_state),
                Key::Right => self.cursor.move_right(game_state),
                Key::Up => self.cursor.select_up(game_state),
                Key::Down => self.cursor.select_down(game_state),
                Key::Char(' ') => self.cards_action(game_state),
                Key::Char('c') => self.debug_unchecked_cards_action(game_state),
                Key::Char('x') => self.selected = None,
                Key::Char('z') => self.debug_check_valid(game_state),
                Key::Esc | Key::Ctrl('c') => {
                    self.ui_state = UiState::Quit;
                    break;
                }
                _ => {}
            }

            if game_logic::victory(game_state) {
                self.debug_message = "Victory".to_string();
            }

            self.display_game_state(game_state);
            self.stdout.flush().unwrap();
        }
    }

    pub fn run(&mut self, game_state: &mut GameState) {
        self.set_up_terminal();
        *game_state = GameState::almost_victory();

        loop {
            match self.ui_state {
                UiState::Game => self.run_game(game_state),
                UiState::Quit => break,
                _ => todo!(),
            }
        }

        self.restore_terminal();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::Card;

    #[test]
    fn test_same_collection() {
        assert!(!Selection::Deck.same_collection(Selection::Column {
            index: 1,
            card_count: 1
        }));
        assert!(!Selection::Column {
            index: 1,
            card_count: 1
        }
        .same_collection(Selection::Pile { index: 1 }));
        assert!(!Selection::Pile { index: 1 }.same_collection(Selection::Deck));

        assert!(Selection::Deck.same_collection(Selection::Deck));

        assert!(Selection::Column {
            index: 1,
            card_count: 1
        }
        .same_collection(Selection::Column {
            index: 1,
            card_count: 1
        }));
        assert!(Selection::Column {
            index: 1,
            card_count: 2
        }
        .same_collection(Selection::Column {
            index: 1,
            card_count: 1
        }));
        assert!(!Selection::Column {
            index: 2,
            card_count: 1
        }
        .same_collection(Selection::Column {
            index: 1,
            card_count: 1
        }));

        assert!(Selection::Pile { index: 1 }.same_collection(Selection::Pile { index: 1 }));
        assert!(!Selection::Pile { index: 2 }.same_collection(Selection::Pile { index: 1 }));
    }

    #[test]
    fn test_selected_collection() {
        let mut a = GameState::init(Card::ordered_deck());
        let b = Selection::Deck.selected_collection(&mut a);
    }
}
