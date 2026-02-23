use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    models::{
        AnswerOption, Card, DifficultyLevel, GameType, PlayerState,
        Position, TableSetup, TrainingScenario, TrainingTopic,
    },
};

// ---------------------------------------------------------------------------
// Hand classification (inline copy of preflop's 5-category logic)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HandCategory {
    Premium,
    Strong,
    Playable,
    Marginal,
    Trash,
}

impl std::fmt::Display for HandCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            HandCategory::Premium  => "premium",
            HandCategory::Strong   => "strong",
            HandCategory::Playable => "playable",
            HandCategory::Marginal => "marginal",
            HandCategory::Trash    => "trash",
        };
        write!(f, "{}", s)
    }
}

fn classify_hand(hand: [Card; 2]) -> HandCategory {
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
        (14, 13, true)              => HandCategory::Premium,
        (14, 13, false)             => HandCategory::Strong,
        (14, 12, _)                 => HandCategory::Strong,
        (14, 11, true)              => HandCategory::Playable,
        (14, r, true) if r >= 9     => HandCategory::Playable,
        (13, 12, true)              => HandCategory::Playable,
        (13, 12, false)             => HandCategory::Marginal,
        (r1, r2, true) if r1 >= 9 && r1 - r2 <= 1 => HandCategory::Playable,
        (r1, _, _) if r1 <= 9      => HandCategory::Trash,
        _                           => HandCategory::Marginal,
    }
}

// ---------------------------------------------------------------------------
// Iso-raise sizing based on limper count
// ---------------------------------------------------------------------------

fn iso_raise_bb(limper_count: u8) -> u32 {
    match limper_count {
        1 => 4,
        2 => 5,
        _ => 6, // 3+ limpers
    }
}

// ---------------------------------------------------------------------------
// Position helpers
// ---------------------------------------------------------------------------

fn is_in_position(pos: Position) -> bool {
    matches!(pos, Position::CO | Position::BTN)
}

fn position_label(pos: Position) -> &'static str {
    match pos {
        Position::CO  => "Cutoff",
        Position::BTN => "Button",
        Position::SB  => "Small Blind",
        _             => "Unknown",
    }
}

// ---------------------------------------------------------------------------
// Main generator
// ---------------------------------------------------------------------------

