mod card;
mod card_column;
mod common;
mod deck;
mod foundation;

use crate::game_state::GameState;
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

    fn selection_col(selection: Selection) -> usize {
        match selection {
            Selection::Deck => Self::DECK_INIT_COL,
            Selection::Column { index, .. } => {
                Self::COLUMNS_INIT_COL + index * Self::COLUMNS_COL_STEP
            }
            Selection::Pile { .. } => Self::PILES_INIT_COL,
        }
    }

    const CURSOR_ROW: usize = 10;
    fn display_column_selection_cursor(&mut self) {
        let col = Self::selection_col(self.cursor);
        self.draw_text(col, Self::CURSOR_ROW, "█↑█");
    }

    fn display_card_selection_cursor(&mut self, selection: Selection, game_state: &GameState) {
        let col = Self::selection_col(selection);

        match selection {
            Selection::Deck => {
                if let Some(row) = Self::deck_selection_cursor_row(game_state) {
                    self.draw_deck_selection_cursor(col, row);
                }
            }
            Selection::Column { index, card_count } => {
                self.draw_card_column_selection_cursor(game_state, col, index, card_count)
            }
            Selection::Pile { index } => self.draw_pile_selection_cursor(col, index),
        };
    }

    fn display_info(&mut self) {
        use color::*;

        self.set_colors(LightYellow, Self::default_bg());
        self.draw_text(1, 1, "Solitext");

        self.set_colors(LightBlack, Self::default_bg());
        self.draw_text(32, 1, "h: Help  Esc: Menu");
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

    fn display_victory_message(&mut self) {
        const CENTER: (usize, usize) = (26, 5);
        const WIDTH_VAL: usize = 3;
        fn draw_box(s: &mut Draw, size: usize) {
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

    pub fn display_start_screen(&mut self) {
        self.clear_screen();
        self.set_colors(color::LightYellow, Self::default_bg());
        self.draw_text(16, 1, "Solitext    ♥ ♠ ♦ ♣");

        let lines = r#"1: New Game (Draw One)
3: New Game (Draw Three)
Esc: Quit"#;
        self.draw_text_box(lines);

        self.set_colors(Self::default_fg(), Self::default_bg());
        self.stdout.flush().unwrap();
    }

    pub fn display_game_menu(&mut self, game_state: &mut GameState) {
        self.clear_screen();
        //just display cards
        self.display_deck(game_state);
        self.display_columns(game_state);
        self.display_piles(game_state);

        let lines = r#"1: New Game (Draw One)
3: New Game (Draw Three)
r: Restart current game
q: Quit
Esc: Return to game"#;
        self.draw_text_box(lines);

        self.set_colors(Self::default_fg(), Self::default_bg());
        self.stdout.flush().unwrap();
    }

    pub fn display_help(&mut self, game_state: &mut GameState) {
        self.clear_screen();
        //just display cards
        self.display_deck(game_state);
        self.display_columns(game_state);
        self.display_piles(game_state);

        let lines = r#"Controls:

 Arrow keys, Home, End: Move cursor
 Enter: Hit/move card to stack
 Space: Select/move cards
 x: Clear selection
 Ctrl+c: Quit"#;
        self.draw_text_box(lines);

        self.set_colors(Self::default_fg(), Self::default_bg());
        self.stdout.flush().unwrap();
    }
}
