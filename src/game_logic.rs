use crate::cards::{Card, Rank};
use crate::game_state::GameState;
use crate::tui::Selection;

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

fn valid_move_deck_to_pile(pile_index: u8, game_state: &mut GameState) -> Result<(), ()> {
    use Selection::{Deck, Pile};
    let deck_card = Deck.selected_collection(game_state).peek().ok_or(())?;
    let pile_card = Pile { index: pile_index }
        .selected_collection(game_state)
        .peek();

    if deck_card.suit as u8 != pile_index {
        //wrong pile
        return Err(());
    }

    if let Some(pile_card) = pile_card {
        if deck_card.rank as u8 == pile_card.rank as u8 + 1 {
            Ok(())
        } else {
            Err(())
        }
    } else {
        if deck_card.rank == Rank::Ace {
            Ok(())
        } else {
            Err(())
        }
    }
}

pub fn valid_move(from: Selection, to: Selection, game_state: &mut GameState) -> Result<(), ()> {
    use Selection::{Column, Deck, Pile};
    match (from, to) {
        (Deck, Deck) => Err(()),
        (Deck, Pile { index }) => valid_move_deck_to_pile(index, game_state),
        _ => Err(()),
    }
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
