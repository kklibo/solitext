use crate::cards::{Card, Suit};
use crate::game_state::{CardState, GameState};
use crate::selection::Selection;
use std::io::{stdout, Stdout, Write};
use std::{thread, time};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor};

pub struct Draw {
    stdout: RawTerminal<Stdout>,
    pub cursor: Selection,
    pub selected: Option<Selection>,
    pub context_help_message: String,
    pub debug_message: String,
    pub debug_mode: bool,
}

enum CardColumnScroll {
    AtMaxRow,
    AtMinRow,
}

impl Draw {
    pub fn new() -> Self {
        Self {
            stdout: stdout().into_raw_mode().unwrap(),
            cursor: Selection::Deck,
            selected: None,
            context_help_message: "".to_string(),
            debug_message: "".to_string(),
            debug_mode: false,
        }
    }

    pub fn display_game_state(&mut self, game_state: &GameState) {
        self.clear_screen();
        self.set_colors(Self::default_fg(), Self::default_bg());

        self.display_info();
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

    fn clear_screen(&mut self) {
        writeln!(self.stdout, "{}", clear::All,).unwrap();
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
        self.draw_text(col, Self::CURSOR_ROW, "█↑█");
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
    }

    fn draw_deck_selection_cursor(&mut self, col: u16, row: u16) {
        self.draw_text(col + 2, row, "◂");
        self.draw_text(col - 2, row, "▸");
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
            self.draw_text(col - 1, row, "[");
            self.draw_text(col + 3, row, "]");
        }
    }

    fn draw_pile_selection_cursor(&mut self, col: u16, index: u8) {
        let row = Self::PILES_INIT_ROW + Self::PILES_ROW_STEP * index as u16;
        self.draw_text(col - 1, row, "[");
        self.draw_text(col + 3, row, "]");
    }

    fn display_card(&mut self, card: Card, card_state: CardState, col: u16, row: u16) {
        use termion::color::*;
        let text = match card_state {
            CardState::FaceUp => {
                if card.suit.is_red() {
                    self.set_colors(Red, White);
                } else {
                    self.set_colors(Black, White);
                }
                card.to_string()
            }
            CardState::FaceDown => {
                if self.debug_mode {
                    if card.suit.is_red() {
                        self.set_colors(LightRed, Black);
                    } else {
                        self.set_colors(LightBlack, Black);
                    }
                    card.to_string()
                } else {
                    self.set_colors(LightGreen, LightBlack);
                    "st".to_string()
                }
            }
        };

        self.draw_text(col, row, text.as_str());
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
                    self.set_colors(White, Green);
                    self.draw_text(col - 1, row, "↑  ↑");
                }
                if !matches!(position, Some(CardColumnScroll::AtMinRow)) {
                    self.set_colors(White, Green);
                    self.draw_text(
                        col - 1,
                        row - 1 + (cards.len() as u16 * Self::COLUMNS_ROW_STEP),
                        "↓  ↓",
                    );
                }

                for (card, card_state) in cards {
                    self.display_card(card, card_state, col, row);
                    row += Self::COLUMNS_ROW_STEP;
                }
            } else {
                for (card, card_state) in &column.0 {
                    self.display_card(*card, *card_state, col, row);
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
                self.display_card(*card, CardState::FaceUp, init_col, row);
            } else {
                self.set_colors(Blue, LightBlack);
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
            self.display_card(*card, CardState::FaceUp, init_col, init_row);
        } else {
            self.set_colors(Green, LightBlack);
            self.draw_text(init_col, init_row, " O ");
        };
    }

    fn display_info(&mut self) {
        use color::*;

        self.set_colors(LightYellow, Self::default_bg());
        self.draw_text(1, 1, "Solitext");

        self.set_colors(LightBlack, Self::default_bg());
        self.draw_text(32, 1, "h: Help  Esc: Quit");
        self.draw_text(2, Self::CURSOR_ROW + 1, "Space: Select/Move cards");
        self.draw_text(
            2,
            Self::CURSOR_ROW + 2,
            self.context_help_message.clone().as_str(),
        );
        if self.debug_mode {
            self.draw_text(2, Self::CURSOR_ROW + 3, self.debug_message.clone().as_str());
        }
    }

    fn draw_box(&mut self, col1: u16, row1: u16, col2: u16, row2: u16) {
        use std::cmp::{max, min};
        for col in min(col1, col2)..=max(col1, col2) {
            for row in min(row1, row2)..=max(row1, row2) {
                self.draw_text(col, row, "█");
            }
        }
    }

    pub fn draw_text(&mut self, col: u16, row: u16, text: &str) {
        writeln!(self.stdout, "{}{}", cursor::Goto(col, row), text).unwrap();
    }

    pub fn set_up_terminal(&mut self) {
        writeln!(
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

    pub fn restore_terminal(&mut self) {
        writeln!(
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

    fn display_victory_message(&mut self) {
        const CENTER: (u16, u16) = (26, 5);
        const WIDTH_VAL: u16 = 3;
        fn draw_box(s: &mut Draw, size: u16) {
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

    pub fn display_victory(&mut self, game_state: &mut GameState) {
        self.clear_screen();
        //just display cards
        self.display_deck(game_state);
        self.display_columns(game_state);
        self.display_piles(game_state);

        self.display_victory_message();

        self.set_colors(Self::default_fg(), Self::default_bg());
        self.stdout.flush().unwrap();
    }

    pub fn display_intro(&mut self) {
        fn pause() {
            thread::sleep(time::Duration::from_millis(500));
        }

        self.clear_screen();
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

    pub fn display_help(&mut self, game_state: &mut GameState) {
        self.clear_screen();
        //just display cards
        self.display_deck(game_state);
        self.display_columns(game_state);
        self.display_piles(game_state);

        const CENTER: (u16, u16) = (26, 5);
        const WIDTH_VAL: u16 = 15;
        fn draw_box(s: &mut Draw, size: u16) {
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
}
