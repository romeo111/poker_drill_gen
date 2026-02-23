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

/// True if there are 2+ cards within a tight rank span (consecutive or 1-gap),
/// indicating open-ender or gutshot potential.
pub fn has_straight_draw(board: &[Card]) -> bool {
    let mut ranks: Vec<u8> = board.iter().map(|c| c.rank.0).collect();
    ranks.sort_unstable();
    ranks.dedup();
    // Check for any 2 cards with a gap of exactly 1 (consecutive)
    // or 3 cards within a span of 4 (covering connected textures)
    for window in ranks.windows(2) {
        if window[1] - window[0] == 1 {
            return true;
        }
    }
    if ranks.len() >= 3 {
        for window in ranks.windows(3) {
            if window[2] - window[0] <= 4 {
                return true;
            }
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

// ---------------------------------------------------------------------------
// Hand strength classification (5-category, used by preflop + anti-limper)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandCategory {
    Premium,   // AA, KK, QQ, AKs
    Strong,    // JJ, TT, AQo, AKo, AQs
    Playable,  // 99-77, AJs, KQs, suited connectors
    Marginal,  // 66-22, offsuit broadway, weak aces
    Trash,
}

impl std::fmt::Display for HandCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hand_category_name(*self))
    }
}

pub fn classify_hand(hand: [Card; 2]) -> HandCategory {
    let (r1, r2) = {
        let mut ranks = [hand[0].rank.0, hand[1].rank.0];
        ranks.sort_unstable_by(|a, b| b.cmp(a));
        (ranks[0], ranks[1])
    };
    let suited = hand[0].suit == hand[1].suit;
    let pair = r1 == r2;

    if pair {
        return match r1 {
            14 | 13 | 12 => HandCategory::Premium,
            11 | 10      => HandCategory::Strong,
            7..=9        => HandCategory::Playable,
            _            => HandCategory::Marginal,
        };
    }

    match (r1, r2, suited) {
        (14, 13, true)                              => HandCategory::Premium,
        (14, 13, false)                             => HandCategory::Strong,
        (14, 12, true)                              => HandCategory::Strong,
        (14, 12, false)                             => HandCategory::Strong,
        (14, 11, true)                              => HandCategory::Playable,
        (14, r, true) if r >= 9                     => HandCategory::Playable,
        (13, 12, true)                              => HandCategory::Playable,
        (13, 12, false)                             => HandCategory::Marginal,
        (r1, r2, true) if r1 >= 9 && r1 - r2 == 1  => HandCategory::Playable,
        (r1, _, _) if r1 <= 9                       => HandCategory::Trash,
        _                                           => HandCategory::Marginal,
    }
}

pub fn hand_category_name(cat: HandCategory) -> &'static str {
    match cat {
        HandCategory::Premium  => "premium",
        HandCategory::Strong   => "strong",
        HandCategory::Playable => "playable",
        HandCategory::Marginal => "marginal",
        HandCategory::Trash    => "trash",
    }
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
