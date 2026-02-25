//! All shared data types used across the training engine.
//!
//! This module defines the card primitives (Suit, Rank, Card), game metadata
//! (GameType, Position, PlayerState), and the request/response types that form
//! the public API (TrainingRequest, TrainingScenario, etc.).
//!
//! Every type derives `Serialize` + `Deserialize` so scenarios can be sent over
//! the wire as JSON without any conversion layer.

use std::fmt;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Card primitives
//
// A card is a (Rank, Suit) pair.  Rank stores 2..=14 where 14 = Ace.
// Display formats follow standard notation: "As" = Ace of spades,
// "Tc" = Ten of clubs.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Suit::Clubs => write!(f, "c"),
            Suit::Diamonds => write!(f, "d"),
            Suit::Hearts => write!(f, "h"),
            Suit::Spades => write!(f, "s"),
        }
    }
}

/// Rank 2..=14 where 14 = Ace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Rank(pub u8);

impl Rank {
    pub fn symbol(self) -> &'static str {
        match self.0 {
            2 => "2", 3 => "3", 4 => "4", 5 => "5", 6 => "6",
            7 => "7", 8 => "8", 9 => "9", 10 => "T",
            11 => "J", 12 => "Q", 13 => "K", 14 => "A",
            _ => "?",
        }
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

// ---------------------------------------------------------------------------
// Table / game metadata
//
// GameType distinguishes cash games (fixed blinds) from tournaments (ICM).
// Position encodes all 9-max seats; `is_late()` returns true for CO and BTN
// which act last postflop — a key strategic advantage.
// PlayerState carries per-seat info used by the scenario UI.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameType {
    CashGame,
    Tournament,
}

impl fmt::Display for GameType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameType::CashGame => write!(f, "Cash Game"),
            GameType::Tournament => write!(f, "Tournament"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Position {
    UTG,
    UTG1,
    UTG2,
    LJ,   // Lojack
    HJ,   // Hijack
    CO,   // Cutoff
    BTN,  // Button
    SB,   // Small Blind
    BB,   // Big Blind
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Position::UTG  => "UTG",
            Position::UTG1 => "UTG+1",
            Position::UTG2 => "UTG+2",
            Position::LJ   => "Lojack",
            Position::HJ   => "Hijack",
            Position::CO   => "Cutoff",
            Position::BTN  => "Button",
            Position::SB   => "Small Blind",
            Position::BB   => "Big Blind",
        };
        write!(f, "{}", s)
    }
}

impl Position {
    /// Is this position considered "in position" (acts last postflop)?
    pub fn is_late(self) -> bool {
        matches!(self, Position::CO | Position::BTN)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub seat: u8,
    pub position: Position,
    pub stack: u32,
    pub is_hero: bool,
    pub is_active: bool,
}

// ---------------------------------------------------------------------------
// Training request / response types
//
// TrainingTopic — the 16 poker skills the engine can drill.
// DifficultyLevel — controls stack depth ranges and bet-size variance.
// TextStyle — Simple (beginner-friendly) vs Technical (poker jargon).
// TrainingRequest — input to `generate_training()`.
// TrainingScenario — the full output: table state, question, answers.
// ---------------------------------------------------------------------------

/// The five streets (phases) of a poker hand.
///
/// Use `Street::topics()` to get all training topics for a street.
/// Use `TrainingTopic::street()` to get the street for a topic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Street {
    /// Before community cards — open-raise, 3-bet, squeeze, ICM push/fold.
    Preflop,
    /// First three community cards — c-bet, pot odds, check-raise, semi-bluff.
    Flop,
    /// Fourth community card — barrel, probe bet, delayed c-bet.
    Turn,
    /// Fifth community card — bluff, value bet, call-or-fold.
    River,
}

