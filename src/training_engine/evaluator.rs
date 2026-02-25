//! Board analysis, draw detection, pot-odds math, and hand classification.
//!
//! This module contains all the poker-analysis primitives shared across topic
//! generators.  No topic module should duplicate this logic.
//!
//! ## Board texture
//! `board_texture()` classifies a board as Dry, SemiWet, or Wet based on flush
//! and straight draw potential.  C-bet sizing in flop topics is driven by this.
//!
//! ## Draw classification
//! `DrawType` + `classify_draw()` identify the strongest draw present on a
//! board.  Used by pot-odds (T3), semi-bluff (T8), and check-raise (T7).
//! `draw_equity_flop()` returns approximate equity for each draw type.
//!
//! ## Hand classification (5-category)
//! `HandCategory` + `classify_hand()` sort a 2-card hand into Premium / Strong /
//! Playable / Marginal / Trash.  Used by preflop topics (T1, T9, T11, T12).
//!
//! ## Pot odds
//! `required_equity()` computes the minimum equity needed to break even on a
//! call: `call / (pot + call)`.

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
// Hand strength classification (5-category)
//
// Used by preflop topics to decide the correct action.  The classification
// is intentionally coarse — it captures the strategic tier of a starting hand
// without full equity calculations:
//
//   Premium  — AA, KK, QQ, AKs             → always raise / 3-bet / push
//   Strong   — JJ, TT, AQo, AKo, AQs       → raise, call 3-bets
//   Playable — 99-77, AJs, KQs, suited conn → open, call, sometimes fold
//   Marginal — 66-22, KJo, weak aces        → fold or limp, rarely raise
//   Trash    — everything else               → always fold facing action
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

// ---------------------------------------------------------------------------
// Suit index helper
//
// Suit has no numeric representation by design — we use an explicit match
// to convert to an array index.  Never cast Suit as usize.
// ---------------------------------------------------------------------------

/// Convert Suit to array index (0–3).  Uses explicit match, not `as usize`.
pub fn suit_index(s: Suit) -> usize {
    match s {
        Suit::Clubs    => 0,
        Suit::Diamonds => 1,
        Suit::Hearts   => 2,
        Suit::Spades   => 3,
    }
}

// ---------------------------------------------------------------------------
// Draw classification
//
// Shared by pot-odds (T3), semi-bluff (T8), and check-raise (T7) topics.
// ComboDraw (flush + straight) is the strongest, GutShot the weakest.
// `draw_equity_flop()` returns approximate equity with 2 streets remaining.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawType {
    ComboDraw,
    FlushDraw,
    OESD,
    GutShot,
}

impl std::fmt::Display for DrawType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DrawType::ComboDraw => write!(f, "combo draw (flush + straight)"),
            DrawType::FlushDraw => write!(f, "flush draw"),
            DrawType::OESD      => write!(f, "open-ended straight draw"),
            DrawType::GutShot   => write!(f, "gutshot straight draw"),
        }
    }
}

/// Classify the draw type present on the board.
pub fn classify_draw(board: &[Card]) -> DrawType {
    match (has_flush_draw(board), has_straight_draw(board)) {
        (true, true)  => DrawType::ComboDraw,
        (true, false) => DrawType::FlushDraw,
        (false, true) => DrawType::OESD,
        _             => DrawType::GutShot,
    }
}

/// Approximate flop equity for a given draw type (2 streets remaining).
pub fn draw_equity_flop(dt: DrawType) -> f32 {
    match dt {
        DrawType::ComboDraw => 0.54,
        DrawType::FlushDraw => 0.35,
        DrawType::OESD      => 0.32,
        DrawType::GutShot   => 0.17,
    }
}

/// True if hero holds a card matching a suit with 2+ board cards.
pub fn hero_has_flush_draw(hand: [Card; 2], board: &[Card]) -> bool {
    let mut counts = [0u8; 4];
    for c in board { counts[suit_index(c.suit)] += 1; }
    hand.iter().any(|c| counts[suit_index(c.suit)] >= 2)
}

/// True if hero participates in a straight draw on the board.
pub fn hero_has_straight_draw(hand: [Card; 2], board: &[Card]) -> bool {
    if !has_straight_draw(board) { return false; }
    let board_ranks: Vec<u8> = board.iter().map(|c| c.rank.0).collect();
    hand.iter().any(|hc| board_ranks.iter().any(|&br| {
        let diff = if hc.rank.0 > br { hc.rank.0 - br } else { br - hc.rank.0 };
        diff <= 3
    }))
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
