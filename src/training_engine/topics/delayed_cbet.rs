use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    evaluator::board_texture,
    models::{
        AnswerOption, Card, DifficultyLevel, GameType, PlayerState,
        Position, TableSetup, TextStyle, TrainingScenario, TrainingTopic,
    },
};

// ---------------------------------------------------------------------------
// Hand strength classification against the board
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TurnStrength {
    Strong, // Overpair, top pair good kicker, two pair, set
    Medium, // Middle pair, weak top pair, decent underpair
    Weak,   // Missed, low pair, air
}

impl std::fmt::Display for TurnStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TurnStrength::Strong => write!(f, "strong (overpair / top pair good kicker / two pair / set)"),
            TurnStrength::Medium => write!(f, "medium (middle pair / weak top pair / underpair)"),
            TurnStrength::Weak   => write!(f, "weak (missed / low pair / air)"),
        }
    }
}

fn strength_simple(s: TurnStrength) -> &'static str {
    match s {
        TurnStrength::Strong => "strong hand",
        TurnStrength::Medium => "medium hand",
        TurnStrength::Weak   => "weak hand",
    }
}

/// Classify hero's hand strength against a 4-card board (flop + turn).
///
/// Classification order (first match wins):
///   Strong — set, two pair, overpair, top pair + good kicker (J+)
///   Medium — weak top pair, middle/bottom pair, any underpair
///   Weak   — no pair (air)
pub(crate) fn classify_turn_strength(hero: [Card; 2], board: &[Card]) -> TurnStrength {
    let h0 = hero[0].rank.0;
    let h1 = hero[1].rank.0;
    let high = h0.max(h1);

    let board_ranks: Vec<u8> = board.iter().map(|c| c.rank.0).collect();
    let board_max = board_ranks.iter().copied().max().unwrap_or(0);

    let pocket_pair = h0 == h1;
    let matches_h0 = board_ranks.iter().filter(|&&r| r == h0).count();
    let matches_h1 = board_ranks.iter().filter(|&&r| r == h1).count();

    if pocket_pair {
        // Set: pocket pair + at least one matching board card
        if matches_h0 >= 1 {
            return TurnStrength::Strong;
        }
        // Overpair: pair above all board cards
        if high > board_max {
            return TurnStrength::Strong;
        }
        // Any other pocket pair (underpair / middle pair)
        return TurnStrength::Medium;
    }

    // Non-pair hands — check how many hero cards paired the board
    let paired_both = matches_h0 >= 1 && matches_h1 >= 1;
    let paired_any  = matches_h0 >= 1 || matches_h1 >= 1;

    // Two pair: both hero cards found on the board
    if paired_both {
        return TurnStrength::Strong;
    }

    // One card paired — which one?
    if paired_any {
        let paired_rank = if matches_h0 >= 1 { h0 } else { h1 };
        if paired_rank == board_max {
            // Top pair — kicker determines strength
            let kicker = if paired_rank == h0 { h1 } else { h0 };
            if kicker >= 11 {
                return TurnStrength::Strong; // top pair + good kicker
            }
            return TurnStrength::Medium; // top pair + weak kicker
        }
        // Middle or bottom pair
        return TurnStrength::Medium;
    }

    // Nothing paired — air
    TurnStrength::Weak
}

// ---------------------------------------------------------------------------
// Turn card classification
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TurnCard {
    Blank,
    Scare,
}

/// Classify the turn card relative to the flop.
/// A scare card is an overcard to the flop or completes a flush/straight draw.
pub(crate) fn classify_turn_card(flop: &[Card], turn: &Card) -> TurnCard {
    let flop_max = flop.iter().map(|c| c.rank.0).max().unwrap_or(0);

    // Overcard to flop
    if turn.rank.0 > flop_max {
        return TurnCard::Scare;
    }

    // Third card of same suit = flush completing card
    let turn_suit_count = flop.iter().filter(|c| c.suit == turn.suit).count();
    if turn_suit_count >= 2 {
        return TurnCard::Scare;
    }

    // Check if turn creates a 4-straight on board
    let mut all_ranks: Vec<u8> = flop.iter().map(|c| c.rank.0).collect();
    all_ranks.push(turn.rank.0);
    all_ranks.sort_unstable();
    all_ranks.dedup();
    if all_ranks.len() >= 4 {
        for w in all_ranks.windows(4) {
            if w[3] - w[0] <= 4 {
                return TurnCard::Scare;
            }
        }
    }

    TurnCard::Blank
}

// ---------------------------------------------------------------------------
// Scenario generator
// ---------------------------------------------------------------------------

