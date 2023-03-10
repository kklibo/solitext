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

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum GameMode {
    #[default]
    DrawOne,
    DrawThree,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct GameState {
    pub game_mode: GameMode,
    pub deck: Vec<Card>,
    pub deck_drawn: Vec<Card>,
    pub columns: [CardColumn; Self::COLUMN_COUNT],
    pub card_piles: [CardPile; Self::CARD_PILES_COUNT],
}

pub trait CardCollection {
    fn take(&mut self, cards_count: usize) -> Result<Vec<Card>, ()>;
    fn receive(&mut self, cards: Vec<Card>) -> Result<(), ()>;
    fn peek(&self) -> Option<Card>;
    fn peek_n(&self, count: usize) -> Option<Vec<Card>>;
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
    fn peek(&self) -> Option<Card> {
        self.0.last().map(|&(card, _)| card)
    }
    fn peek_n(&self, count: usize) -> Option<Vec<Card>> {
        let start_index = self.0.len().checked_sub(count)?;
        let cards = self.0.get(start_index..)?;
        Some(cards.iter().map(|(card, _)| *card).collect())
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
    fn peek(&self) -> Option<Card> {
        self.0.last().copied()
    }
    fn peek_n(&self, count: usize) -> Option<Vec<Card>> {
        let start_index = self.0.len().checked_sub(count)?;
        self.0.get(start_index..).map(Into::into)
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
    fn peek(&self) -> Option<Card> {
        self.last().copied()
    }
    fn peek_n(&self, count: usize) -> Option<Vec<Card>> {
        let start_index = self.len().checked_sub(count)?;
        self.get(start_index..).map(Into::into)
    }
}

impl CardColumn {
    pub fn face_up_cards(&self) -> usize {
        self.0
            .iter()
            .rev()
            .take_while(|(_, card_state)| *card_state == CardState::FaceUp)
            .count()
    }
}

impl GameState {
    pub const COLUMN_COUNT: usize = 7;
    pub const CARD_PILES_COUNT: usize = 4;

    pub fn init(mut deck: Vec<Card>) -> Self {
        let mut columns: [CardColumn; Self::COLUMN_COUNT] = Default::default();
        let card_piles: [CardPile; Self::CARD_PILES_COUNT] = Default::default();

        for (i, column) in columns.iter_mut().enumerate() {
            for _ in 0..=i {
                column.0.push((
                    deck.pop().expect("deck should have enough cards to deal"),
                    CardState::FaceDown,
                ));
            }
        }

        Self {
            game_mode: Default::default(),
            deck,
            deck_drawn: Default::default(),
            columns,
            card_piles,
        }
    }

    pub fn deck_hit(&mut self) {
        if self.deck.is_empty() && !self.deck_drawn.is_empty() {
            self.deck = self.deck_drawn.clone();
            self.deck.reverse();
            self.deck_drawn.clear();
        }

        let count = match self.game_mode {
            GameMode::DrawOne => 1,
            GameMode::DrawThree => 3,
        };

        for _ in 0..count {
            if let Some(card) = self.deck.pop() {
                self.deck_drawn.push(card);
            }
        }
    }

    #[allow(dead_code)]
    pub fn auto_hit(&mut self) {
        if !self.deck.is_empty() && self.deck_drawn.is_empty() {
            self.deck_hit();
        }
    }

    #[allow(dead_code)]
    pub fn victory() -> Self {
        let mut card_piles: [CardPile; Self::CARD_PILES_COUNT] = Default::default();

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

    #[allow(dead_code)]
    pub fn almost_victory() -> Self {
        let mut x = Self::victory();
        x.columns[0]
            .0
            .push((x.card_piles[0].0.pop().unwrap(), CardState::FaceUp));
        x
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

    #[test]
    fn test_card_collection_peek_n() {
        use crate::cards::{Rank::*, Suit::*};
        let a = GameState::init(Card::ordered_deck());

        let peek3 = a.columns[6].peek_n(3);
        let expected = vec![
            Card::new(Diamonds, Ace),
            Card::new(Spades, King),
            Card::new(Spades, Queen),
        ];
        assert_eq!(peek3, Some(expected));

        let peek_too_many = a.columns[0].peek_n(2);
        assert_eq!(peek_too_many, None);
    }
}
