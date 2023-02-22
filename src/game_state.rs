use crate::cards::{Card, Rank, Suit};
use strum::IntoEnumIterator;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum CardState {
    #[default]
    FaceUp,
    FaceDown,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct CardColumn(pub Vec<(Card, CardState)>);

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct CardPile(pub Vec<Card>);

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct GameState {
    pub deck: Vec<Card>,
    pub columns: [CardColumn; Self::COLUMN_COUNT as usize],
    pub card_piles: [CardPile; Self::CARD_PILES_COUNT as usize],
}

pub trait CardCollection {
    fn take(&mut self, cards_count: usize) -> Result<Vec<Card>, ()>;
    fn receive(&mut self, cards: Vec<Card>) -> Result<(), ()>;
}

impl CardCollection for CardColumn {
    fn take(&mut self, count: usize) -> Result<Vec<Card>, ()> {
        let mut v = vec![];
        for _ in 0..count {
            let val = self.0.pop().ok_or(())?;
            v.push(val.0);
        }
        v.reverse();

        Ok(v)
    }
    fn receive(&mut self, cards: Vec<Card>) -> Result<(), ()> {
        for card in cards {
            self.0.push((card, CardState::FaceUp));
        }
        Ok(())
    }
}

impl CardCollection for CardPile {
    fn take(&mut self, count: usize) -> Result<Vec<Card>, ()> {
        let mut v = vec![];
        match count {
            1 => {
                let val = self.0.pop().ok_or(())?;
                v.push(val);
            }
            _ => return Err(()),
        }
        v.reverse();

        Ok(v)
    }
    fn receive(&mut self, cards: Vec<Card>) -> Result<(), ()> {
        if cards.len() != 1 {
            return Err(());
        }
        self.0.push(cards[0]);
        Ok(())
    }
}

impl CardCollection for Vec<Card> {
    fn take(&mut self, count: usize) -> Result<Vec<Card>, ()> {
        let mut v = vec![];
        match count {
            1 => {
                let val = self.pop().ok_or(())?;
                v.push(val);
            }
            _ => return Err(()),
        }
        v.reverse();

        Ok(v)
    }
    fn receive(&mut self, cards: Vec<Card>) -> Result<(), ()> {
        if cards.len() != 1 {
            return Err(());
        }
        self.push(cards[0]);
        Ok(())
    }
}

impl GameState {
    pub const COLUMN_COUNT: u8 = 7;
    pub const CARD_PILES_COUNT: u8 = 4;

    pub fn init(mut deck: Vec<Card>) -> Self {
        let mut columns: [CardColumn; Self::COLUMN_COUNT as usize] = Default::default();
        let card_piles: [CardPile; Self::CARD_PILES_COUNT as usize] = Default::default();

        for (i, column) in columns.iter_mut().enumerate() {
            for _ in 0..=i {
                column.0.push((
                    deck.pop().expect("deck should have enough cards to deal"),
                    CardState::FaceDown,
                ));
            }
        }

        Self {
            deck,
            columns,
            card_piles,
        }
    }

    pub fn column_is_empty(&self, index: u8) -> bool {
        self.columns
            .get(index as usize)
            .expect("column should exist")
            .0
            .is_empty()
    }

    pub fn victory() -> Self {
        let mut card_piles: [CardPile; Self::CARD_PILES_COUNT as usize] = Default::default();

        for (index, suit) in Suit::iter().enumerate() {
            for rank in Rank::iter() {
                card_piles
                    .get_mut(index)
                    .expect("card pile should exist")
                    .0
                    .push(Card { suit, rank });
            }
        }

        Self {
            card_piles,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_init() {
        let a = GameState::init(Card::ordered_deck());
        dbg!(&a);
        assert_eq!(24, a.deck.len());
    }

    #[test]
    fn test_card_collection_send_to() {
        let mut a = GameState::init(Card::ordered_deck());
        assert_eq!(3, a.columns[2].0.len());
        assert_eq!(4, a.columns[3].0.len());

        dbg!(&a.columns[2]);
        dbg!(&a.columns[3]);

        let cards = a.columns[2].take(2).unwrap();
        assert_eq!(2, cards.len());
        assert_eq!(1, a.columns[2].0.len());
        assert_eq!(4, a.columns[3].0.len());

        a.columns[3].receive(cards).unwrap();
        assert_eq!(1, a.columns[2].0.len());
        assert_eq!(6, a.columns[3].0.len());

        dbg!(&a.columns[2]);
        dbg!(&a.columns[3]);
    }
}
