use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    evaluator::{board_texture, BoardTexture},
    models::{
        AnswerOption, Card, DifficultyLevel, GameType, PlayerState,
        Position, TableSetup, TrainingScenario, TrainingTopic,
    },
};

/// Classification of the turn card relative to the flop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TurnCard {
    /// Low card (≤ 9) that doesn't complete obvious draws.
    Blank,
    /// Broadway card (T+) that hits many 3-bet / continuation ranges.
    ScareBroadway,
    /// Card that could complete a flush or straight draw.
    DrawComplete,
}

fn classify_turn(flop: &[Card], turn: &Card) -> TurnCard {
    // Check if turn completes flush: 2+ flop cards share suit with turn
    let turn_suit_count = flop.iter().filter(|c| c.suit == turn.suit).count();
    if turn_suit_count >= 2 {
        return TurnCard::DrawComplete;
    }

    // Check if turn completes a straight: gather flop ranks + turn rank, look for 5-in-a-row
    let mut ranks: Vec<u8> = flop.iter().map(|c| c.rank.0).collect();
    ranks.push(turn.rank.0);
    ranks.sort_unstable();
    ranks.dedup();
    for window in ranks.windows(4) {
        if window[3] - window[0] <= 4 {
            return TurnCard::DrawComplete;
        }
    }

    // Scare Broadway
    if turn.rank.0 >= 10 {
        return TurnCard::ScareBroadway;
    }

    TurnCard::Blank
}

