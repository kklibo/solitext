use crate::cards::Card;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum CardState {
    #[default]
    FaceUp,
    FaceDown,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct CardColumn(Vec<(Card, CardState)>);

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct CardPile(Vec<Card>);

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct GameState {
    deck: Vec<Card>,
    columns: [CardColumn; 7],
    card_piles: [CardPile; 4],
}

impl GameState {
    pub fn init(mut deck: Vec<Card>) -> Self {
        let mut columns: [CardColumn; 7] = Default::default();
        let mut card_piles: [CardPile; 4] = Default::default();

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
