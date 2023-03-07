//! Draws a card.

use super::Draw;
use crate::cards::Card;
use crate::game_state::CardState;

impl Draw {
    pub(crate) fn display_card(
        &mut self,
        card: Card,
        card_state: CardState,
        col: usize,
        row: usize,
    ) {
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
}
