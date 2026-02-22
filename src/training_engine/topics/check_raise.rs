use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    evaluator::{has_flush_draw, has_straight_draw},
    models::{
        AnswerOption, Card, DifficultyLevel, GameType, PlayerState,
        Position, Suit, TableSetup, TrainingScenario, TrainingTopic,
    },
};

/// How much the flop board favours the BB caller vs the IP raiser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoardFavour {
    /// Low/connected board (rank sum ≤ 20) — hits BB's wide range.
    BBFavorable,
    /// High/dry board — hits IP raiser's strong preflop range.
    IPFavorable,
}

/// How the hero hand interacts with the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HandInteraction {
    /// Pairs a board card (strong made hand on the flop).
    Strong,
    /// Has a flush draw and/or straight draw.
    Draw,
    /// No pair, no draw.
    Weak,
}

fn classify_board(board: &[Card]) -> BoardFavour {
    let sum: u16 = board.iter().map(|c| c.rank.0 as u16).sum();
    if sum <= 20 {
        BoardFavour::BBFavorable
    } else {
        BoardFavour::IPFavorable
    }
}

fn suit_index(s: Suit) -> usize {
    match s {
        Suit::Clubs    => 0,
        Suit::Diamonds => 1,
        Suit::Hearts   => 2,
        Suit::Spades   => 3,
    }
}

/// True if the hero's hole cards include a card that shares a suit with a board card,
/// and the board already has at least one card of that suit (flush draw potential).
fn hero_has_flush_draw(hand: [Card; 2], board: &[Card]) -> bool {
    let mut board_suit_counts = [0u8; 4];
    for c in board {
        board_suit_counts[suit_index(c.suit)] += 1;
    }
    hand.iter()
        .any(|c| board_suit_counts[suit_index(c.suit)] >= 1)
        && has_flush_draw(board)
}

/// True if hero holds a card whose rank is within 2 of any board card rank, and the
/// board itself has connected cards — indicating hero participates in a straight draw.
fn hero_has_straight_draw(hand: [Card; 2], board: &[Card]) -> bool {
    if !has_straight_draw(board) {
        return false;
    }
    let board_ranks: Vec<u8> = board.iter().map(|c| c.rank.0).collect();
    hand.iter().any(|hc| {
        board_ranks.iter().any(|&br| {
            let diff = if hc.rank.0 > br { hc.rank.0 - br } else { br - hc.rank.0 };
            diff <= 3
        })
    })
}

fn classify_hand_interaction(hand: [Card; 2], board: &[Card]) -> HandInteraction {
    let flush = hero_has_flush_draw(hand, board);
    let straight = hero_has_straight_draw(hand, board);

    if flush || straight {
        return HandInteraction::Draw;
    }

    // Strong: hero pairs any board card
    let board_ranks: Vec<u8> = board.iter().map(|c| c.rank.0).collect();
    let hits_board = hand.iter().any(|c| board_ranks.contains(&c.rank.0));
    if hits_board {
        HandInteraction::Strong
    } else {
        HandInteraction::Weak
    }
}

/// True when the hero hand has both a flush draw AND a straight draw component.
fn is_combo_draw(hand: [Card; 2], board: &[Card]) -> bool {
    hero_has_flush_draw(hand, board) && hero_has_straight_draw(hand, board)
}

