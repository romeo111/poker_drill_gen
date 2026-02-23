use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    models::{
        AnswerOption, Card, DifficultyLevel, GameType, PlayerState,
        Position, TableSetup, TextStyle, TrainingScenario, TrainingTopic,
    },
};

/// Hero's holding category when defending the Big Blind against a single raise.
#[derive(Debug, Clone, Copy)]
enum DefenseStrength {
    Strong,   // JJ+, AK, AQs — 3-bet for value from the BB
    Playable, // 22–TT, suited connectors, broadways — call with pot-odds discount
    Weak,     // Off-suit non-broadway trash — fold even from the BB
}

impl std::fmt::Display for DefenseStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefenseStrength::Strong   => write!(f, "strong (JJ+/AK/AQs)"),
            DefenseStrength::Playable => write!(f, "playable (mid pair / suited connector / broadway)"),
            DefenseStrength::Weak     => write!(f, "weak (off-suit trash)"),
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
        0 => DefenseStrength::Strong,
        1 => DefenseStrength::Playable,
        _ => DefenseStrength::Weak,
    };

    // Villain's position determines their opening range width
    let villain_pos = match rng.gen_range(0..3) {
        0 => Position::UTG,
        1 => Position::CO,
        _ => Position::BTN,
    };

    let bb = 2u32;
    let (raise_bb, stack_bb) = match difficulty {
        DifficultyLevel::Beginner     => (3u32, 100u32),
        DifficultyLevel::Intermediate => (rng.gen_range(2..=4), rng.gen_range(60..=120)),
        DifficultyLevel::Advanced     => (rng.gen_range(2..=5), rng.gen_range(25..=150)),
    };

    // Pot before hero acts: raise + 1 BB (hero's dead big blind; SB folds)
    let pot_bb    = raise_bb + 1;
    let pot       = pot_bb * bb;
    let stack     = stack_bb * bb;

    // Standard BB 3-bet sizing: ~3× raise + 1 dead BB re-invested
    let three_bet_bb = raise_bb * 3 + 1;
    let three_bet    = three_bet_bb * bb;

    // Correct action:
    // Strong   → 3-bet (value 3-bet, build pot with equity advantage)
    // Playable → Call  (BB discount makes defence profitable; good implied odds)
    // Weak     → Fold  (even the BB discount can't save off-suit trash)
    let correct: &str = match strength {
        DefenseStrength::Strong   => "C",
        DefenseStrength::Playable => "B",
        DefenseStrength::Weak     => "A",
    };

    let branch_key = match strength {
        DefenseStrength::Strong   => "Strong:ThreeBet",
        DefenseStrength::Playable => "Playable:Call",
        DefenseStrength::Weak     => "Weak:Fold",
    }.to_string();

    let hero_pos = Position::BB;
    let hand_str = format!("{}{}", hero_hand[0], hero_hand[1]);

    let question = match text_style {
        TextStyle::Simple => format!(
            "Before the flop. You have {hand_str} in the Big Blind. \
             {villain_pos} raised to {raise_bb} big blinds. Everyone else folded. \
             Pot: {pot} chips. Stack: {stack} chips. \
             A re-raise would be ~{three_bet_bb} big blinds. What do you do?"
        ),
        TextStyle::Technical => format!(
            "Big Blind defense. You hold {hand_str} ({strength}) in the Big Blind. \
             {villain_pos} raises to {raise_bb} BB. Action folds to you. \
             Pot: {pot} chips ({pot_bb} BB). Stack: {stack} chips. \
             A 3-bet would be ~{three_bet} chips ({three_bet_bb} BB). \
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
                    DefenseStrength::Weak => format!(
                        "Correct — fold. Even with your money already in, {hand_str} isn't strong enough to continue against this raise."
                    ),
                    _ => format!(
                        "Folding here throws away money you already put in. You have a playable hand — call at minimum."
                    ),
                },
                TextStyle::Technical => match strength {
                    DefenseStrength::Weak => format!(
                        "Correct. Folding a {strength} from the BB is correct even with the \
                         pot-odds discount. Off-suit non-broadway hands have poor equity and \
                         will routinely flop dominated pairs, no draws, and difficult second-best \
                         spots. Save the chips for a better hand."
                    ),
                    _ => format!(
                        "Folding a {strength} from the BB is too tight. You already have 1 BB \
                         invested and are getting a direct pot-odds discount. Strong hands should \
                         3-bet; playable hands should call. Only trash warrants a fold here."
                    ),
                },
            },
        },
        AnswerOption {
            id: "B".to_string(),
            text: format!("Call ({raise_bb} BB)"),
            is_correct: correct == "B",
            explanation: match text_style {
                TextStyle::Simple => match strength {
                    DefenseStrength::Playable => format!(
                        "Correct — call! You're already part-way in with the Big Blind and {hand_str} can see a flop at a discount. Don't fold this away."
                    ),
                    DefenseStrength::Strong => format!(
                        "Just calling is too passive — re-raise with this strong hand to build the pot."
                    ),
                    DefenseStrength::Weak => format!(
                        "Just calling is too passive — re-raise with this strong hand to build the pot."
                    ),
                },
                TextStyle::Technical => match strength {
                    DefenseStrength::Playable => format!(
                        "Correct. Calling with a {strength} from the BB is the best play. \
                         Your pot-odds discount and direct equity make defence profitable. \
                         Playable hands (pairs, suited connectors, broadways) have enough equity \
                         and implied odds to justify calling {raise_bb} BB and seeing a flop. \
                         Avoid 3-betting marginal hands OOP."
                    ),
                    DefenseStrength::Strong => format!(
                        "Calling with a {strength} from the BB misses a value opportunity. \
                         You have a large equity advantage over {villain_pos}'s range — a 3-bet \
                         builds the pot while you're ahead and may win the dead money outright. \
                         Calling allows villain to realise their equity cheaply."
                    ),
                    DefenseStrength::Weak => format!(
                        "Calling with a {strength} from the BB is still -EV. The pot-odds \
                         discount helps but doesn't overcome the fact that off-suit trash has \
                         minimal equity, poor flop-hit rate, and faces a strong opening range. \
                         Fold and wait."
                    ),
                },
            },
        },
        AnswerOption {
            id: "C".to_string(),
            text: format!("3-bet to {three_bet} chips ({three_bet_bb} BB)"),
            is_correct: correct == "C",
            explanation: match text_style {
                TextStyle::Simple => match strength {
                    DefenseStrength::Strong => format!(
                        "Correct — re-raise to {three_bet_bb} big blinds! {hand_str} is a strong hand. Make your opponent pay to continue and take control of the pot."
                    ),
                    _ => format!(
                        "Re-raising here is too aggressive with {hand_str}. Just call and see the flop."
                    ),
                },
                TextStyle::Technical => match strength {
                    DefenseStrength::Strong => format!(
                        "Correct. 3-betting to {three_bet_bb} BB with a {strength} from the BB \
                         is the highest-EV play. You have a significant equity advantage over \
                         {villain_pos}'s opening range. A 3-bet builds the pot while you're ahead, \
                         denies equity to dominated hands, and forces a tough decision. Against a \
                         wide opener (CO/BTN) this is even more profitable."
                    ),
                    DefenseStrength::Playable => format!(
                        "3-betting with a {strength} from the BB turns a +EV call into a \
                         marginal bluff-raise. Playable hands do not have enough raw equity to \
                         3-bet for value against most opening ranges, and bloating the pot OOP \
                         with a speculative hand is risky. Calling preserves implied odds."
                    ),
                    DefenseStrength::Weak => format!(
                        "3-betting with a {strength} from the BB is a bluff with no equity \
                         foundation. Even if it works occasionally, this play has poor \
                         risk-to-reward — villain can 4-bet or call with many better hands. \
                         Fold instead."
                    ),
                },
            },
        },
    ];

    let players = vec![
        PlayerState { seat: 1, position: villain_pos, stack, is_hero: false, is_active: true },
        PlayerState { seat: 2, position: hero_pos,    stack, is_hero: true,  is_active: true },
    ];

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::BigBlindDefense,
        branch_key,
        table_setup: TableSetup {
            game_type:     GameType::CashGame,
            hero_position: hero_pos,
            hero_hand,
            board:         vec![],
            players,
            pot_size:      pot,
            current_bet:   raise_bb * bb,
        },
        question,
        answers,
    }
}