impl Street {
    /// All training topics that belong to this street.
    pub fn topics(self) -> &'static [TrainingTopic] {
        match self {
            Street::Preflop => &[
                TrainingTopic::PreflopDecision,
                TrainingTopic::ICMAndTournamentDecision,
                TrainingTopic::AntiLimperIsolation,
                TrainingTopic::SqueezePlay,
                TrainingTopic::BigBlindDefense,
            ],
            Street::Flop => &[
                TrainingTopic::PostflopContinuationBet,
                TrainingTopic::PotOddsAndEquity,
                TrainingTopic::CheckRaiseSpot,
                TrainingTopic::SemiBluffDecision,
                TrainingTopic::ThreeBetPotCbet,
            ],
            Street::Turn => &[
                TrainingTopic::TurnBarrelDecision,
                TrainingTopic::TurnProbeBet,
                TrainingTopic::DelayedCbet,
            ],
            Street::River => &[
                TrainingTopic::BluffSpot,
                TrainingTopic::RiverValueBet,
                TrainingTopic::RiverCallOrFold,
            ],
        }
    }
}

impl fmt::Display for Street {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Street::Preflop => write!(f, "Preflop"),
            Street::Flop    => write!(f, "Flop"),
            Street::Turn    => write!(f, "Turn"),
            Street::River   => write!(f, "River"),
        }
    }
}

/// The 16 poker skills the engine can generate drills for.
///
/// Topics are grouped by street in the source code:
/// - **Preflop** (preflop.rs): T1, T5, T9, T11, T12
/// - **Flop** (flop.rs): T2, T3, T7, T8, T13
/// - **Turn** (turn.rs): T6, T15, T16
/// - **River** (river.rs): T4, T10, T14
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrainingTopic {
    /// T1  (PF-) Open-raise vs fold based on hand strength and position.
    PreflopDecision,
    /// T2  (CB-) C-bet sizing on the flop: small on dry boards, large on wet.
    PostflopContinuationBet,
    /// T3  (PO-) Call vs fold with a draw — compare pot odds to draw equity.
    PotOddsAndEquity,
    /// T4  (BL-) River bluff sizing when hero has no showdown value.
    BluffSpot,
    /// T5  (IC-) Tournament push/fold adjusted by ICM pressure.
    ICMAndTournamentDecision,
    /// T6  (TB-) Double-barrel the turn vs check back based on the turn card.
    TurnBarrelDecision,
    /// T7  (CR-) Check-raise, check-call, or fold from the BB on the flop.
    CheckRaiseSpot,
    /// T8  (SB-) Semi-bluff raise with a draw on the flop.
    SemiBluffDecision,
    /// T9  (AL-) Iso-raise a preflop limper vs overlimp vs fold.
    AntiLimperIsolation,
    /// T10 (RV-) River value bet sizing: overbet nuts, large strong, check medium.
    RiverValueBet,
    /// T11 (SQ-) Squeeze preflop vs an open and one or more callers.
    SqueezePlay,
    /// T12 (BD-) Big Blind defense: 3-bet, call, or fold facing a single raise.
    BigBlindDefense,
    /// T13 (3B-) C-bet sizing in 3-bet pots (smaller SPR, different dynamics).
    ThreeBetPotCbet,
    /// T14 (RF-) Facing a river bet: call, fold, or raise.
    RiverCallOrFold,
    /// T15 (PB-) Turn probe bet OOP after the flop checks through.
    TurnProbeBet,
    /// T16 (DC-) Delayed c-bet on the turn after checking back the flop IP.
    DelayedCbet,
}

