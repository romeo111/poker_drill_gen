use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    models::{
        AnswerOption, Card, DifficultyLevel, GameType, PlayerState,
        Position, TableSetup, TextStyle, TrainingScenario, TrainingTopic,
    },
};

// ---------------------------------------------------------------------------
// Hand strength classification helpers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HandCategory {
    Premium,   // AA, KK, QQ, AKs
    Strong,    // JJ, TT, AQs, AKo, AQo
    Playable,  // 99-77, AJs, KQs, suited connectors
    Marginal,  // 66-22, offsuit broadway, weak aces
    Trash,
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

    // Non-pair
    match (r1, r2, suited) {
        (14, 13, true)         => HandCategory::Premium,
        (14, 13, false)        => HandCategory::Strong,
        (14, 12, true)         => HandCategory::Strong,
        (14, 12, false)        => HandCategory::Strong,
        (14, 11, true)         => HandCategory::Playable,
        (14, r, true) if r >= 9 => HandCategory::Playable,
        (13, 12, true)         => HandCategory::Playable,
        (13, 12, false)        => HandCategory::Marginal,
        (r1, r2, true) if r1 >= 9 && r1 - r2 == 1 => HandCategory::Playable,
        (r1, r2, true) if r1 >= 9 && r1 - r2 == 0 => HandCategory::Playable,
        (r1, _, _) if r1 <= 9 => HandCategory::Trash,
        _                      => HandCategory::Marginal,
    }
}

fn hand_category_name(cat: HandCategory) -> &'static str {
    match cat {
        HandCategory::Premium  => "premium",
        HandCategory::Strong   => "strong",
        HandCategory::Playable => "playable",
        HandCategory::Marginal => "marginal",
        HandCategory::Trash    => "trash",
    }
}

// ---------------------------------------------------------------------------
// Scenario-type enumeration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum PreflopSpot {
    OpenRaise,   // Hero faces a limped pot or everyone folded
    FacingOpen,  // Hero faces a single raise
    ThreeBetPot, // Hero faces a 3-bet after having opened
}

fn select_spot<R: Rng>(rng: &mut R) -> PreflopSpot {
    match rng.gen_range(0..3) {
        0 => PreflopSpot::OpenRaise,
        1 => PreflopSpot::FacingOpen,
        _ => PreflopSpot::ThreeBetPot,
    }
}

// ---------------------------------------------------------------------------
// Position helpers
// ---------------------------------------------------------------------------

const POSITIONS_6MAX: &[Position] = &[
    Position::UTG, Position::HJ, Position::CO,
    Position::BTN, Position::SB, Position::BB,
];

const POSITIONS_9MAX: &[Position] = &[
    Position::UTG, Position::UTG1, Position::UTG2,
    Position::LJ, Position::HJ, Position::CO,
    Position::BTN, Position::SB, Position::BB,
];

fn random_position<R: Rng>(rng: &mut R, is_6max: bool) -> Position {
    let pool = if is_6max { POSITIONS_6MAX } else { POSITIONS_9MAX };
    pool[rng.gen_range(0..pool.len())]
}

fn stack_for_difficulty<R: Rng>(rng: &mut R, diff: DifficultyLevel) -> u32 {
    match diff {
        DifficultyLevel::Beginner     => rng.gen_range(80..=120),
        DifficultyLevel::Intermediate => rng.gen_range(40..=150),
        DifficultyLevel::Advanced     => rng.gen_range(15..=300),
    }
}

// ---------------------------------------------------------------------------
// Main generator
// ---------------------------------------------------------------------------

pub fn generate<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let is_6max = rng.gen_bool(0.5);
    let table_size = if is_6max { 6usize } else { 9 };
    let spot = select_spot(rng);
    let hero_pos = random_position(rng, is_6max);
    let effective_stack = stack_for_difficulty(rng, difficulty);

    let mut deck = Deck::new_shuffled(rng);
    let hero_cards: [Card; 2] = [deck.deal(), deck.deal()];

    let cat = classify_hand(hero_cards);
    let pos_type = if hero_pos.is_late() { "IP" } else { "OOP" };
    let branch_key = match spot {
        PreflopSpot::ThreeBetPot => format!("ThreeBetPot:{}", hand_category_name(cat)),
        PreflopSpot::OpenRaise   => format!("OpenRaise:{}:{}", hand_category_name(cat), pos_type),
        PreflopSpot::FacingOpen  => format!("FacingOpen:{}:{}", hand_category_name(cat), pos_type),
    };

    // Build player list (simplified)
    let positions = if is_6max { POSITIONS_6MAX } else { POSITIONS_9MAX };
    let players: Vec<PlayerState> = positions
        .iter()
        .enumerate()
        .map(|(i, &pos)| PlayerState {
            seat: i as u8 + 1,
            position: pos,
            stack: if pos == hero_pos {
                effective_stack
            } else {
                stack_for_difficulty(rng, difficulty)
            },
            is_hero: pos == hero_pos,
            is_active: true,
        })
        .collect();

    let bb = 2u32;
    let (pot_size, current_bet, question, answers) =
        build_spot(rng, spot, hero_pos, hero_cards, effective_stack, bb, difficulty, table_size, text_style);

    let table_setup = TableSetup {
        game_type: GameType::CashGame,
        hero_position: hero_pos,
        hero_hand: hero_cards,
        board: vec![],
        players,
        pot_size,
        current_bet,
    };

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::PreflopDecision,
        branch_key,
        table_setup,
        question,
        answers,
    }
}

