use crate::cards::{Card, Rank};
use crate::game_state::GameState;

pub fn victory(game_state: &GameState) -> bool {
    for pile in &game_state.card_piles {
        if let Some(Card { rank, .. }) = pile.0.last() {
            if *rank != Rank::King {
                return false;
            }
        } else {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_victory() {
        assert!(victory(&GameState::victory()));
        assert!(!victory(&GameState::init(Card::ordered_deck())));
    }
}
