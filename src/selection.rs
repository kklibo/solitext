
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
    pub fn select_up(&mut self) {
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