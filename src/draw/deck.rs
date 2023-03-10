//! Draws the stock & wastepile decks.

use super::Draw;
use crate::game_state::{CardState, GameMode, GameState};
use termion::color;

impl Draw {
    pub(super) fn draw_deck_selection_cursor(&mut self, col: usize, row: usize) {
        self.draw_text(col + 2, row, "◂");
        self.draw_text(col - 2, row, "▸");
    }

    fn max_visible_cards(game_mode: GameMode) -> usize {
        match game_mode {
            GameMode::DrawOne => 1,
            GameMode::DrawThree => Self::DECK_DRAWN_MAX_DISPLAY_CARDS,
        }
    }

    pub(super) const DECK_INIT_COL: usize = 2;
    const DECK_INIT_ROW: usize = 2;
    const DECK_DRAWN_STEP: usize = 2;
    const DECK_ROW_STEP: usize = 1;
    const DECK_DRAWN_MAX_DISPLAY_CARDS: usize = 3;
    pub(super) fn display_deck(&mut self, game_state: &GameState) {
        use color::*;
        let (col, mut row) = (Self::DECK_INIT_COL, Self::DECK_INIT_ROW);
        if let Some(card) = game_state.deck.last() {
            self.display_card(*card, CardState::FaceDown, col, row);
        } else {
            self.set_colors(Green, LightBlack);
            self.draw_text(col, row, " O ");
        };

        let max_cards = Self::max_visible_cards(game_state.game_mode);

        // display up to `max_cards` cards from the top of the drawn pile
        row += Self::DECK_DRAWN_STEP;
        for card in game_state.deck_drawn.iter().rev().take(max_cards).rev() {
            self.display_card(*card, CardState::FaceUp, col, row);
            row += Self::DECK_ROW_STEP;
        }
    }

    pub(super) fn deck_selection_cursor_row(game_state: &GameState) -> Option<usize> {
        let max_cards = Self::max_visible_cards(game_state.game_mode);

        let displayed_cards = game_state.deck_drawn.iter().take(max_cards).count();
        if displayed_cards > 0 {
            Some(
                Self::DECK_INIT_ROW
                    + Self::DECK_DRAWN_STEP
                    + Self::DECK_ROW_STEP * (displayed_cards - 1),
            )
        } else {
            None
        }
    }
}