pub fn generate<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let mut deck = Deck::new_shuffled(rng);
    let hero_hand: [Card; 2] = [deck.deal(), deck.deal()];
    let board: Vec<Card> = deck.deal_n(4);

    let flop = &board[..3];
    let turn = &board[3];

    let strength = classify_turn_strength(hero_hand, &board);
    let turn_type = classify_turn_card(flop, turn);

    let bb = 2u32;
    let (pot_bb, stack_bb) = match difficulty {
        DifficultyLevel::Beginner     => (rng.gen_range(6..=14u32), 80u32),
        DifficultyLevel::Intermediate => (rng.gen_range(4..=20),    rng.gen_range(40..=100)),
        DifficultyLevel::Advanced     => (rng.gen_range(4..=30),    rng.gen_range(20..=150)),
    };
    let pot   = pot_bb * bb;
    let stack = stack_bb * bb;

    let small_cbet = (pot as f32 * 0.33).round() as u32;
    let medium_cbet = (pot as f32 * 0.60).round() as u32;

    // Decision matrix:
    // Strong + Any          → C (medium delayed c-bet ~60%)
    // Medium + Blank        → B (small delayed c-bet ~33%)
    // Medium + Scare        → A (check — pot control)
    // Weak + Any            → A (check — save chips)
    let correct: &str = match (strength, turn_type) {
        (TurnStrength::Strong, _)                   => "C",
        (TurnStrength::Medium, TurnCard::Blank)     => "B",
        (TurnStrength::Medium, TurnCard::Scare)     => "A",
        (TurnStrength::Weak, _)                     => "A",
    };

    let turn_label = match turn_type {
        TurnCard::Blank => "Blank",
        TurnCard::Scare => "Scare",
    };
    let strength_label = match strength {
        TurnStrength::Strong => "Strong",
        TurnStrength::Medium => "Medium",
        TurnStrength::Weak   => "Weak",
    };
    let branch_key = format!("{strength_label}:{turn_label}");

    let hero_pos  = Position::BTN;
    let hand_str  = format!("{}{}", hero_hand[0], hero_hand[1]);
    let board_str = board.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
    let texture   = board_texture(&board);

    let strength_simple = strength_simple(strength);

    let question = match text_style {
        TextStyle::Simple => format!(
            "You raised before the flop from the Button. The Big Blind called. \
             On the flop you checked behind. Now on the turn the board is: {board_str}. \
             You have {hand_str} ({strength_simple}). \
             Pot: {pot} chips. Stack: {stack} chips. \
             Villain checks to you. What do you do?"
        ),
        TextStyle::Technical => format!(
            "Delayed c-bet spot. You opened BTN, BB called. You checked back the flop \
             (no c-bet). Board: {board_str} ({texture}). \
             You hold {hand_str} ({strength}). \
             Turn card is a {turn_label}. \
             Pot: {pot} chips ({pot_bb} BB). Stack: {stack} chips. \
             Villain checks. Delayed c-bet options: small ({small_cbet} ~33%), \
             medium ({medium_cbet} ~60%), or check. What is your play?"
        ),
    };

    let answers = vec![
        AnswerOption {
            id: "A".to_string(),
            text: "Check".to_string(),
            is_correct: correct == "A",
            explanation: match text_style {
                TextStyle::Simple => match (strength, turn_type) {
                    (TurnStrength::Weak, _) => format!(
                        "Correct — check. Your hand missed the board. Betting here with nothing \
                         risks chips for no reason. Check and see a free river."
                    ),
                    (TurnStrength::Medium, TurnCard::Scare) => format!(
                        "Correct — check for pot control. The turn card changed the board and \
                         your medium hand may no longer be best. Keep the pot small."
                    ),
                    (TurnStrength::Medium, TurnCard::Blank) => format!(
                        "Checking here is too passive. You have a decent hand on a quiet turn card — \
                         a small bet would get value and protect against draws."
                    ),
                    (TurnStrength::Strong, _) => format!(
                        "Checking here wastes your strong hand. You skipped the flop — \
                         now is the time to bet and build the pot."
                    ),
                },
                TextStyle::Technical => match (strength, turn_type) {
                    (TurnStrength::Weak, _) => format!(
                        "Correct. With a {strength} you have no equity to justify a delayed c-bet. \
                         Villain's flop check/check line doesn't cap their range enough to bluff \
                         profitably. Check and realise any equity you have."
                    ),
                    (TurnStrength::Medium, TurnCard::Scare) => format!(
                        "Correct. The scare turn card (overcard / draw completion) devalues your \
                         {strength}. A delayed c-bet here bloats the pot when villain's continuing \
                         range has improved. Check for pot control and reassess on the river."
                    ),
                    (TurnStrength::Medium, TurnCard::Blank) => format!(
                        "Checking a {strength} on a blank turn is too passive. The blank didn't \
                         change the board — a small delayed c-bet (~33%) extracts thin value \
                         and denies equity to overcards."
                    ),
                    (TurnStrength::Strong, _) => format!(
                        "Checking a {strength} on the turn after already checking the flop forfeits \
                         too much value. You must fire a delayed c-bet to build the pot — villain's \
                         range includes many hands that will call a medium sizing."
                    ),
                },
            },
        },
        AnswerOption {
            id: "B".to_string(),
            text: format!("Small delayed c-bet ({small_cbet} chips ~33%)"),
            is_correct: correct == "B",
            explanation: match text_style {
                TextStyle::Simple => match (strength, turn_type) {
                    (TurnStrength::Medium, TurnCard::Blank) => format!(
                        "Correct — bet small. You have a decent hand on a quiet board. A small bet \
                         gets value from worse hands and makes draws pay a bit, without risking too much."
                    ),
                    (TurnStrength::Strong, _) => format!(
                        "A small bet is too timid with a strong hand. Bet bigger to build the pot \
                         and charge draws properly."
                    ),
                    _ => format!(
                        "A small bet here doesn't accomplish much. With your hand, either check back \
                         or bet bigger."
                    ),
                },
                TextStyle::Technical => match (strength, turn_type) {
                    (TurnStrength::Medium, TurnCard::Blank) => format!(
                        "Correct. A small delayed c-bet (~33% pot) with a {strength} on a blank \
                         turn is optimal. You extract thin value, deny equity to overcards and \
                         gutshots, and keep the pot manageable if raised."
                    ),
                    (TurnStrength::Strong, _) => format!(
                        "A 33% sizing with a {strength} leaves too much value on the table. \
                         Villain's range includes one-pair and draw hands that will call ~60%. \
                         Size up to maximise EV."
                    ),
                    (TurnStrength::Medium, TurnCard::Scare) => format!(
                        "Betting small on a scare turn with a {strength} risks getting raised off \
                         the best hand. The scare card improves villain's continuing range — \
                         pot control via check is preferred."
                    ),
                    (TurnStrength::Weak, _) => format!(
                        "A small delayed c-bet with a {strength} is a bluff with poor equity. \
                         Villain's calling range on this runout beats you. Save chips and check."
                    ),
                },
            },
        },
        AnswerOption {
            id: "C".to_string(),
            text: format!("Medium delayed c-bet ({medium_cbet} chips ~60%)"),
            is_correct: correct == "C",
            explanation: match text_style {
                TextStyle::Simple => match strength {
                    TurnStrength::Strong => format!(
                        "Correct — bet medium! You have a strong hand and you already checked \
                         the flop. Time to get value. A ~60% pot bet puts pressure on weaker \
                         hands and makes draws pay."
                    ),
                    TurnStrength::Medium => format!(
                        "A medium bet is too big for your hand strength. You risk too many chips \
                         when you might not have the best hand."
                    ),
                    TurnStrength::Weak => format!(
                        "Betting big with a weak hand is reckless. Check and save your chips."
                    ),
                },
                TextStyle::Technical => match strength {
                    TurnStrength::Strong => format!(
                        "Correct. A medium delayed c-bet (~60% pot) with a {strength} is highest-EV. \
                         After checking the flop your range is perceived as weak, so villain will \
                         call more liberally. This sizing extracts max value from one-pair and draw \
                         hands while setting up a river shove."
                    ),
                    TurnStrength::Medium => format!(
                        "A 60% pot delayed c-bet over-commits a {strength}. If raised, you're in \
                         a tough spot with a marginal hand. A smaller sizing (~33%) achieves the \
                         same protection at lower risk, or check on a scare card."
                    ),
                    TurnStrength::Weak => format!(
                        "A large delayed c-bet with a {strength} is a high-risk bluff. \
                         Villain's range after calling preflop and seeing a check-through \
                         includes many sticky hands. Check to preserve your stack."
                    ),
                },
            },
        },
    ];

    let players = vec![
        PlayerState { seat: 1, position: Position::BB,  stack, is_hero: false, is_active: true },
        PlayerState { seat: 2, position: hero_pos,       stack, is_hero: true,  is_active: true },
    ];

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::DelayedCbet,
        branch_key,
        table_setup: TableSetup {
            game_type:     GameType::CashGame,
            hero_position: hero_pos,
            hero_hand,
            board,
            players,
            pot_size:      pot,
            current_bet:   0,
        },
        question,
        answers,
    }
}