// ---------------------------------------------------------------------------
// Spot builders
// ---------------------------------------------------------------------------

fn build_spot<R: Rng>(
    _rng: &mut R,
    spot: PreflopSpot,
    pos: Position,
    hand: [Card; 2],
    stack: u32,
    bb: u32,
    _difficulty: DifficultyLevel,
    table_size: usize,
    text_style: TextStyle,
) -> (u32, u32, String, Vec<AnswerOption>) {
    let cat = classify_hand(hand);
    let cat_name = hand_category_name(cat);
    let hand_str = format!("{}{}", hand[0], hand[1]);
    let pos_str = format!("{}", pos);
    let stack_bb = stack / bb;

    match spot {
        // ---- Hero acts first (open-raise spot) ----------------------------
        PreflopSpot::OpenRaise => {
            let pot = bb + bb / 2; // SB + BB already in
            let open_size = if stack_bb >= 40 { bb * 3 } else { bb * 2 };
            let q = match text_style {
                TextStyle::Simple => format!(
                    "You have {hand_str} in {pos_str} at a {table_size}-handed table. \
                     Stack: {stack_bb} big blinds. Everyone before you folded. What do you do?"
                ),
                TextStyle::Technical => format!(
                    "You hold {hand_str} in {pos_str} at a {table_size}-handed table. \
                     Effective stack is {stack_bb} BB. Action folds to you. What is your action?"
                ),
            };
            // Single correct answer: raise if hand is strong enough, fold otherwise.
            // Limping is never correct here.
            let should_raise = matches!(cat, HandCategory::Premium | HandCategory::Strong)
                || (pos.is_late() && matches!(cat, HandCategory::Playable | HandCategory::Marginal));
            let correct = if should_raise { "B" } else { "A" };

            let fold_body = if correct == "A" {
                format!("Correct. A {cat_name} hand from {pos_str} lacks the equity and \
                         playability to justify entering the pot against a full range.")
            } else {
                format!("Too tight. {hand_str} ({cat_name}) from {pos_str} with {stack_bb} BB \
                         has enough value to open-raise profitably.")
            };
            let raise_body = if correct == "B" {
                format!("Correct. {hand_str} ({cat_name}) from {pos_str} merits an open-raise. \
                         A standard 2-3x sizing builds a pot with the best of it and denies \
                         equity to weaker holdings.")
            } else {
                format!("Raising with a {cat_name} hand from {pos_str} over-commits your stack \
                         ({stack_bb} BB) with insufficient equity. A fold is better.")
            };

            let answers = vec![
                AnswerOption {
                    id: "A".to_string(),
                    text: "Fold".to_string(),
                    is_correct: correct == "A",
                    explanation: match text_style {
                        TextStyle::Simple => if correct == "A" {
                            format!("Correct. {hand_str} is too weak for {pos_str}. Folding saves your chips for a better hand.")
                        } else {
                            format!("Folding is a mistake here. {hand_str} is strong enough to bet from {pos_str}. Don't throw away the opportunity.")
                        },
                        TextStyle::Technical => format!(
                            "Folding {hand_str} ({cat_name}) from {pos_str} with {stack_bb} BB: \
                             {fold_body}"
                        ),
                    },
                },
                AnswerOption {
                    id: "B".to_string(),
                    text: format!("Raise to {} BB", open_size / bb),
                    is_correct: correct == "B",
                    explanation: match text_style {
                        TextStyle::Simple => if correct == "B" {
                            format!("Raise! {hand_str} is a good hand in {pos_str}. Bet {open_size} chips and take control of the pot.")
                        } else {
                            format!("Raising is too risky here — {hand_str} isn't strong enough from {pos_str} with {stack_bb} big blinds. Fold instead.")
                        },
                        TextStyle::Technical => format!(
                            "Raising to {open_size} chips ({} BB) with {hand_str} ({cat_name}) \
                             from {pos_str}: {raise_body}",
                            open_size / bb
                        ),
                    },
                },
                AnswerOption {
                    id: "C".to_string(),
                    text: "Call".to_string(),
                    is_correct: false,
                    explanation: match text_style {
                        TextStyle::Simple => format!(
                            "Just calling the big blind here is a bad idea. It lets everyone in cheaply, and you lose control of the hand. Either raise or fold."
                        ),
                        TextStyle::Technical => format!(
                            "Limping with {hand_str} from {pos_str}: In most cash game formats \
                             limping is a leak — it invites multiway pots with no initiative, \
                             weakening your range and giving the BB a free squeeze opportunity."
                        ),
                    },
                },
            ];
            (pot, 0, q, answers)
        }

        // ---- Hero faces a single raise ------------------------------------
        PreflopSpot::FacingOpen => {
            let raiser_size = if stack_bb >= 40 { bb * 3 } else { bb * 2 };
            let pot = bb / 2 + bb + raiser_size; // SB + BB + open
            let three_bet = raiser_size * 3;
            let q = match text_style {
                TextStyle::Simple => format!(
                    "You have {hand_str} in {pos_str} ({stack_bb} big blinds). \
                     Someone raised to {} big blinds. What do you do?",
                    raiser_size / bb
                ),
                TextStyle::Technical => format!(
                    "You hold {hand_str} in {pos_str} ({stack_bb} BB deep). \
                     A player raises to {} BB. Action is on you. What do you do?",
                    raiser_size / bb
                ),
            };
            // Single correct answer to guarantee invariant.
            let correct = match cat {
                HandCategory::Premium  => "C",  // 3-bet for value
                HandCategory::Strong   => "C",  // 3-bet (may also call, 3-bet is cleaner)
                HandCategory::Playable => if pos.is_late() { "C" } else { "B" },
                HandCategory::Marginal => "A",  // fold regardless of position
                HandCategory::Trash    => if pos.is_late() && stack_bb >= 25 { "C" } else { "A" },
            };

            let fold_body = if correct == "A" {
                format!("Correct. A {cat_name} hand vs a raise from {pos_str} is an easy fold. \
                         This hand lacks equity to continue profitably against a raising range.")
            } else {
                format!("Too tight. {hand_str} ({cat_name}) has sufficient equity against a \
                         typical raising range to continue from {pos_str}.")
            };
            let call_body = if correct == "B" {
                format!("Correct. Calling with {hand_str} ({cat_name}) from {pos_str} retains \
                         pot control and lets you realize equity with positional advantage.")
            } else if correct == "A" {
                format!("Calling with a {cat_name} hand invests chips without sufficient equity. \
                         A fold is cleaner from {pos_str}.")
            } else {
                format!("Calling with {hand_str} ({cat_name}) is too passive — a 3-bet for \
                         value is higher EV here from {pos_str}.")
            };
            let threebet_body = if correct == "C" {
                format!("Correct. A 3-bet with {hand_str} ({cat_name}) from {pos_str} extracts \
                         value from worse hands, denies equity, and builds a pot with an equity \
                         advantage.")
            } else {
                format!("3-betting a {cat_name} hand bloats the pot unfavourably from {pos_str}. \
                         You risk a 4-bet or playing a large pot with insufficient hand strength.")
            };

            let answers = vec![
                AnswerOption {
                    id: "A".to_string(),
                    text: "Fold".to_string(),
                    is_correct: correct == "A",
                    explanation: match text_style {
                        TextStyle::Simple => if correct == "A" {
                            format!("Correct. {hand_str} from {pos_str} isn't strong enough to call or re-raise here. Save your chips.")
                        } else {
                            format!("Folding is too cautious — {hand_str} is good enough to continue here.")
                        },
                        TextStyle::Technical => format!(
                            "Folding {hand_str} ({cat_name}) vs a raise from {pos_str}: {fold_body}"
                        ),
                    },
                },
                AnswerOption {
                    id: "B".to_string(),
                    text: "Call".to_string(),
                    is_correct: correct == "B",
                    explanation: match text_style {
                        TextStyle::Simple => if correct == "B" {
                            format!("Correct. Call with {hand_str} from {pos_str}. You have a decent hand and a good position — see the flop.")
                        } else if correct == "A" {
                            format!("Calling with {hand_str} isn't worth it — this hand can't beat a raise. Fold.")
                        } else {
                            format!("Calling is too passive here — re-raise with {hand_str} to build the pot while you have the advantage.")
                        },
                        TextStyle::Technical => format!(
                            "Calling with {hand_str} ({cat_name}) in {pos_str}: {call_body}"
                        ),
                    },
                },
                AnswerOption {
                    id: "C".to_string(),
                    text: format!("Raise to {} BB", three_bet / bb),
                    is_correct: correct == "C",
                    explanation: match text_style {
                        TextStyle::Simple => if correct == "C" {
                            format!("Re-raise! {hand_str} from {pos_str} is strong enough to bet big. This builds the pot when you have the best hand.")
                        } else {
                            format!("Re-raising {hand_str} here is too risky. You'd be putting in a lot of chips with a hand that isn't strong enough.")
                        },
                        TextStyle::Technical => format!(
                            "3-betting to {three_bet} with {hand_str} ({cat_name}) from \
                             {pos_str}: {threebet_body}"
                        ),
                    },
                },
            ];
            (pot, raiser_size, q, answers)
        }

        // ---- Hero faces a 3-bet after opening ----------------------------
        PreflopSpot::ThreeBetPot => {
            let hero_open = bb * 3;
            let three_bet_size = hero_open * 3;
            let pot = bb / 2 + bb + hero_open + three_bet_size;
            let four_bet = three_bet_size * 3;
            let q = match text_style {
                TextStyle::Simple => format!(
                    "You bet {} big blinds with {hand_str} from {pos_str} \
                     ({stack_bb} big blinds). Your opponent re-raised to {} big blinds. What do you do?",
                    hero_open / bb,
                    three_bet_size / bb
                ),
                TextStyle::Technical => format!(
                    "You opened to {} BB with {hand_str} from {pos_str} \
                     ({stack_bb} BB deep). A player re-raises to {} BB. What do you do?",
                    hero_open / bb,
                    three_bet_size / bb
                ),
            };
            let correct = match cat {
                HandCategory::Premium  => "C", // 4-bet for value
                HandCategory::Strong   => "B", // call or small 4-bet
                HandCategory::Playable => "B", // call with implied odds
                HandCategory::Marginal => "A", // fold
                HandCategory::Trash    => "A", // fold
            };

            let fold_body = if correct == "A" {
                format!("Correct. A {cat_name} hand cannot profitably continue vs a 3-bet \
                         given the pot odds and likely 3-bet range of your opponent.")
            } else {
                "Folding is too tight here. Your hand has sufficient equity against a \
                 balanced 3-bet range to continue.".to_string()
            };
            let call_body = if correct == "B" {
                "Correct. Calling preserves stack depth and lets you navigate postflop with \
                 good implied odds, avoiding getting all-in preflop with a dominated or \
                 marginal hand.".to_string()
            } else {
                "Simply calling here leaves money on the table with a premium holding — \
                 4-betting for value is higher EV against a 3-bet range.".to_string()
            };
            let fourbet_body = if correct == "C" {
                "Correct. With a premium hand you should 4-bet for value. This polarizes \
                 your range, forces folds of hands with equity against you, and builds the \
                 pot with the best of it.".to_string()
            } else {
                format!("4-betting a {cat_name} hand turns your hand face-up and gets called \
                         (or shoved) by hands that dominate you, resulting in a large pot \
                         with negative EV.")
            };

            let answers = vec![
                AnswerOption {
                    id: "A".to_string(),
                    text: "Fold".to_string(),
                    is_correct: correct == "A",
                    explanation: match text_style {
                        TextStyle::Simple => if correct == "A" {
                            format!("Correct. {hand_str} can't beat your opponent's re-raise range profitably. Let this one go.")
                        } else {
                            format!("Folding here is too cautious — you have enough of a hand to continue.")
                        },
                        TextStyle::Technical => format!(
                            "Folding {hand_str} ({cat_name}) vs 3-bet: {fold_body}"
                        ),
                    },
                },
                AnswerOption {
                    id: "B".to_string(),
                    text: "Call".to_string(),
                    is_correct: correct == "B",
                    explanation: match text_style {
                        TextStyle::Simple => if correct == "B" {
                            format!("Correct. Call and see the flop. {hand_str} has good enough potential and you keep the pot manageable.")
                        } else {
                            format!("Just calling here wastes the opportunity — re-raise for value with this strong hand.")
                        },
                        TextStyle::Technical => format!(
                            "Calling the 3-bet with {hand_str} ({cat_name}) from {pos_str}: \
                             {call_body}"
                        ),
                    },
                },
                AnswerOption {
                    id: "C".to_string(),
                    text: format!("Raise to {} BB", four_bet / bb),
                    is_correct: correct == "C",
                    explanation: match text_style {
                        TextStyle::Simple => if correct == "C" {
                            format!("Correct. Re-raise again! {hand_str} is a premium hand. Build the pot — you have the best of it here.")
                        } else {
                            format!("Re-raising here puts too many chips at risk with {hand_str}. Call or fold instead.")
                        },
                        TextStyle::Technical => format!(
                            "4-betting with {hand_str} ({cat_name}): {fourbet_body}"
                        ),
                    },
                },
            ];
            (pot, three_bet_size, q, answers)
        }
    }
}
