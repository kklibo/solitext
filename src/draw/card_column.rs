//! Draws a column of cards in the tableau.

use super::Draw;
use crate::cards::Card;
use crate::game_state::{CardState, GameState};
use crate::selection::Selection;
use std::cmp::min;
use termion::color::*;

enum CardColumnScroll {
    AtMaxRow,
    AtMinRow,
}

struct ScrolledColumn {
    visible_cards: Vec<(Card, CardState)>,
    at_edge: Option<CardColumnScroll>,
}

impl Draw {
    pub(super) const COLUMNS_INIT_COL: usize = 8;
    pub(super) const COLUMNS_INIT_ROW: usize = 2;
    pub(super) const COLUMNS_COL_STEP: usize = 5;
    pub(super) const COLUMNS_ROW_STEP: usize = 1;
    pub(super) fn display_columns(&mut self, game_state: &GameState) {
        let (mut col, init_row) = (Self::COLUMNS_INIT_COL, Self::COLUMNS_INIT_ROW);
        for (index, column) in game_state.columns.iter().enumerate() {
            let mut row = init_row;
            if let Some(ScrolledColumn {
                visible_cards,
                at_edge,
            }) = Self::scrolled_column(&column.0, self.selection_count(index))
            {
                if !matches!(at_edge, Some(CardColumnScroll::AtMaxRow)) {
                    self.set_colors(White, Green);
                    self.draw_text(col - 1, row, "↑  ↑");
                }
                if !matches!(at_edge, Some(CardColumnScroll::AtMinRow)) {
                    self.set_colors(White, Green);
                    self.draw_text(
                        col - 1,
                        row - 1 + (visible_cards.len() * Self::COLUMNS_ROW_STEP),
                        "↓  ↓",
                    );
                }

                for (card, card_state) in visible_cards {
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

    const COLUMN_MAX_VISIBLE_CARDS: usize = 7;
    /// Scrolled offset in card column + position info, or None if not scrolled
    fn scrolled_column_offset(
        cards: usize,
        selected: usize,
    ) -> Option<(usize, Option<CardColumnScroll>)> {
        if cards <= Self::COLUMN_MAX_VISIBLE_CARDS {
            return None;
        }

        let max_offset = cards - Self::COLUMN_MAX_VISIBLE_CARDS;
        let unselected = cards - min(selected, cards);
        let offset = min(unselected, max_offset);

        let position = match offset {
            0 => Some(CardColumnScroll::AtMaxRow),
            x if x == max_offset => Some(CardColumnScroll::AtMinRow),
            _ => None,
        };

        Some((offset, position))
    }

    /// A scrolled card column's visible cards, or None if not scrolled
    fn scrolled_column(cards: &Vec<(Card, CardState)>, selected: usize) -> Option<ScrolledColumn> {
        Self::scrolled_column_offset(cards.len(), selected).map(|(offset, at_edge)| {
            ScrolledColumn {
                visible_cards: cards[offset..offset + Self::COLUMN_MAX_VISIBLE_CARDS].into(),
                at_edge,
            }
        })
    }

    /// A column's active selection count; 0 if not selected.
    fn selection_count(&mut self, column_index: usize) -> usize {
        if let Selection::Column { index, card_count } = self.cursor {
            if column_index == index {
                return card_count;
            }
        }
        if let Some(Selection::Column { index, card_count }) = self.selected {
            if column_index == index {
                return card_count;
            }
        }
        0
    }

    pub(super) fn draw_card_column_selection_cursor(
        &mut self,
        game_state: &GameState,
        col: usize,
        index: usize,
        card_count: usize,
    ) {
        let length = game_state.columns[index].0.len();
        let (scroll, _) = Self::scrolled_column_offset(length, card_count).unwrap_or((0, None));

        let upper = Self::COLUMNS_INIT_ROW
            + Self::COLUMNS_ROW_STEP
                * length
                    .checked_sub(scroll)
                    .expect("column scroll should not exceed total cards");
        let lower = upper
            .checked_sub(card_count)
            .expect("should not select nonexistent cards");

        // Don't draw past the end of the column
        let upper = min(
            upper,
            Self::COLUMNS_INIT_ROW + Self::COLUMNS_ROW_STEP * Self::COLUMN_MAX_VISIBLE_CARDS,
        );

        for row in lower..upper {
            self.draw_text(col - 1, row, "[");
            self.draw_text(col + 3, row, "]");
        }
    }
}
