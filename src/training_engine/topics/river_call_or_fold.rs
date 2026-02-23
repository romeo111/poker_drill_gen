use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    models::{
        AnswerOption, Card, DifficultyLevel, GameType, PlayerState,
        Position, TableSetup, TrainingScenario, TrainingTopic,
    },
};

/// Hero's made-hand strength on the river when facing a villain bet.
#[derive(Debug, Clone, Copy)]
enum HandStrength {
    Strong,   // Two pair+, top pair strong kicker — raise to extract maximum value
    Marginal, // Top pair weak kicker, middle pair — call at standard sizing
    Weak,     // Bottom pair, underpair, missed draw — fold at large sizing
}

/// How large villain's river bet is relative to the pot.
#[derive(Debug, Clone, Copy)]
enum BetSize {
    Small,    // ~33% pot — cheap, pot odds often favour hero; raise with strong hands
    Standard, // ~67% pot — typical value/bluff bet; marginal hands barely +EV to call
    Large,    // ~pot-sized — only strong hands can profitably call
}

impl std::fmt::Display for HandStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandStrength::Strong   => write!(f, "strong hand (two pair+ / top pair strong kicker)"),
            HandStrength::Marginal => write!(f, "marginal hand (top pair weak kicker / middle pair)"),
            HandStrength::Weak     => write!(f, "weak hand (bottom pair / missed draw)"),
        }
    }
}

impl std::fmt::Display for BetSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BetSize::Small    => write!(f, "small (~33%)"),
            BetSize::Standard => write!(f, "standard (~67%)"),
            BetSize::Large    => write!(f, "large (~pot)"),
        }
    }
}