impl TrainingTopic {
    /// Which street this topic belongs to.
    pub fn street(self) -> Street {
        match self {
            TrainingTopic::PreflopDecision
            | TrainingTopic::ICMAndTournamentDecision
            | TrainingTopic::AntiLimperIsolation
            | TrainingTopic::SqueezePlay
            | TrainingTopic::BigBlindDefense => Street::Preflop,

            TrainingTopic::PostflopContinuationBet
            | TrainingTopic::PotOddsAndEquity
            | TrainingTopic::CheckRaiseSpot
            | TrainingTopic::SemiBluffDecision
            | TrainingTopic::ThreeBetPotCbet => Street::Flop,

            TrainingTopic::TurnBarrelDecision
            | TrainingTopic::TurnProbeBet
            | TrainingTopic::DelayedCbet => Street::Turn,

            TrainingTopic::BluffSpot
            | TrainingTopic::RiverValueBet
            | TrainingTopic::RiverCallOrFold => Street::River,
        }
    }
}

impl fmt::Display for TrainingTopic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TrainingTopic::PreflopDecision           => "Preflop Decision",
            TrainingTopic::PostflopContinuationBet   => "Postflop Continuation Bet",
            TrainingTopic::PotOddsAndEquity          => "Pot Odds & Equity",
            TrainingTopic::BluffSpot                 => "Bluff Spot",
            TrainingTopic::ICMAndTournamentDecision  => "ICM & Tournament Decision",
            TrainingTopic::TurnBarrelDecision        => "Turn Barrel Decision",
            TrainingTopic::CheckRaiseSpot            => "Check-Raise Spot",
            TrainingTopic::SemiBluffDecision         => "Semi-Bluff Decision",
            TrainingTopic::AntiLimperIsolation       => "Anti-Limper Isolation",
            TrainingTopic::RiverValueBet             => "River Value Bet",
            TrainingTopic::SqueezePlay               => "Squeeze Play",
            TrainingTopic::BigBlindDefense           => "Big Blind Defense",
            TrainingTopic::ThreeBetPotCbet           => "3-Bet Pot C-Bet",
            TrainingTopic::RiverCallOrFold           => "River Call or Fold",
            TrainingTopic::TurnProbeBet              => "Turn Probe Bet",
            TrainingTopic::DelayedCbet               => "Delayed C-Bet",
        };
        write!(f, "{}", s)
    }
}

/// Controls stack-depth ranges and bet-size variance.
///
/// `Beginner` is the default — fixed stacks, narrow bet sizes, predictable
/// scenarios for new players.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DifficultyLevel {
    #[default]
    Beginner,
    Intermediate,
    Advanced,
}

impl fmt::Display for DifficultyLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DifficultyLevel::Beginner     => write!(f, "Beginner"),
            DifficultyLevel::Intermediate => write!(f, "Intermediate"),
            DifficultyLevel::Advanced     => write!(f, "Advanced"),
        }
    }
}

/// Controls the language style of question and explanation text.
///
/// `Simple` (the default) uses plain English with no poker jargon — suitable
/// for new players.  `Technical` uses standard poker terminology (SPR, EV,
/// fold equity, c-bet, etc.) aimed at more experienced players.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TextStyle {
    /// Plain English, no jargon.  This is the default.
    #[default]
    Simple,
    /// Standard poker terminology — SPR, EV, fold equity, c-bet, etc.
    Technical,
}

/// Choose what to drill: a specific topic or a random topic from a street.
///
/// ```ignore
/// // Specific topic:
/// TopicSelector::Topic(TrainingTopic::BluffSpot)
///
/// // Random flop topic (engine picks one from the 5 flop topics):
/// TopicSelector::Street(Street::Flop)
/// ```
///
/// Implements `From<TrainingTopic>` and `From<Street>` so you can use `.into()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TopicSelector {
    /// A specific training topic.
    Topic(TrainingTopic),
    /// Any random topic from this street (chosen by the RNG).
    Street(Street),
}

impl From<TrainingTopic> for TopicSelector {
    fn from(t: TrainingTopic) -> Self { TopicSelector::Topic(t) }
}

impl From<Street> for TopicSelector {
    fn from(s: Street) -> Self { TopicSelector::Street(s) }
}

