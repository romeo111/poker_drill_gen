use rand::Rng;
use crate::training_engine::models::{Card, Rank, Suit};

/// A standard 52-card deck that can be shuffled and dealt from.
pub struct Deck {
    cards: Vec<Card>,
    cursor: usize,
}

impl Deck {
    /// Build a fresh ordered deck and shuffle it with `rng`.
    pub fn new_shuffled<R: Rng>(rng: &mut R) -> Self {
        let suits = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];
        let mut cards: Vec<Card> = suits
            .iter()
            .flat_map(|&suit| (2u8..=14).map(move |r| Card { rank: Rank(r), suit }))
            .collect();

        // Fisher-Yates shuffle
        for i in (1..cards.len()).rev() {
            let j = rng.gen_range(0..=i);
            cards.swap(i, j);
        }

        Deck { cards, cursor: 0 }
    }

    /// Deal one card; panics if the deck is exhausted.
    pub fn deal(&mut self) -> Card {
        assert!(self.cursor < self.cards.len(), "Deck exhausted");
        let card = self.cards[self.cursor];
        self.cursor += 1;
        card
    }

    /// Deal `n` cards at once.
    pub fn deal_n(&mut self, n: usize) -> Vec<Card> {
        (0..n).map(|_| self.deal()).collect()
    }

    /// Remaining cards available.
    pub fn remaining(&self) -> usize {
        self.cards.len() - self.cursor
    }

    /// All dealt cards so far (useful for integrity checks).
    pub fn dealt_cards(&self) -> &[Card] {
        &self.cards[..self.cursor]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn deck_has_52_unique_cards() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut deck = Deck::new_shuffled(&mut rng);
        let all: Vec<Card> = (0..52).map(|_| deck.deal()).collect();

        // All unique
        let mut seen = std::collections::HashSet::new();
        for c in &all {
            let key = (c.rank.0, c.suit as u8);
            assert!(seen.insert(key), "Duplicate card: {}", c);
        }
        assert_eq!(all.len(), 52);
    }

    #[test]
    fn deck_is_deterministic_with_seed() {
        let make = |seed: u64| -> Vec<Card> {
            let mut rng = StdRng::seed_from_u64(seed);
            let mut deck = Deck::new_shuffled(&mut rng);
            deck.deal_n(5)
        };
        assert_eq!(make(99), make(99));
        assert_ne!(make(99), make(100));
    }
}
