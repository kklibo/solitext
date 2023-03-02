use crate::cards::{Card, Suit};
use crate::game_logic;
use crate::game_state::GameState;
use crate::game_state::{CardCollection, CardState};
use std::io::{stdin, stdout, Stdout, Write};
use std::{thread, time};
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
    pub fn select_up(&mut self, game_state: &GameState, debug_mode: bool) {
        *self = match *self {
            x @ Self::Deck => x,
            Self::Column {
                index,
                mut card_count,
            } => {
                card_count += 1;
                Self::Column { index, card_count }
            }
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

    pub fn apply_column_selection_rules(&mut self, game_state: &GameState, debug_mode: bool) {
        if let Self::Column { index, card_count } = *self {
            // Prevent size zero selection for non-empty column
            if !game_state.columns[index as usize].0.is_empty() && card_count == 0 {
                *self = Self::Column {
                    index,
                    card_count: 1,
                };
                return;
            }

            let max_count = if debug_mode {
                // In debug mode, allow selection of face-down cards
                game_state.columns[index as usize].0.len()
            } else {
                // Only allow selection of face-up cards
                game_state.columns[index as usize].face_up_cards()
            };

            //todo: fix usize -> u8 conversion
            *self = Self::Column {
                index,
                card_count: std::cmp::min(card_count, max_count as u8),
            }
        }
    }

    /// Get the selected card collection
    pub fn selected_collection<'a>(
        &'a self,
        game_state: &'a mut GameState,
    ) -> &mut dyn CardCollection {
        match self {
            Self::Deck => &mut game_state.deck_drawn,
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
    context_help_message: String,
    debug_message: String,
    debug_mode: bool,
}

enum UiState {
    Intro,
    NewGame,
    Game,
    Victory,
    Quit,
}

enum CardColumnScroll {
    AtMaxRow,
    AtMinRow,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            ui_state: UiState::Intro,
            stdout: stdout().into_raw_mode().unwrap(),
            cursor: Selection::Deck,
            selected: None,
            context_help_message: "".to_string(),
            debug_message: "".to_string(),
            debug_mode: false,
        }
    }
    pub fn reset_for_new_game(&mut self) {
        self.cursor = Selection::Deck;
        self.selected = None;
        self.debug_message.clear();
        self.context_help_message.clear();
    }

    fn display_game_state(&mut self, game_state: &GameState) {
        writeln!(self.stdout, "{}", clear::All,).unwrap();
        self.set_colors(Self::default_fg(), Self::default_bg());

        self.display_info(game_state);
        self.display_deck(game_state);
        self.display_columns(game_state);
        self.display_piles(game_state);

        self.set_colors(color::Blue, Self::default_bg());
        self.display_column_selection_cursor();

        self.set_colors(Self::default_fg(), color::LightGreen);
        self.display_card_selection_cursor(self.cursor, game_state);

        self.set_colors(Self::default_fg(), color::LightYellow);
        if let Some(selected) = self.selected {
            self.display_card_selection_cursor(selected, game_state);
        }

        self.set_colors(Self::default_fg(), Self::default_bg());
    }

    fn default_bg() -> impl color::Color {
        color::Black
    }
    fn default_fg() -> impl color::Color {
        color::LightWhite
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
        writeln!(self.stdout, "{}", color::Bg(Self::default_bg()),).unwrap();
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
        let length = game_state.columns[index as usize].0.len();
        let (scroll, _) =
            Self::scrolled_column_offset(length, card_count as usize).unwrap_or((0, None));

        let upper = Self::COLUMNS_INIT_ROW
            + Self::COLUMNS_ROW_STEP
                * length
                    .checked_sub(scroll)
                    .expect("column scroll should not exceed total cards") as u16;
        let lower = upper
            .checked_sub(card_count as u16)
            .expect("should not select nonexistent cards");

        // Don't draw past the end of the column
        let upper = std::cmp::min(
            upper,
            Self::COLUMNS_INIT_ROW + Self::COLUMNS_ROW_STEP * Self::COLUMN_MAX_VISIBLE_CARDS as u16,
        );

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

    fn display_card(
        &mut self,
        card: Card,
        card_state: CardState,
        col: u16,
        row: u16,
        game_state: &GameState,
    ) {
        use termion::color::*;
        let text = match card_state {
            CardState::FaceUp => {
                if card.suit.is_red() {
                    writeln!(self.stdout, "{}{}", Fg(Red), Bg(White)).unwrap();
                } else {
                    writeln!(self.stdout, "{}{}", Fg(Black), Bg(White)).unwrap();
                }
                card.to_string()
            }
            CardState::FaceDown => {
                if self.debug_mode {
                    if card.suit.is_red() {
                        writeln!(self.stdout, "{}{}", Fg(LightRed), Bg(Black)).unwrap();
                    } else {
                        writeln!(self.stdout, "{}{}", Fg(LightBlack), Bg(Black)).unwrap();
                    }
                    card.to_string()
                } else {
                    writeln!(self.stdout, "{}{}", Fg(LightGreen), Bg(LightBlack)).unwrap();
                    "st".to_string()
                }
            }
        };

        writeln!(self.stdout, "{}{}", cursor::Goto(col, row), text).unwrap();
    }

    const COLUMN_MAX_VISIBLE_CARDS: usize = 7;
    /// Scrolled offset in card column + position info, or None if not scrolled
    fn scrolled_column_offset(
        cards: usize,
        selected: usize,
    ) -> Option<(usize, Option<CardColumnScroll>)> {
        use std::cmp::min;
        if cards <= Self::COLUMN_MAX_VISIBLE_CARDS {
            return None;
        }

        let range = cards - Self::COLUMN_MAX_VISIBLE_CARDS;
        let a = cards - min(selected, cards);
        let offset = min(a, range);

        let position = match offset {
            0 => Some(CardColumnScroll::AtMaxRow),
            x if x == range => Some(CardColumnScroll::AtMinRow),
            _ => None,
        };

        Some((offset, position))
    }

    /// A scrolled card column's visible cards, or None if not scrolled
    fn scrolled_column(
        cards: &Vec<(Card, CardState)>,
        selected: usize,
    ) -> Option<(Vec<(Card, CardState)>, Option<CardColumnScroll>)> {
        Self::scrolled_column_offset(cards.len(), selected).map(|(offset, position)| {
            (
                cards[offset..offset + Self::COLUMN_MAX_VISIBLE_CARDS].into(),
                position,
            )
        })
    }

    /// A column's active selection count; 0 if not selected.
    fn selection_count(&mut self, column_index: usize) -> usize {
        if let Selection::Column { index, card_count } = self.cursor {
            if column_index == index as usize {
                return card_count as usize;
            }
        }
        if let Some(Selection::Column { index, card_count }) = self.selected {
            if column_index == index as usize {
                return card_count as usize;
            }
        }
        0
    }

    const COLUMNS_INIT_COL: u16 = 8;
    const COLUMNS_INIT_ROW: u16 = 2;
    const COLUMNS_COL_STEP: u16 = 5;
    const COLUMNS_ROW_STEP: u16 = 1;
    fn display_columns(&mut self, game_state: &GameState) {
        use termion::color::*;
        let (init_col, init_row) = (Self::COLUMNS_INIT_COL, Self::COLUMNS_INIT_ROW);
        let mut col = init_col;
        for (index, column) in game_state.columns.iter().enumerate() {
            let mut row = init_row;
            if let Some((cards, position)) =
                Self::scrolled_column(&column.0, self.selection_count(index))
            {
                if !matches!(position, Some(CardColumnScroll::AtMaxRow)) {
                    writeln!(self.stdout, "{}{}", Fg(White), Bg(Green)).unwrap();
                    self.draw_text(col - 1, row, "↑  ↑");
                }
                if !matches!(position, Some(CardColumnScroll::AtMinRow)) {
                    writeln!(self.stdout, "{}{}", Fg(White), Bg(Green)).unwrap();
                    self.draw_text(
                        col - 1,
                        row - 1 + (cards.len() as u16 * Self::COLUMNS_ROW_STEP),
                        "↓  ↓",
                    );
                }

                for (card, card_state) in cards {
                    self.display_card(card, card_state, col, row, game_state);
                    row += Self::COLUMNS_ROW_STEP;
                }
            } else {
                for (card, card_state) in &column.0 {
                    self.display_card(*card, *card_state, col, row, game_state);
                    row += Self::COLUMNS_ROW_STEP;
                }
            }
            col += Self::COLUMNS_COL_STEP;
        }
    }

    const PILES_INIT_COL: u16 = 48;
    const PILES_INIT_ROW: u16 = 2;
    const PILES_ROW_STEP: u16 = 2;
    fn display_piles(&mut self, game_state: &GameState) {
        use color::*;
        let (init_col, init_row) = (Self::PILES_INIT_COL, Self::PILES_INIT_ROW);
        let mut row = init_row;
        for (index, pile) in game_state.card_piles.iter().enumerate() {
            if let Some(card) = pile.0.last() {
                self.display_card(*card, CardState::FaceUp, init_col, row, game_state);
            } else {
                writeln!(self.stdout, "{}{}", Fg(Blue), Bg(LightBlack)).unwrap();
                self.draw_text(
                    init_col,
                    row,
                    format!(
                        "{}_",
                        Suit::from_index(index as u8).expect("pile suit should exist")
                    )
                    .as_str(),
                );
            };

            row += Self::PILES_ROW_STEP;
        }
    }

    const DECK_INIT_COL: u16 = 2;
    const DECK_INIT_ROW: u16 = 2;
    fn display_deck(&mut self, game_state: &GameState) {
        use color::*;
        let (init_col, init_row) = (Self::DECK_INIT_COL, Self::DECK_INIT_ROW);
        if let Some(card) = game_state.deck_drawn.last() {
            self.display_card(*card, CardState::FaceUp, init_col, init_row, game_state);
        } else {
            writeln!(self.stdout, "{}{}", Fg(Green), Bg(LightBlack)).unwrap();
            self.draw_text(init_col, init_row, " O ");
        };
    }

    fn display_info(&mut self, game_state: &GameState) {
        use color::*;
        use cursor::*;

        writeln!(self.stdout, "{}{}Solitext", Goto(1, 1), Fg(LightYellow),).unwrap();
        writeln!(
            self.stdout,
            "{}{}h: Help  Esc: Quit",
            Goto(32, 1),
            Fg(LightBlack),
        )
        .unwrap();

        self.set_colors(LightBlack, Self::default_bg());
        let (col, row) = (2, Self::CURSOR_ROW + 1);
        self.draw_text(col, row, "Space: Select/Move cards");

        let (col, row) = (2, Self::CURSOR_ROW + 2);
        writeln!(
            self.stdout,
            "{}{}{}",
            Goto(col, row),
            Fg(LightBlack),
            self.context_help_message
        )
        .unwrap();

        if self.debug_mode {
            let (col, row) = (2, Self::CURSOR_ROW + 3);
            writeln!(
                self.stdout,
                "{}{}debug: {}",
                Goto(col, row),
                Fg(LightBlack),
                self.debug_message
            )
            .unwrap();
        }
    }

    fn draw_box(&mut self, col1: u16, row1: u16, col2: u16, row2: u16) {
        use std::cmp::{max, min};
        for col in min(col1, col2)..=max(col1, col2) {
            for row in min(row1, row2)..=max(row1, row2) {
                writeln!(self.stdout, "{}█", cursor::Goto(col, row)).unwrap();
            }
        }
    }

    fn draw_text(&mut self, col: u16, row: u16, text: &str) {
        writeln!(self.stdout, "{}{}", cursor::Goto(col, row), text).unwrap();
    }

    fn set_up_terminal(&mut self) {
        write!(
            self.stdout,
            "{}{}{}{}{}",
            color::Fg(Self::default_fg()),
            color::Bg(Self::default_bg()),
            clear::All,
            cursor::Goto(1, 1),
            cursor::Hide,
        )
        .unwrap();
        self.stdout.flush().unwrap();
    }

    fn restore_terminal(&mut self) {
        write!(
            self.stdout,
            "{}{}{}{}{}",
            color::Fg(color::Reset),
            color::Bg(color::Reset),
            clear::All,
            cursor::Goto(1, 1),
            cursor::Show,
        )
        .unwrap();
        self.stdout.flush().unwrap();
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
        } else if self.cursor.card_count() > 0 {
            self.selected = Some(self.cursor);
        }
    }

    fn enter_key_action(&mut self, game_state: &mut GameState) {
        if matches!(self.cursor, Selection::Deck) {
            game_state.deck_hit();
        } else if let Selection::Column { index, .. } = self.cursor {
            let from = Selection::Column {
                index,
                card_count: 1,
            };
            //todo: refactor this
            for i in 0..4 {
                let to = Selection::Pile { index: i };
                if game_logic::valid_move(from, to, game_state).is_ok() {
                    let _ = Self::move_cards(from, to, game_state);
                }
            }
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

    fn apply_column_selection_rules(&mut self, game_state: &mut GameState) {
        self.cursor
            .apply_column_selection_rules(game_state, self.debug_mode);
        if let Some(mut selected) = self.selected {
            selected.apply_column_selection_rules(game_state, self.debug_mode);
        }
    }

    fn set_context_help_message(&mut self, game_state: &mut GameState) {
        self.context_help_message = match self.cursor {
            Selection::Deck => "Enter: Hit",
            Selection::Column { .. } => "Enter: Try to Move to Stack",
            _ => "",
        }
        .to_string()
    }

    /// Actions run on each user turn
    /// Returns: true IFF UiState has changed
    fn turn_actions(&mut self, game_state: &mut GameState) -> bool {
        // Ensure a face-up card at the end of each column
        game_logic::face_up_on_columns(game_state);
        // Hit if the deck has cards and the drawn deck is empty
        game_state.auto_hit();
        // Fix column selections, if needed
        self.apply_column_selection_rules(game_state);
        // Update context help line
        self.set_context_help_message(game_state);

        // (Any other automatic state changes can go here too)

        if game_logic::victory(game_state) {
            self.debug_message = "Victory".to_string();
            self.ui_state = UiState::Victory;
            return true;
        }

        self.display_game_state(game_state);
        false
    }

    fn run_game(&mut self, game_state: &mut GameState) {
        if self.turn_actions(game_state) {
            return;
        }

        let stdin = stdin();
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Left => self.cursor.move_left(game_state),
                Key::Right => self.cursor.move_right(game_state),
                Key::Up => self.cursor.select_up(game_state, self.debug_mode),
                Key::Down => self.cursor.select_down(game_state),
                Key::Home => self.cursor = Selection::Deck,
                Key::End => self.cursor = Selection::Pile { index: 0 },
                Key::Char(' ') => self.cards_action(game_state),
                Key::Char('\n') => self.enter_key_action(game_state),
                Key::Char('c') if self.debug_mode => self.debug_unchecked_cards_action(game_state),
                Key::Char('x') => self.selected = None,
                Key::Char('z') if self.debug_mode => self.debug_check_valid(game_state),
                Key::Char('d') => self.debug_mode = !self.debug_mode,
                Key::Char('h') => self.run_help(game_state),
                Key::Esc | Key::Ctrl('c') => {
                    self.ui_state = UiState::Quit;
                    break;
                }
                _ => {}
            }
            if self.turn_actions(game_state) {
                return;
            }
        }
    }

    fn display_victory_message(&mut self, game_state: &mut GameState) {
        const CENTER: (u16, u16) = (26, 5);
        const WIDTH_VAL: u16 = 3;
        fn draw_box(s: &mut Ui, size: u16) {
            s.draw_box(
                CENTER.0 - WIDTH_VAL - size,
                CENTER.1 - size,
                CENTER.0 + WIDTH_VAL + size,
                CENTER.1 + size,
            );
        }
        fn pause() {
            thread::sleep(time::Duration::from_millis(300));
        }

        self.set_colors(color::Blue, Self::default_bg());
        draw_box(self, 3);
        pause();
        self.set_colors(color::Green, Self::default_bg());
        draw_box(self, 2);
        pause();
        self.set_colors(color::Red, Self::default_bg());
        draw_box(self, 1);
        pause();

        self.set_colors(color::LightYellow, color::LightBlue);
        self.draw_text(CENTER.0 - 3, CENTER.1, "YOU WIN");
        pause();
        pause();
        self.set_colors(Self::default_fg(), Self::default_bg());
        self.draw_text(CENTER.0 - 8, CENTER.1 + 4, "Play again? (y/n)");
    }

    fn display_victory(&mut self, game_state: &mut GameState) {
        writeln!(self.stdout, "{}", clear::All).unwrap();
        //just display cards
        self.display_deck(game_state);
        self.display_columns(game_state);
        self.display_piles(game_state);

        self.display_victory_message(game_state);

        self.set_colors(Self::default_fg(), Self::default_bg());
        self.stdout.flush().unwrap();
    }

    fn display_intro(&mut self) {
        fn pause() {
            thread::sleep(time::Duration::from_millis(500));
        }

        writeln!(self.stdout, "{}", clear::All).unwrap();
        self.set_colors(Self::default_fg(), Self::default_bg());

        self.draw_text(1, 1, "haha you ran this program");
        pause();
        pause();
        pause();
        self.draw_text(10, 3, "NOW");
        pause();
        self.draw_text(30, 5, "YOU");
        pause();
        self.draw_text(12, 7, "MUST");
        pause();
        self.draw_text(32, 9, "PLAY");
        pause();
        pause();
        pause();
        pause();

        self.set_colors(Self::default_fg(), Self::default_bg());
        self.stdout.flush().unwrap();
    }

    fn run_intro(&mut self, game_state: &mut GameState) {
        self.display_intro();
        self.ui_state = UiState::NewGame;
    }

    fn run_victory(&mut self, game_state: &mut GameState) {
        self.display_victory(game_state);

        let stdin = stdin();
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('y') => {
                    self.ui_state = UiState::NewGame;
                    break;
                }
                Key::Char('n') | Key::Esc | Key::Ctrl('c') => {
                    self.ui_state = UiState::Quit;
                    break;
                }
                _ => {}
            }

            self.stdout.flush().unwrap();
        }
    }

    pub fn run_new_game(&mut self, game_state: &mut GameState) {
        *game_state = GameState::init(Card::shuffled_deck());
        self.reset_for_new_game();
        self.ui_state = UiState::Game;
    }

    fn display_help(&mut self, game_state: &mut GameState) {
        writeln!(self.stdout, "{}", clear::All).unwrap();
        //just display cards
        self.display_deck(game_state);
        self.display_columns(game_state);
        self.display_piles(game_state);

        const CENTER: (u16, u16) = (26, 5);
        const WIDTH_VAL: u16 = 15;
        fn draw_box(s: &mut Ui, size: u16) {
            s.draw_box(
                CENTER.0 - WIDTH_VAL - size,
                CENTER.1 - size,
                CENTER.0 + WIDTH_VAL + size,
                CENTER.1 + size,
            );
        }
        self.set_colors(color::LightBlue, Self::default_bg());
        draw_box(self, 4);
        self.set_colors(color::White, Self::default_bg());
        draw_box(self, 3);

        self.set_colors(color::LightBlack, color::White);
        const INIT_TEXT: (u16, u16) = (8, 2);
        let (mut col, mut row) = INIT_TEXT;
        self.draw_text(col, row, "Controls:");
        row += 1;
        row += 1;
        col += 1;
        self.draw_text(col, row, "Arrow keys, Home, End: Move cursor");
        row += 1;
        self.draw_text(col, row, "Enter: Hit/move card to stack");
        row += 1;
        self.draw_text(col, row, "Space: Select/move cards");
        row += 1;
        self.draw_text(col, row, "x: Clear selection");
        row += 1;
        self.draw_text(col, row, "Esc, Ctrl+c: Quit");

        self.set_colors(Self::default_fg(), Self::default_bg());
        self.stdout.flush().unwrap();
    }

    pub fn run_help(&mut self, game_state: &mut GameState) {
        self.display_help(game_state);
        stdin().keys().next();
    }

    pub fn run(&mut self, game_state: &mut GameState) {
        self.set_up_terminal();

        loop {
            match self.ui_state {
                UiState::Intro => self.run_intro(game_state),
                UiState::NewGame => self.run_new_game(game_state),
                UiState::Game => self.run_game(game_state),
                UiState::Victory => self.run_victory(game_state),
                UiState::Quit => break,
            }
        }

        self.restore_terminal();
        self.draw_text(1, 1, "please send bug reports via IRC or ham radio");
        self.draw_text(1, 1, "");
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
