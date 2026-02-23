use std::fmt;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Card primitives
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrainingTopic {
    PreflopDecision,
    PostflopContinuationBet,
    PotOddsAndEquity,
    BluffSpot,
    ICMAndTournamentDecision,
    TurnBarrelDecision,
    CheckRaiseSpot,
    SemiBluffDecision,
    AntiLimperIsolation,
    RiverValueBet,
    SqueezePlay,
    BigBlindDefense,
    ThreeBetPotCbet,
    RiverCallOrFold,
    TurnProbeBet,
    MultiwayPot,
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
            TrainingTopic::MultiwayPot               => "Multiway Pot",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DifficultyLevel {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingRequest {
    pub topic: TrainingTopic,
    pub difficulty: DifficultyLevel,
    pub rng_seed: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSetup {
    pub game_type: GameType,
    pub hero_position: Position,
    pub hero_hand: [Card; 2],
    pub board: Vec<Card>,
    pub players: Vec<PlayerState>,
    pub pot_size: u32,
    pub current_bet: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerOption {
    pub id: String,
    pub text: String,
    pub is_correct: bool,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingScenario {
    pub scenario_id: String,
    pub topic: TrainingTopic,
    /// Identifies the logical decision branch within this topic.
    /// Stable across seeds — use for per-branch progress tracking.
    /// Examples: "OpenRaise:premium:IP", "BBFav:ComboDraw", "OESD:Deep"
    pub branch_key: String,
    pub table_setup: TableSetup,
    pub question: String,
    pub answers: Vec<AnswerOption>,
}