pub fn generate<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
) -> TrainingScenario {
    let mut deck = Deck::new_shuffled(rng);
    let hero_hand: [Card; 2] = [deck.deal(), deck.deal()];
    let board: Vec<Card> = deck.deal_n(5);

    // Three paired scenarios — each maps to a different correct answer
    let (strength, bet_size) = match rng.gen_range(0..3) {
        0 => (HandStrength::Strong,   BetSize::Small),    // raise for value
        1 => (HandStrength::Marginal, BetSize::Standard), // call (barely +EV)
        _ => (HandStrength::Weak,     BetSize::Large),    // fold (overpriced)
    };

    let bb = 2u32;
    let (pot_bb, stack_bb) = match difficulty {
        DifficultyLevel::Beginner     => (rng.gen_range(10..=20u32), 80u32),
        DifficultyLevel::Intermediate => (rng.gen_range(8..=28),     rng.gen_range(30..=100)),
        DifficultyLevel::Advanced     => (rng.gen_range(6..=40),     rng.gen_range(15..=150)),
    };
    let pot   = pot_bb * bb;
    let stack = stack_bb * bb;

    let villain_bet = match bet_size {
        BetSize::Small    => (pot as f32 * 0.33).round() as u32,
        BetSize::Standard => (pot as f32 * 0.67).round() as u32,
        BetSize::Large    => pot,
    };

    // Required equity to break even on a call: call / (pot_facing + call)
    // pot_facing = existing pot + villain's bet; total pot after call = pot_facing + villain_bet
    let required_equity_pct =
        (villain_bet as f32 / (pot as f32 + villain_bet as f32 * 2.0) * 100.0).round() as u32;

    // Hero raise size — relevant only for the Strong:SmallBet branch
    let raise_size = (villain_bet as f32 * 2.5).round() as u32;

    // Correct action:
    // Strong + small bet   → Raise (C): extract maximum value; credible raise range IP
    // Marginal + std bet   → Call  (B): pot odds are marginally +EV with top pair / mid pair
    // Weak + large bet     → Fold  (A): can't call pot-sized bet with bottom pair or missed draw
    let correct: &str = match (strength, bet_size) {
        (HandStrength::Strong,   BetSize::Small)    => "C",
        (HandStrength::Marginal, BetSize::Standard) => "B",
        (HandStrength::Weak,     BetSize::Large)    => "A",
        _                                            => "A",
    };

    let branch_key = match (strength, bet_size) {
        (HandStrength::Strong,   BetSize::Small)    => "Strong:SmallBet:Raise",
        (HandStrength::Marginal, BetSize::Standard) => "Marginal:StdBet:Call",
        (HandStrength::Weak,     BetSize::Large)    => "Weak:LargeBet:Fold",
        _                                            => "Unknown",
    }.to_string();

    let hero_pos  = Position::BTN;
    let hand_str  = format!("{}{}", hero_hand[0], hero_hand[1]);
    let board_str = board.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");

    let question = format!(
        "River call or fold. You hold {hand_str} ({strength}) on the Button. \
         Board: {board_str}. Pot: {pot} chips ({pot_bb} BB). Stack: {stack} chips. \
         Villain bets {villain_bet} chips ({bet_size}) into you. \
         You need ~{required_equity_pct}% equity to break even on a call. \
         What do you do?"
    );

    let answers = vec![
        AnswerOption {
            id: "A".to_string(),
            text: "Fold".to_string(),
            is_correct: correct == "A",
            explanation: match (strength, bet_size) {
                (HandStrength::Weak, BetSize::Large) => format!(
                    "Correct. Folding a {strength} against a {bet_size} bet is right. \
                     You need ~{required_equity_pct}% equity to break even, but a {strength} \
                     is unlikely to have that against a polarised river betting range. \
                     Villain's large bet signals a strong hand or bluff — your weak hand \
                     loses to the former and gains nothing against the latter. Fold."
                ),
                _ => format!(
                    "Folding here surrenders too much value. Against a {bet_size} bet you \
                     need only ~{required_equity_pct}% equity — your {strength} exceeds that. \
                     Either call to realise your equity, or raise to extract more value."
                ),
            },
        },
        AnswerOption {
            id: "B".to_string(),
            text: format!("Call ({villain_bet} chips)"),
            is_correct: correct == "B",
            explanation: match (strength, bet_size) {
                (HandStrength::Marginal, BetSize::Standard) => format!(
                    "Correct. Calling {villain_bet} chips against a {bet_size} bet with a \
                     {strength} is the right play. You need ~{required_equity_pct}% equity \
                     and your hand is likely ahead of villain's bluffing frequency at this \
                     sizing. Folding is too tight; raising turns a thin call into an \
                     aggressive bluff-raise that few worse hands will call."
                ),
                (HandStrength::Strong, BetSize::Small) => format!(
                    "Calling with a {strength} against a {bet_size} bet is fine but leaves \
                     value behind. Villain is likely betting thin for value with hands you \
                     beat — a raise to ~{raise_size} chips extracts more EV and is credible \
                     given your strong range on the river."
                ),
                _ => format!(
                    "Calling {villain_bet} chips with a {strength} against a {bet_size} bet \
                     is -EV. You need ~{required_equity_pct}% equity and your hand is unlikely \
                     to have it against a polarised river bet at this sizing. Fold."
                ),
            },
        },
        AnswerOption {
            id: "C".to_string(),
            text: format!("Raise to {raise_size} chips"),
            is_correct: correct == "C",
            explanation: match (strength, bet_size) {
                (HandStrength::Strong, BetSize::Small) => format!(
                    "Correct. Raising to ~{raise_size} chips with a {strength} against a \
                     {bet_size} villain bet maximises value. A small river bet from villain \
                     often represents a thin value bet or a small bluff — your strong hand \
                     is ahead of much of that range. A raise to ~2.5× the bet is credible \
                     and extracts significantly more EV than a flat call. Villain will call \
                     with weaker top pairs and strong one-pair hands."
                ),
                _ => format!(
                    "Raising with a {strength} against a {bet_size} bet is too aggressive. \
                     A raise commits a large portion of the stack with a hand that cannot \
                     profitably call many re-raises. Only raise on the river when your hand \
                     is strong enough to comfortably stack off."
                ),
            },
        },
    ];

    let players = vec![
        PlayerState { seat: 1, position: Position::BB, stack, is_hero: false, is_active: true },
        PlayerState { seat: 2, position: hero_pos,     stack, is_hero: true,  is_active: true },
    ];

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::RiverCallOrFold,
        branch_key,
        table_setup: TableSetup {
            game_type:     GameType::CashGame,
            hero_position: hero_pos,
            hero_hand,
            board,
            players,
            pot_size:      pot,
            current_bet:   villain_bet,
        },
        question,
        answers,
    }
}
