use crate::training_engine::models::{Card, Suit};

/// Describes the texture of a flop/board for human-readable explanations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardTexture {
    /// All three suits different, no pair, no connected cards.
    Dry,
    /// Two cards of the same suit present.
    SemiWet,
    /// Flush possible and/or straight possible.
    Wet,
}

impl std::fmt::Display for BoardTexture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoardTexture::Dry     => write!(f, "dry"),
            BoardTexture::SemiWet => write!(f, "semi-wet"),
            BoardTexture::Wet     => write!(f, "wet"),
        }
    }
}

/// Classify the texture of up to 5 board cards.
pub fn board_texture(board: &[Card]) -> BoardTexture {
    if board.is_empty() {
        return BoardTexture::Dry;
    }

    let flush_draw = has_flush_draw(board);
    let straight_draw = has_straight_draw(board);

    if flush_draw || straight_draw {
        // Two or more draws = wet; one draw = semi-wet
        if flush_draw && straight_draw {
            BoardTexture::Wet
        } else {
            BoardTexture::SemiWet
        }
    } else {
        BoardTexture::Dry
    }
}

/// True if 2+ cards share a suit (flush draw possible).
pub fn has_flush_draw(board: &[Card]) -> bool {
    let mut counts = [0u8; 4]; // clubs, diamonds, hearts, spades
    for c in board {
        let idx = match c.suit {
            Suit::Clubs    => 0,
            Suit::Diamonds => 1,
            Suit::Hearts   => 2,
            Suit::Spades   => 3,
        };
        counts[idx] += 1;
        if counts[idx] >= 2 {
            return true;
        }
    }
    false
}

/// True if there are 2 consecutively ranked cards (open-ender/gutshot possible).
pub fn has_straight_draw(board: &[Card]) -> bool {
    let mut ranks: Vec<u8> = board.iter().map(|c| c.rank.0).collect();
    ranks.sort_unstable();
    ranks.dedup();
    for window in ranks.windows(2) {
        if window[1] - window[0] <= 2 {
            return true;
        }
    }
    false
}

/// Approximate hero equity for a flush draw (standard ~36% on flop, ~20% on turn).
pub fn flush_draw_equity(streets_remaining: u8) -> f32 {
    match streets_remaining {
        2 => 0.35,
        1 => 0.20,
        _ => 0.0,
    }
}

/// Approximate hero equity for an open-ended straight draw.
pub fn oesd_equity(streets_remaining: u8) -> f32 {
    match streets_remaining {
        2 => 0.32,
        1 => 0.17,
        _ => 0.0,
    }
}

/// Approximate hero equity for a combo draw (flush + straight draw).
pub fn combo_draw_equity(streets_remaining: u8) -> f32 {
    match streets_remaining {
        2 => 0.54,
        1 => 0.30,
        _ => 0.0,
    }
}

/// Compute pot odds as a fraction: `call / (pot + call)`.
/// Returns required equity to break even.
pub fn required_equity(call_amount: u32, pot_before_call: u32) -> f32 {
    let total = pot_before_call + call_amount;
    if total == 0 {
        return 0.0;
    }
    call_amount as f32 / total as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::training_engine::models::{Card, Rank, Suit};

    fn card(r: u8, s: Suit) -> Card {
        Card { rank: Rank(r), suit: s }
    }

    #[test]
    fn dry_board_detected() {
        let board = vec![
            card(2, Suit::Clubs),
            card(7, Suit::Diamonds),
            card(13, Suit::Hearts),
        ];
        assert_eq!(board_texture(&board), BoardTexture::Dry);
    }

    #[test]
    fn flush_draw_board_detected() {
        let board = vec![
            card(2, Suit::Clubs),
            card(7, Suit::Clubs),
            card(13, Suit::Hearts),
        ];
        assert_eq!(board_texture(&board), BoardTexture::SemiWet);
    }

    #[test]
    fn pot_odds_calculation() {
        // 100 pot, 50 call → need 33% equity
        let eq = required_equity(50, 100);
        assert!((eq - 0.333).abs() < 0.01);
    }
}
