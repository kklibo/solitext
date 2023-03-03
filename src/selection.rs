use crate::game_state::{CardCollection, GameState};
use std::cmp::{max, min};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Selection {
    Deck,
    Column { index: usize, card_count: usize },
    Pile { index: usize },
}

impl Selection {
    fn new_column(index: usize, card_count: usize) -> Selection {
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
    }

    /// for the Left key
    pub fn move_left(&mut self) {
        #[allow(clippy::assertions_on_constants)]
        {
            assert!(GameState::COLUMN_COUNT > 0);
        }
        *self = match *self {
            x @ Self::Deck => x,
            Self::Column { index, .. } if index > 0 => Self::new_column(index - 1, 0),
            Self::Column { .. } => Self::Deck,
            Self::Pile { .. } => Self::new_column(GameState::COLUMN_COUNT - 1, 0),
        };
    }

    /// for the Right key
    pub fn move_right(&mut self) {
        #[allow(clippy::assertions_on_constants)]
        {
            assert!(GameState::COLUMN_COUNT > 0);
        }
        *self = match *self {
            Self::Deck => Self::new_column(0, 0),
            Self::Column { index, .. } if index < GameState::COLUMN_COUNT - 1 => {
                Self::new_column(index + 1, 0)
            }
            Self::Column { .. } => Self::Pile { index: 0 },
            x @ Self::Pile { .. } => x,
        };
    }

    /// for the Up key
    pub fn select_up(&mut self) {
        *self = match *self {
            Self::Deck => Self::Deck,
            Self::Column { index, card_count } => Self::Column {
                index,
                card_count: card_count + 1,
            },
            Self::Pile { index } => Self::Pile {
                index: max(1, index) - 1,
            },
        }
    }

    /// for the Down key
    pub fn select_down(&mut self) {
        *self = match *self {
            Self::Deck => Self::Deck,
            Self::Column { index, card_count } => Self::Column {
                index,
                card_count: max(1, card_count) - 1,
            },
            Self::Pile { index } => Self::Pile {
                index: min(GameState::CARD_PILES_COUNT - 1, index + 1),
            },
        }
    }

    pub fn apply_column_selection_rules(&mut self, game_state: &GameState, debug_mode: bool) {
        if let Self::Column { index, card_count } = *self {
            // Prevent size zero selection for non-empty column
            if !game_state.columns[index].0.is_empty() && card_count == 0 {
                *self = Self::Column {
                    index,
                    card_count: 1,
                };
                return;
            }

            let max_count = if debug_mode {
                // In debug mode, allow selection of face-down cards
                game_state.columns[index].0.len()
            } else {
                // Only allow selection of face-up cards
                game_state.columns[index].face_up_cards()
            };

            *self = Self::Column {
                index,
                card_count: min(card_count, max_count),
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
                .get_mut(*index)
                .expect("selected card column should exist"),
            Self::Pile { index } => game_state
                .card_piles
                .get_mut(*index)
                .expect("selected card pile should exist"),
        }
    }
}
