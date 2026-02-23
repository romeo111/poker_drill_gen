use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    models::{
        AnswerOption, Card, DifficultyLevel, GameType, PlayerState,
        Position, TableSetup, TextStyle, TrainingScenario, TrainingTopic,
    },
};

/// Strength of hero's holding in a preflop squeeze spot.
#[derive(Debug, Clone, Copy)]
enum HoleStrength {
    Premium,     // AA, KK, QQ, AKs — squeeze for maximum value
    Speculative, // Mid pairs 77–99, suited connectors — call for implied odds
    Weak,        // Off-suit rag, dominated holdings — fold
}

impl std::fmt::Display for HoleStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HoleStrength::Premium     => write!(f, "premium (AA/KK/QQ/AKs)"),
            HoleStrength::Speculative => write!(f, "speculative (mid pair / suited connector)"),
            HoleStrength::Weak        => write!(f, "weak (off-suit rag / dominated hand)"),
        }
    }
}

pub fn generate<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let mut deck = Deck::new_shuffled(rng);
    let hero_hand: [Card; 2] = [deck.deal(), deck.deal()];

    let strength = match rng.gen_range(0..3) {
        0 => HoleStrength::Premium,
        1 => HoleStrength::Speculative,
        _ => HoleStrength::Weak,
    };

    let callers: u32 = match difficulty {
        DifficultyLevel::Beginner     => 1,
        DifficultyLevel::Intermediate => rng.gen_range(1..=2),
        DifficultyLevel::Advanced     => rng.gen_range(1..=3),
    };

    let bb = 2u32;
    let (open_bb, stack_bb) = match difficulty {
        DifficultyLevel::Beginner     => (3u32, 100u32),
        DifficultyLevel::Intermediate => (rng.gen_range(2..=4), rng.gen_range(60..=120)),
        DifficultyLevel::Advanced     => (rng.gen_range(2..=5), rng.gen_range(25..=150)),
    };

    // Dead money before hero acts: open + callers × open + SB (1 BB simplified)
    let pot_bb = open_bb + callers * open_bb + 1;
    let pot    = pot_bb * bb;
    let stack  = stack_bb * bb;

    // Squeeze sizing: ~3× the open + 1 open per caller (isolation premium)
    let squeeze_bb = open_bb * 3 + callers * open_bb;
    let squeeze    = squeeze_bb * bb;

    // Correct action:
    // Premium     → Squeeze (dominate the field, build a pot you likely win)
    // Speculative → Call   (implied odds justify set/draw play with callers in)
    // Weak        → Fold   (dominated equity, no profitable path)
    let correct: &str = match strength {
        HoleStrength::Premium     => "C",
        HoleStrength::Speculative => "B",
        HoleStrength::Weak        => "A",
    };

    let branch_key = match strength {
        HoleStrength::Premium     => "Premium:Squeeze",
        HoleStrength::Speculative => "Speculative:Call",
        HoleStrength::Weak        => "Weak:Fold",
    }.to_string();

    let hero_pos   = Position::BTN;
    let opener_pos = Position::UTG;
    let hand_str   = format!("{}{}", hero_hand[0], hero_hand[1]);
    let caller_str = if callers == 1 {
        "1 caller".to_string()
    } else {
        format!("{callers} callers")
    };

    let question = match text_style {
        TextStyle::Simple => format!(
            "Before the flop. You have {hand_str} on the Button. \
             One player raised to {open_bb} big blinds and {caller_str} called. \
             Pot: {pot} chips. Stack: {stack} chips. \
             A big re-raise would be ~{squeeze_bb} big blinds. What do you do?"
        ),
        TextStyle::Technical => format!(
            "Preflop squeeze. You hold {hand_str} ({strength}) on the Button. \
             UTG opens to {open_bb} BB, {caller_str} in between. \
             Pot: {pot} chips ({pot_bb} BB). Stack: {stack} chips. \
             A squeeze would be ~{squeeze} chips ({squeeze_bb} BB). \
             What do you do?"
        ),
    };

    let answers = vec![
        AnswerOption {
            id: "A".to_string(),
            text: "Fold".to_string(),
            is_correct: correct == "A",
            explanation: match text_style {
                TextStyle::Simple => match strength {
                    HoleStrength::Weak => format!(
                        "Correct — fold. Your hand isn't strong enough to enter a large pot against multiple active players."
                    ),
                    _ => format!(
                        "Folding here is too cautious — you have a good enough hand to re-raise or call."
                    ),
                },
                TextStyle::Technical => match strength {
                    HoleStrength::Weak => format!(
                        "Correct. Folding a {strength} in this squeeze spot is the right play. \
                         Your hand has poor equity against the opener's range and the callers — \
                         even the BTN pot-odds discount doesn't compensate for dominated equity. \
                         Wait for a better spot."
                    ),
                    _ => format!(
                        "Folding a {strength} gives up significant equity. Premium hands profit \
                         most when the pot is large and opponents are dominated. Speculative \
                         hands need callers for implied odds. Only fold genuinely weak holdings."
                    ),
                },
            },
        },
        AnswerOption {
            id: "B".to_string(),
            text: format!("Call ({open_bb} BB)"),
            is_correct: correct == "B",
            explanation: match text_style {
                TextStyle::Simple => match strength {
                    HoleStrength::Speculative => format!(
                        "Correct — call. With a hand that plays well in big pots, you can call and try to hit a big hand on the flop."
                    ),
                    HoleStrength::Premium => format!(
                        "Just calling isn't the best play here — re-raise to take control and thin the field."
                    ),
                    HoleStrength::Weak => format!(
                        "Just calling isn't the best play here — re-raise to take control and thin the field."
                    ),
                },
                TextStyle::Technical => match strength {
                    HoleStrength::Speculative => format!(
                        "Correct. Calling with a {strength} is optimal. Multiple callers create \
                         a large pot and improve your implied odds for sets, straights, and \
                         flushes. Squeezing bloats the pot where you may be dominated. Calling \
                         keeps your range disguised and preserves the implied-odds edge."
                    ),
                    HoleStrength::Premium => format!(
                        "Calling with a {strength} leaves too much value on the table. You have \
                         a massive equity edge — squeezing forces dominated hands to pay a steep \
                         price, often winning the dead money outright or playing a large pot as \
                         a significant favourite."
                    ),
                    HoleStrength::Weak => format!(
                        "Calling with a {strength} is -EV. You have poor equity against both the \
                         opener and the callers with no strong implied-odds potential. Fold."
                    ),
                },
            },
        },
        AnswerOption {
            id: "C".to_string(),
            text: format!("Squeeze to {squeeze} chips ({squeeze_bb} BB)"),
            is_correct: correct == "C",
            explanation: match text_style {
                TextStyle::Simple => match strength {
                    HoleStrength::Premium => format!(
                        "Correct — re-raise big! With {hand_str} you have a great hand. A big re-raise will often win the pot right now, or leave you heads-up against one player with the best hand."
                    ),
                    HoleStrength::Speculative => format!(
                        "Re-raising here with {hand_str} isn't justified. Your hand isn't strong enough to play a huge pot."
                    ),
                    HoleStrength::Weak => format!(
                        "Re-raising here with {hand_str} isn't justified. Your hand isn't strong enough to play a huge pot."
                    ),
                },
                TextStyle::Technical => match strength {
                    HoleStrength::Premium => format!(
                        "Correct. Squeezing to {squeeze_bb} BB with a {strength} is the highest-EV \
                         play. Your hand has dominant equity over the field. A squeeze isolates, \
                         collects dead money when folds come, and builds a large pot played as a \
                         heavy favourite when called. Never limp or flat with premium hands in a \
                         squeeze spot."
                    ),
                    HoleStrength::Speculative => format!(
                        "Squeezing with a {strength} turns a good implied-odds hand into a \
                         commitment bluff. If called, you are out of position with a hand that \
                         needs to hit the board — facing a range that calls 3-bets and likely \
                         dominates you. Calling is higher EV."
                    ),
                    HoleStrength::Weak => format!(
                        "Squeezing with a {strength} is a low-equity bluff. The opener and \
                         callers have uncapped ranges — expect 4-bets and calls from better \
                         hands. This play has strongly negative expected value."
                    ),
                },
            },
        },
    ];

    let players = vec![
        PlayerState { seat: 1, position: opener_pos, stack, is_hero: false, is_active: true },
        PlayerState { seat: 2, position: hero_pos,   stack, is_hero: true,  is_active: true },
    ];

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::SqueezePlay,
        branch_key,
        table_setup: TableSetup {
            game_type:     GameType::CashGame,
            hero_position: hero_pos,
            hero_hand,
            board:         vec![],
            players,
            pot_size:      pot,
            current_bet:   open_bb * bb,
        },
        question,
        answers,
    }
}