/// Input to [`generate_training`](super::generate_training).
///
/// Only `topic` is truly required.  Everything else has a sensible default:
/// - `difficulty` → `Beginner`
/// - `rng_seed` → `None` (entropy)
/// - `text_style` → `Simple` (plain English)
///
/// ## Minimal usage
///
/// ```ignore
/// // Just a topic:
/// TrainingRequest::new(TrainingTopic::BluffSpot)
///
/// // Just a street:
/// TrainingRequest::new(Street::Flop)
/// ```
///
/// ## Full control
///
/// ```ignore
/// TrainingRequest {
///     topic: Street::Turn.into(),
///     difficulty: DifficultyLevel::Advanced,
///     rng_seed: Some(42),
///     text_style: TextStyle::Technical,
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingRequest {
    /// What to drill — a specific topic or any topic from a street.
    pub topic: TopicSelector,
    /// Controls stack-depth ranges and bet-size variance.
    /// Defaults to `Beginner`.
    #[serde(default)]
    pub difficulty: DifficultyLevel,
    /// `Some(seed)` for reproducible output; `None` (default) for entropy.
    #[serde(default)]
    pub rng_seed: Option<u64>,
    /// Language style for question and explanation text.
    /// Defaults to `Simple` (plain English).
    #[serde(default)]
    pub text_style: TextStyle,
}

impl TrainingRequest {
    /// Create a request with just a topic (or street).  All other fields
    /// use defaults: Beginner difficulty, entropy seed, Simple text.
    pub fn new(topic: impl Into<TopicSelector>) -> Self {
        Self {
            topic: topic.into(),
            difficulty: DifficultyLevel::default(),
            rng_seed: None,
            text_style: TextStyle::default(),
        }
    }
}

/// The physical table state: cards, positions, stacks, and pot.
///
/// `board` length depends on the street: 0 (preflop), 3 (flop), 4 (turn), 5 (river).
/// `current_bet` is 0 when hero is first to act; non-zero when facing a villain bet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSetup {
    pub game_type: GameType,
    pub hero_position: Position,
    /// Hero's two hole cards — never appear on the board.
    pub hero_hand: [Card; 2],
    /// Community cards dealt so far (always unique, disjoint from hero_hand).
    pub board: Vec<Card>,
    pub players: Vec<PlayerState>,
    pub pot_size: u32,
    /// The bet hero must call (0 = hero acts first / no bet to face).
    pub current_bet: u32,
}

/// One answer choice. Exactly one per scenario has `is_correct: true`.
///
/// `explanation` is a dynamically generated string (not a static template) that
/// explains *why* this option is correct or incorrect, adapted to the dealt cards
/// and the active `TextStyle`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerOption {
    /// Short ID shown in the UI (e.g. "A", "B", "C").
    pub id: String,
    /// Human-readable label (e.g. "Raise to 6 BB", "Fold").
    pub text: String,
    /// True for exactly one answer per scenario.
    pub is_correct: bool,
    /// Why this choice is right or wrong — changes with cards and TextStyle.
    pub explanation: String,
}

/// The complete output of [`generate_training`](super::generate_training).
///
/// Contains everything a UI needs: the table state, a question, and all
/// answer options (exactly one correct). The `scenario_id` is unique per
/// generation and the `branch_key` is stable across seeds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingScenario {
    /// Unique ID with a 2-letter topic prefix, e.g. `"PF-3A1C8F02"`.
    pub scenario_id: String,
    pub topic: TrainingTopic,
    /// Logical decision branch — stable across seeds.
    ///
    /// Use this for per-branch progress tracking. Examples:
    /// `"OpenRaise:premium:IP"`, `"Dry:RangeAdv"`, `"FlushDraw:Call"`.
    pub branch_key: String,
    pub table_setup: TableSetup,
    /// The question posed to the player (adapted to TextStyle).
    pub question: String,
    /// All answer choices — exactly one has `is_correct: true`.
    pub answers: Vec<AnswerOption>,
}
