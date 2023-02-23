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
    } else if deck_card.rank == Rank::Ace {
        Ok(())
    } else {
        Err(())
    }
}

fn valid_move_card_to_column(
    card: Card,
    column_index: u8,
    game_state: &mut GameState,
) -> Result<(), ()> {
    use Selection::Column;
    let column_card = Column {
        index: column_index,
        card_count: 0,
    }
    .selected_collection(game_state)
    .peek();

    if let Some(column_card) = column_card {
        if card.rank as u8 + 1 == column_card.rank as u8
            && card.suit.is_red() != column_card.suit.is_red()
        {
            Ok(())
        } else {
            Err(())
        }
    } else if card.rank == Rank::King {
        Ok(())
    } else {
        Err(())
    }
}

fn valid_move_deck_to_column(column_index: u8, game_state: &mut GameState) -> Result<(), ()> {
    use Selection::Deck;
    let deck_card = Deck.selected_collection(game_state).peek().ok_or(())?;
    valid_move_card_to_column(deck_card, column_index, game_state)
}

fn valid_move_column_to_column(
    from_index: u8,
    card_count: u8,
    to_index: u8,
    game_state: &mut GameState,
) -> Result<(), ()> {
    let cards = Selection::Column {
        index: from_index,
        card_count,
    }
    .selected_collection(game_state)
    .peek_n(card_count as usize)
    .ok_or(())?;

    let first_card = cards.first().copied().ok_or(())?;
    valid_move_card_to_column(first_card, to_index, game_state)
}

fn valid_move_column_to_pile(
    column_index: u8,
    card_count: u8,
    pile_index: u8,
    game_state: &mut GameState,
) -> Result<(), ()> {
    use Selection::{Column, Pile};

    if card_count != 1 {
        return Err(());
    }

    let column_card = Column {
        index: column_index,
        card_count,
    }
    .selected_collection(game_state)
    .peek()
    .ok_or(())?;
    if column_card.suit as u8 != pile_index {
        return Err(());
    }

    let pile_card = Pile { index: pile_index }
        .selected_collection(game_state)
        .peek();

    if let Some(pile_card) = pile_card {
        if column_card.rank as u8 == pile_card.rank as u8 + 1 {
            return Ok(());
        }
    } else if column_card.rank == Rank::Ace {
        return Ok(());
    }

    Err(())
}

fn valid_move_pile_to_column(
    pile_index: u8,
    column_index: u8,
    game_state: &mut GameState,
) -> Result<(), ()> {
    let card = Selection::Pile { index: pile_index }
        .selected_collection(game_state)
        .peek()
        .ok_or(())?;

    valid_move_card_to_column(card, column_index, game_state)
}

pub fn valid_move(from: Selection, to: Selection, game_state: &mut GameState) -> Result<(), ()> {
    use Selection::{Column, Deck, Pile};
    match from {
        Deck => match to {
            Deck => Err(()),
            Pile { index } => valid_move_deck_to_pile(index, game_state),
            Column { index, .. } => valid_move_deck_to_column(index, game_state),
        },
        Pile { index } => match to {
            Deck => Err(()),
            Pile { .. } => Err(()),
            Column {
                index: column_index,
                ..
            } => valid_move_pile_to_column(index, column_index, game_state),
        },
        Column { index, card_count } => match to {
            Deck => Err(()),
            Pile { index: pile_index } => {
                valid_move_column_to_pile(index, card_count, pile_index, game_state)
            }
            Column {
                index: to_index, ..
            } => valid_move_column_to_column(index, card_count, to_index, game_state),
        },
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
