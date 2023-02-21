use crate::cards::Card;

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

impl GameState {
    pub const COLUMN_COUNT: u8 = 7;
    pub const CARD_PILES_COUNT: u8 = 4;

    pub fn init(mut deck: Vec<Card>) -> Self {
        let mut columns: [CardColumn; Self::COLUMN_COUNT as usize] = Default::default();
        let mut card_piles: [CardPile; Self::CARD_PILES_COUNT as usize] = Default::default();

        for i in 0..columns.len() {
            for _ in 0..=i {
                columns[i].0.push((
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
}