pub fn generate<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
) -> TrainingScenario {
    let mut deck = Deck::new_shuffled(rng);

    let hero_hand: [Card; 2] = [deck.deal(), deck.deal()];
    let board: Vec<Card> = deck.deal_n(3);

    let board_favour = classify_board(&board);
    let interaction = classify_hand_interaction(hero_hand, &board);
    let combo = is_combo_draw(hero_hand, &board);

    let branch_key = match (board_favour, interaction, combo) {
        (BoardFavour::BBFavorable, HandInteraction::Strong, _)   => "BBFav:Strong".to_string(),
        (BoardFavour::BBFavorable, HandInteraction::Draw, true)  => "BBFav:ComboDraw".to_string(),
        (BoardFavour::BBFavorable, HandInteraction::Draw, false) => "BBFav:Draw".to_string(),
        (BoardFavour::BBFavorable, HandInteraction::Weak, _)     => "BBFav:Weak".to_string(),
        (BoardFavour::IPFavorable, HandInteraction::Strong, _)   => "IPFav:Strong".to_string(),
        (BoardFavour::IPFavorable, HandInteraction::Draw, true)  => "IPFav:ComboDraw".to_string(),
        (BoardFavour::IPFavorable, HandInteraction::Draw, false) => "IPFav:Draw".to_string(),
        (BoardFavour::IPFavorable, HandInteraction::Weak, _)     => "IPFav:Weak".to_string(),
    };

    let bb = 2u32;
    let (stack_bb, pot_bb) = match difficulty {
        DifficultyLevel::Beginner     => (100u32, rng.gen_range(8..=14)),
        DifficultyLevel::Intermediate => (rng.gen_range(50..=130), rng.gen_range(6..=20)),
        DifficultyLevel::Advanced     => (rng.gen_range(20..=200), rng.gen_range(4..=30)),
    };
    let pot = pot_bb * bb;
    let stack = stack_bb * bb;

    // Villain (IP) bets ~50–70% pot
    let villain_bet_pct: u32 = rng.gen_range(50..=70);
    let villain_bet = (pot * villain_bet_pct / 100).max(bb);
    let cr_size = villain_bet * 5 / 2; // 2.5× raise

    let hero_pos = Position::BB;
    let villain_pos = Position::BTN;

    let board_str = board.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
    let hand_str = format!("{}{}", hero_hand[0], hero_hand[1]);
    let board_favour_str = match board_favour {
        BoardFavour::BBFavorable => "BB-favorable (low/connected)",
        BoardFavour::IPFavorable => "IP-favorable (high/dry)",
    };
    let interaction_str = match (interaction, combo) {
        (HandInteraction::Draw, true)  => "combo draw",
        (HandInteraction::Draw, false) => "draw",
        (HandInteraction::Strong, _)   => "strong hand (pairs the board)",
        (HandInteraction::Weak, _)     => "weak/air",
    };

    // Correct answer (single ID):
    // BBFavorable + Strong → "C" (Check-raise for value)
    // ComboDraw on any board → "C" (Check-raise semi-bluff)
    // Weak + IPFavorable → "A" (Fold)
    // Everything else → "B" (Check-call)
    let correct: &str = match (board_favour, interaction) {
        (BoardFavour::BBFavorable, HandInteraction::Strong) => "C",
        (_, HandInteraction::Draw) if combo                 => "C",
        (BoardFavour::IPFavorable, HandInteraction::Weak)   => "A",
        _                                                   => "B",
    };

    let question = format!(
        "You are in the Big Blind (OOP). Flop: {board_str} ({board_favour_str}). \
         You hold {hand_str} ({interaction_str}). \
         Villain on the Button bets {villain_bet} chips ({villain_bet_pct}% pot). \
         Pot is {pot} chips ({pot_bb} BB). Stack: {stack} chips ({stack_bb} BB). \
         What is your action?"
    );

    let fold_exp = if matches!((board_favour, interaction),
        (BoardFavour::IPFavorable, HandInteraction::Weak)) {
        format!(
            "Correct. With {interaction_str} on a {board_favour_str} board ({board_str}), \
             you have no pair, no draw, and the board heavily favours villain's preflop range. \
             Calling invests {villain_bet} chips with almost no equity. Fold."
        )
    } else {
        format!(
            "Folding {hand_str} ({interaction_str}) here is too tight. You have enough \
             equity or positional leverage to continue, either by calling or check-raising. \
             A fold surrenders too much to villain's {villain_bet_pct}% pot bet."
        )
    };

    let call_exp = if correct == "B" {
        format!(
            "Correct. Check-calling with {hand_str} ({interaction_str}) on {board_str} \
             is the best play. You have equity to continue but not the ideal conditions for \
             a check-raise (either the board doesn't favour your range, or your draw alone \
             doesn't justify building a large pot OOP). Call {villain_bet} and re-evaluate \
             on the turn."
        )
    } else if matches!((board_favour, interaction),
        (BoardFavour::BBFavorable, HandInteraction::Strong)) {
        format!(
            "Check-calling with a strong hand on a BB-favorable board leaves value on the \
             table. You should check-raise to {cr_size} chips to build the pot while you're \
             ahead and deny villain's equity from backdoor draws and overcards."
        )
    } else {
        format!(
            "Check-calling is passive here. With {interaction_str} on {board_str}, a \
             check-raise to {cr_size} chips extracts more value and applies pressure. \
             Calling gives villain a free turn card to improve or bluff again cheaply."
        )
    };

    let cr_exp = match (board_favour, interaction, correct) {
        (BoardFavour::BBFavorable, HandInteraction::Strong, "C") => format!(
            "Correct. Check-raising to {cr_size} chips (2.5× villain's {villain_bet}) with \
             {hand_str} ({interaction_str}) on a {board_favour_str} board ({board_str}) is \
             the highest-EV play. This board hits your BB defending range (low/connected) \
             much harder than villain's late-position range. You protect your hand, build \
             the pot with the best of it, and deny villain cheap equity."
        ),
        (_, HandInteraction::Draw, "C") => format!(
            "Correct. Check-raising to {cr_size} chips (2.5× villain's {villain_bet}) as a \
             combo-draw semi-bluff with {hand_str} on {board_str} is correct. Your combo \
             draw has ~54% equity on the flop — you are a slight favourite! The check-raise \
             wins the pot outright when villain folds, and builds a large pot when villain \
             calls into your equity advantage."
        ),
        (BoardFavour::IPFavorable, _, _) => format!(
            "Check-raising on a {board_favour_str} board ({board_str}) with {hand_str} \
             ({interaction_str}) is a bluff into villain's strongest range. This board \
             connects heavily with late-position preflop hands; your check-raise has very \
             low fold equity and risks getting 3-bet off a weak hand."
        ),
        _ => format!(
            "Check-raising with only a {interaction_str} (not a combo draw) may be \
             too aggressive here. Without either a very strong made hand or a combo draw, \
             the check-raise over-commits chips OOP without sufficient equity to back it up."
        ),
    };

    let answers = vec![
        AnswerOption {
            id: "A".to_string(),
            text: "Fold".to_string(),
            is_correct: correct == "A",
            explanation: fold_exp,
        },
        AnswerOption {
            id: "B".to_string(),
            text: "Check-call".to_string(),
            is_correct: correct == "B",
            explanation: call_exp,
        },
        AnswerOption {
            id: "C".to_string(),
            text: format!("Check-raise to {} chips (2.5× bet)", cr_size),
            is_correct: correct == "C",
            explanation: cr_exp,
        },
    ];

    let players = vec![
        PlayerState {
            seat: 1,
            position: hero_pos,
            stack,
            is_hero: true,
            is_active: true,
        },
        PlayerState {
            seat: 2,
            position: villain_pos,
            stack,
            is_hero: false,
            is_active: true,
        },
    ];

    let table_setup = TableSetup {
        game_type: GameType::CashGame,
        hero_position: hero_pos,
        hero_hand,
        board,
        players,
        pot_size: pot,
        current_bet: villain_bet,
    };

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::CheckRaiseSpot,
        branch_key,
        table_setup,
        question,
        answers,
    }
}
