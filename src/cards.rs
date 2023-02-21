use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt::{Display, Formatter};
use strum::{EnumIter, IntoEnumIterator};

#[derive(EnumIter, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Rank {
    Ace = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
}

impl Rank {
    pub fn is_odd(self) -> bool {
        self as u8 % 2 == 1
    }
}

impl Display for Rank {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let v = match self {
            Rank::Ace => "A",
            Rank::R2 => "2",
            Rank::R3 => "3",
            Rank::R4 => "4",
            Rank::R5 => "5",
            Rank::R6 => "6",
            Rank::R7 => "7",
            Rank::R8 => "8",
            Rank::R9 => "9",
            Rank::R10 => "10",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
        };
        write!(f, "{v}")
    }
}

#[derive(EnumIter, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Suit {
    Hearts = 0,
    Spades = 1,
    Diamonds = 2,
    Clubs = 3,
}

impl Suit {
    pub fn is_red(self) -> bool {
        match self {
            Self::Hearts | Self::Diamonds => true,
            Self::Spades | Self::Clubs => false,
        }
    }
}

impl Display for Suit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let v = match self {
            Self::Hearts => "♥",
            Self::Spades => "♠",
            Self::Diamonds => "♦",
            Self::Clubs => "♣",
        };
        write!(f, "{v}")
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Card {
    suit: Suit,
    rank: Rank,
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

impl Card {
    pub fn ordered_deck() -> Vec<Self> {
        let mut cards = vec![];
        for suit in Suit::iter() {
            for rank in Rank::iter() {
                cards.push(Card { suit, rank });
            }
        }
        cards
    }

    pub fn shuffled_deck() -> Vec<Self> {
        let mut rng = thread_rng();
        let mut deck = Self::ordered_deck();
        deck.shuffle(&mut rng);
        deck
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(Rank::Ace => true)]
    #[test_case(Rank::R2 => false)]
    fn test_rank_is_odd(rank: Rank) -> bool {
        rank.is_odd()
    }

    #[test]
    fn test_ordered_deck() {
        const PRINT: bool = true;
        let cards = Card::ordered_deck();
        if PRINT {
            print!("Black: ");
            for card in &cards {
                if !card.suit.is_red() {
                    print!("{card} ");
                }
            }
            println!();
            print!("Red:   ");
            for card in &cards {
                if card.suit.is_red() {
                    print!("{card} ");
                }
            }
            println!();
        }
        assert_eq!(cards.len(), 52);
    }

    #[test]
    fn test_shuffled_deck() {
        const PRINT: bool = true;
        let cards = Card::shuffled_deck();
        if PRINT {
            print!("Black: ");
            for card in &cards {
                if !card.suit.is_red() {
                    print!("{card} ");
                }
            }
            println!();
            print!("Red:   ");
            for card in &cards {
                if card.suit.is_red() {
                    print!("{card} ");
                }
            }
            println!();
        }
        assert_eq!(cards.len(), 52);
    }
}