pub fn generate<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
) -> TrainingScenario {
    let mut deck = Deck::new_shuffled(rng);

    let hero_hand: [Card; 2] = [deck.deal(), deck.deal()];
    let flop: Vec<Card> = deck.deal_n(3);
    let turn = deck.deal();

    let texture = board_texture(&flop);
    let turn_type = classify_turn(&flop, &turn);

    let bb = 2u32;
    let (stack_bb, pot_bb) = match difficulty {
        DifficultyLevel::Beginner     => (100u32, rng.gen_range(14..=22)),
        DifficultyLevel::Intermediate => (rng.gen_range(50..=130), rng.gen_range(10..=28)),
        DifficultyLevel::Advanced     => (rng.gen_range(25..=200), rng.gen_range(8..=40)),
    };
    let pot = pot_bb * bb;
    let stack = stack_bb * bb;

    // Hero is IP (acted last on flop as aggressor)
    let hero_pos = if rng.gen_bool(0.5) { Position::BTN } else { Position::CO };

    let players = vec![
        PlayerState {
            seat: 1,
            position: Position::BB,
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

    let branch_key = match turn_type {
        TurnCard::DrawComplete  => "DrawComplete".to_string(),
        TurnCard::ScareBroadway => "ScareBroadway".to_string(),
        TurnCard::Blank => match texture {
            BoardTexture::Wet | BoardTexture::SemiWet => "Blank:Wet".to_string(),
            BoardTexture::Dry                         => "Blank:Dry".to_string(),
        },
    };

    let flop_str = flop.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
    let hand_str = format!("{}{}", hero_hand[0], hero_hand[1]);
    let pos_str = format!("{}", hero_pos);
    let texture_str = format!("{}", texture);
    let turn_str = turn.to_string();

    let turn_label = match turn_type {
        TurnCard::Blank         => "blank",
        TurnCard::ScareBroadway => "scare Broadway card",
        TurnCard::DrawComplete  => "draw-completing card",
    };

    let question = format!(
        "You c-bet the flop and villain called. You hold {hand_str} from {pos_str}. \
         Flop: {flop_str} ({texture_str}). Turn: {turn_str} (a {turn_label}). \
         Pot is {pot} chips ({pot_bb} BB), stack {stack} chips ({stack_bb} BB). \
         Villain checks to you. What is your turn action?"
    );

    // Determine correct answer (single ID):
    // DrawComplete → A (Check — draws completed, villain's range strengthened)
    // ScareBroadway + IP → C (Large barrel — scare card polarises range in hero's favour)
    // Blank + wet flop → B (Medium barrel — charge remaining draws)
    // Blank + dry flop → A (Check — no value barreling air on dry board)
    let correct: &str = match turn_type {
        TurnCard::DrawComplete  => "A",
        TurnCard::ScareBroadway => "C",
        TurnCard::Blank => {
            if matches!(texture, BoardTexture::Wet | BoardTexture::SemiWet) {
                "B"
            } else {
                "A"
            }
        }
    };

    let bet_50 = pot / 2;
    let bet_80 = pot * 4 / 5;

    let check_exp = match turn_type {
        TurnCard::DrawComplete => format!(
            "Correct. The {turn_str} completes potential draws — villain's check-calling range \
             is now stronger and your bluff equity has collapsed. Checking back controls the pot \
             and takes a free showdown or river spot."
        ),
        TurnCard::ScareBroadway => format!(
            "The {turn_str} is a scare card that actually hits your late-position preflop \
             range harder than villain's calling range. Checking surrenders fold equity when \
             barrelling is profitable."
        ),
        TurnCard::Blank => {
            if correct == "A" {
                format!(
                    "Correct. On a {texture_str} dry board a blank turn ({turn_str}) gives you \
                     no reason to barrel without a value hand or clear draw. Checking back to \
                     control pot size is the strongest play."
                )
            } else {
                format!(
                    "Checking on a {texture_str} board with draws still live gives villain a \
                     free card. You should charge draws with a medium-sized bet."
                )
            }
        }
    };

    let bet50_exp = match turn_type {
        TurnCard::DrawComplete => format!(
            "Barrelling into a completed draw is a leak. The {turn_str} strengthens villain's \
             check-calling range; a bet risks getting check-raised or called by made hands \
             that now beat you."
        ),
        TurnCard::ScareBroadway => format!(
            "A 50% pot bet is an option but undersizes the scare-card advantage. When a \
             Broadway card ({turn_str}) hits, your polarised range can support a larger barrel \
             to maximise fold equity from villain's medium-strength hands."
        ),
        TurnCard::Blank => {
            if correct == "B" {
                format!(
                    "Correct. A ~50% pot barrel on a {texture_str} board gives villain \
                     incorrect pot odds to continue with flush draws (~20% equity on the turn). \
                     It charges draws without over-committing."
                )
            } else {
                format!(
                    "Betting 50% pot on a {texture_str} dry board without a value hand or draw \
                     is a marginal bluff with little fold equity. Checking back is higher EV."
                )
            }
        }
    };

    let bet80_exp = match turn_type {
        TurnCard::DrawComplete => format!(
            "A large barrel into a completed draw board is a significant mistake. Villain's \
             check-calling range is polarised toward made hands after the {turn_str}; an \
             80% pot bet as a bluff has very low fold equity and costs you a lot when called."
        ),
        TurnCard::ScareBroadway => format!(
            "Correct. An ~80% pot barrel on the {turn_str} leverages the scare card to \
             maximise fold equity. Your range (opening from {pos_str}) is heavily weighted \
             toward Broadway cards, making this bet highly credible and difficult for \
             villain's medium pairs and draws to continue against."
        ),
        TurnCard::Blank => format!(
            "An 80% pot bet on a blank turn without a strong hand or draw over-commits \
             resources. Size down or check back — large barrels on {texture_str} boards \
             without the nuts can become difficult to follow through on the river."
        ),
    };

    let answers = vec![
        AnswerOption {
            id: "A".to_string(),
            text: "Check".to_string(),
            is_correct: correct == "A",
            explanation: check_exp,
        },
        AnswerOption {
            id: "B".to_string(),
            text: format!("Bet ~50% pot ({} chips)", bet_50),
            is_correct: correct == "B",
            explanation: bet50_exp,
        },
        AnswerOption {
            id: "C".to_string(),
            text: format!("Bet ~80% pot ({} chips)", bet_80),
            is_correct: correct == "C",
            explanation: bet80_exp,
        },
    ];

    let mut board = flop;
    board.push(turn);

    let table_setup = TableSetup {
        game_type: GameType::CashGame,
        hero_position: hero_pos,
        hero_hand,
        board,
        players,
        pot_size: pot,
        current_bet: 0,
    };

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::TurnBarrelDecision,
        branch_key,
        table_setup,
        question,
        answers,
    }
}