pub fn generate<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
) -> TrainingScenario {
    let mut deck = Deck::new_shuffled(rng);
    let hero_hand: [Card; 2] = [deck.deal(), deck.deal()];

    let hero_pos = match rng.gen_range(0..3) {
        0 => Position::CO,
        1 => Position::BTN,
        _ => Position::SB,
    };

    let limper_count: u8 = rng.gen_range(1..=3);
    let ip = is_in_position(hero_pos);

    let bb = 2u32;
    let stack_bb: u32 = match difficulty {
        DifficultyLevel::Beginner     => rng.gen_range(60..=120),
        DifficultyLevel::Intermediate => rng.gen_range(30..=150),
        DifficultyLevel::Advanced     => rng.gen_range(15..=200),
    };
    let stack = stack_bb * bb;
    let pot = bb + (bb / 2) + (bb * limper_count as u32); // BB + SB + limpers

    let cat = classify_hand(hero_hand);
    let iso_bb = iso_raise_bb(limper_count);
    let iso_chips = iso_bb * bb;

    let hand_str = format!("{}{}", hero_hand[0], hero_hand[1]);
    let pos_str = position_label(hero_pos);
    let limper_word = if limper_count == 1 { "limper" } else { "limpers" };
    let pos_qualifier = if ip { "in position" } else { "out of position" };

    // Correct answer (single ID):
    // Premium/Strong                   → "C" (Iso-raise always)
    // Playable + IP (CO/BTN)           → "C" (Iso-raise with position)
    // Playable + SB (OOP)              → "B" (Overlimp)
    // Marginal/Trash                   → "A" (Fold)
    let correct: &str = match cat {
        HandCategory::Premium | HandCategory::Strong         => "C",
        HandCategory::Playable if ip                         => "C",
        HandCategory::Playable                               => "B",
        HandCategory::Marginal | HandCategory::Trash         => "A",
    };

    let branch_key = match (cat, ip) {
        (HandCategory::Premium, _)      => "Premium".to_string(),
        (HandCategory::Strong, _)       => "Strong".to_string(),
        (HandCategory::Playable, true)  => "Playable:IP".to_string(),
        (HandCategory::Playable, false) => "Playable:OOP".to_string(),
        (HandCategory::Marginal, _)     => "Marginal".to_string(),
        (HandCategory::Trash, _)        => "Trash".to_string(),
    };

    let question = format!(
        "You hold {hand_str} ({cat}) on the {pos_str} ({pos_qualifier}, {stack_bb} BB deep). \
         {limper_count} player(s) limp in front of you. Pot is {pot} chips. \
         What is your action?"
    );

    // --- Explanations ---

    let fold_exp = if matches!(cat, HandCategory::Marginal | HandCategory::Trash) {
        format!(
            "Correct. A {cat} hand from {pos_str} against {limper_count} {limper_word} is a \
             clear fold. Iso-raising with {hand_str} builds a large pot without sufficient \
             equity against even limping ranges. Overlimping is even worse — it invites more \
             players and removes any initiative. Fold and wait for a stronger hand."
        )
    } else {
        format!(
            "Folding {hand_str} ({cat}) from {pos_str} is too tight. You have enough hand \
             strength and/or positional advantage to profitably enter the pot here. \
             Limpers have shown weakness — exploit it."
        )
    };

    let overlimp_exp = match (cat, ip) {
        (HandCategory::Playable, false) => format!(
            "Correct. Overlimping with {hand_str} ({cat}) from the Small Blind is the best \
             play. Iso-raising to {iso_chips} chips would build a large pot that you'll play \
             from the worst position at the table (OOP every street). Instead, calling 1 BB \
             lets you see a cheap flop with a speculative hand and realise implied odds \
             without committing too many chips. Note: iso-raise from CO or BTN with this hand."
        ),
        _ => format!(
            "Overlimping with {hand_str} ({cat}) from {pos_str} is too passive. \
             {}",
            if ip {
                format!(
                    "You have positional advantage (IP) — iso-raising to {iso_chips} chips \
                     ({iso_bb} BB) is higher EV. It denies limpers' cheap flops, wins \
                     dead money outright sometimes, and sets up a profitable postflop spot \
                     in position."
                )
            } else {
                format!(
                    "This hand is too strong to just call — iso-raise to {iso_chips} chips \
                     ({iso_bb} BB) to punish the limpers and build the pot with initiative."
                )
            }
        ),
    };

    let iso_exp = match (cat, ip) {
        (HandCategory::Premium | HandCategory::Strong, _) => format!(
            "Correct. Iso-raising to {iso_chips} chips ({iso_bb} BB) with {hand_str} ({cat}) \
             is mandatory from {pos_str}. You never let limpers see a cheap flop with a \
             premium or strong hand. The raise: (1) defines your hand as strong, \
             (2) builds a pot with an equity advantage, (3) often wins uncontested vs \
             {limper_count} {limper_word}. Size is {iso_bb} BB to account for {limper_count} \
             limper(s) already in the pot."
        ),
        (HandCategory::Playable, true) => format!(
            "Correct. Iso-raising to {iso_chips} chips ({iso_bb} BB) with {hand_str} ({cat}) \
             from {pos_str} (IP) is correct. Limpers are almost always weaker than a raiser's \
             range. With positional advantage postflop you can: c-bet profitably on a wide \
             range of boards, win with fold equity, and extract value when you connect. \
             {iso_bb} BB accounts for {limper_count} {limper_word} already limped."
        ),
        _ => format!(
            "Iso-raising to {iso_chips} chips with {hand_str} ({cat}) from {pos_str} \
             (OOP) builds too large a pot to play from the worst position at the table. \
             With a {cat} hand OOP, overlimping or folding is better than iso-raising."
        ),
    };

    let players = vec![
        // Simplified: show one limper + hero
        PlayerState {
            seat: 1,
            position: Position::UTG,
            stack,
            is_hero: false,
            is_active: true,
        },
        PlayerState {
            seat: 2,
            position: hero_pos,
            stack,
            is_hero: true,
            is_active: true,
        },
    ];

    let answers = vec![
        AnswerOption {
            id: "A".to_string(),
            text: "Fold".to_string(),
            is_correct: correct == "A",
            explanation: fold_exp,
        },
        AnswerOption {
            id: "B".to_string(),
            text: "Call".to_string(),
            is_correct: correct == "B",
            explanation: overlimp_exp,
        },
        AnswerOption {
            id: "C".to_string(),
            text: format!("Raise to {} BB", iso_bb),
            is_correct: correct == "C",
            explanation: iso_exp,
        },
    ];

    let table_setup = TableSetup {
        game_type: GameType::CashGame,
        hero_position: hero_pos,
        hero_hand,
        board: vec![],
        players,
        pot_size: pot,
        current_bet: bb, // the limp amount
    };

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::AntiLimperIsolation,
        branch_key,
        table_setup,
        question,
        answers,
    }
}
