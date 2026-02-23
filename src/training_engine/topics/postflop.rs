use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    evaluator::{board_texture, BoardTexture},
    models::{
        AnswerOption, Card, DifficultyLevel, GameType, PlayerState,
        Position, TableSetup, TextStyle, TrainingScenario, TrainingTopic,
    },
};

pub fn generate<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let mut deck = Deck::new_shuffled(rng);

    let hero_hand: [Card; 2] = [deck.deal(), deck.deal()];
    let board: Vec<Card> = deck.deal_n(3); // flop only

    let texture = board_texture(&board);

    // Stack / pot sizes
    let bb = 2u32;
    let (stack_bb, pot_bb) = match difficulty {
        DifficultyLevel::Beginner     => (100u32, rng.gen_range(8..=14)),
        DifficultyLevel::Intermediate => (rng.gen_range(60..=130), rng.gen_range(6..=20)),
        DifficultyLevel::Advanced     => (rng.gen_range(20..=200), rng.gen_range(4..=30)),
    };
    let pot = pot_bb * bb;
    let stack = stack_bb * bb;

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

    // Range advantage flag (simplified: hero has range adv from late pos on low boards)
    let lowest_rank = board.iter().map(|c| c.rank.0).min().unwrap_or(14);
    let hero_has_range_adv = hero_pos.is_late() && lowest_rank <= 8;

    let board_str = board.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
    let hand_str = format!("{}{}", hero_hand[0], hero_hand[1]);
    let pos_str = format!("{}", hero_pos);
    let texture_str = format!("{}", texture);

    let branch_key = match (&texture, hero_has_range_adv) {
        (BoardTexture::Dry, true)  => "Dry:RangeAdv".to_string(),
        (BoardTexture::Dry, false) => "Dry:NoRangeAdv".to_string(),
        (BoardTexture::SemiWet, _) => "SemiWet".to_string(),
        (BoardTexture::Wet, _)     => "Wet".to_string(),
    };

    let question = match text_style {
        TextStyle::Simple => format!(
            "You bet before the flop and your opponent checked. You have {hand_str} in {pos_str}. \
             The first three cards: {board_str}. Pot: {pot} chips. Stack: {stack} chips. \
             Options: check, bet small (~{} chips), bet big (~{} chips), or overbet (~{} chips). What do you do?",
            pot / 3,
            pot * 3 / 4,
            pot * 5 / 4
        ),
        TextStyle::Technical => format!(
            "You raised preflop and are the aggressor. You hold {hand_str} on {pos_str}. \
             The flop comes {board_str} (a {texture_str} board). The pot is {pot} chips \
             ({pot_bb} BB). Your stack is {stack} chips ({stack_bb} BB). \
             Villain checks to you. Bet options: small (~33% pot = {} chips), \
             large (~75% pot = {} chips), or overbet (~125% pot = {} chips). What do you do?",
            pot / 3,
            pot * 3 / 4,
            pot * 5 / 4
        ),
    };

    let answers = build_cbet_answers(
        &hand_str, &pos_str, &texture_str, &board_str,
        texture.clone(), hero_has_range_adv, pot, stack_bb, difficulty,
        text_style,
    );

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
        topic: TrainingTopic::PostflopContinuationBet,
        branch_key,
        table_setup,
        question,
        answers,
    }
}

fn build_cbet_answers(
    hand_str: &str,
    pos_str: &str,
    texture_str: &str,
    board_str: &str,
    texture: BoardTexture,
    hero_range_adv: bool,
    pot: u32,
    stack_bb: u32,
    _difficulty: DifficultyLevel,
    text_style: TextStyle,
) -> Vec<AnswerOption> {
    // On dry boards with range advantage, 33% is often optimal.
    // On wet boards, larger sizing or check is better.
    let (correct_id, check_rationale, small_rationale, large_rationale, overbet_rationale) =
        match texture {
            BoardTexture::Dry if hero_range_adv => (
                "B",
                match text_style {
                    TextStyle::Simple => format!(
                        "Checking gives your opponent a free card. You're in a strong position here — a small bet is the better play."
                    ),
                    TextStyle::Technical => format!(
                        "Checking with {hand_str} on a {texture_str} board ({board_str}) sacrifices \
                         fold equity and gives villain a free card. From {pos_str} with range \
                         advantage, betting is better than checking."
                    ),
                },
                match text_style {
                    TextStyle::Simple => format!(
                        "Correct. A small bet works well here. The board is dry (no likely draws), so a cheap bet is enough to keep the pressure on and collect chips."
                    ),
                    TextStyle::Technical => format!(
                        "A 33% pot c-bet on a {texture_str} board is correct here. It exploits your \
                         range advantage from {pos_str}, applies pressure at low risk, and denies \
                         equity to villain's backdoor draws and overcards."
                    ),
                },
                match text_style {
                    TextStyle::Simple => format!(
                        "Betting this big on a dry board is too much. A small bet gets the same job done for less risk."
                    ),
                    TextStyle::Technical => format!(
                        "A 75% pot sizing on a {texture_str} board is unnecessarily large. Villain \
                         folds hands you beat and calls with hands that have equity, making this \
                         sizing -EV on a board where a small bet achieves the same goals."
                    ),
                },
                match text_style {
                    TextStyle::Simple => format!(
                        "Overbetting on a dry board is too aggressive — a small bet achieves the same goal more cheaply."
                    ),
                    TextStyle::Technical => format!(
                        "An overbet on a {texture_str} board from {pos_str} is exploitable. \
                         Villain's calling range will have enough equity against an overbet that \
                         you cannot profitably use this sizing as a bluff."
                    ),
                },
            ),
            BoardTexture::Dry => (
                "A",
                match text_style {
                    TextStyle::Simple => format!(
                        "Correct. Check here. The board is dry (no likely draws) and you don't have a big advantage. A free card costs you little and lets you see how the hand develops."
                    ),
                    TextStyle::Technical => format!(
                        "Checking with {hand_str} on a {texture_str} board ({board_str}) from \
                         {pos_str} is correct when you lack clear range advantage. A check allows \
                         you to control pot size and reassess on the turn."
                    ),
                },
                match text_style {
                    TextStyle::Simple => format!(
                        "A small bet can work but checking is safer here — you don't have a clear advantage on this board."
                    ),
                    TextStyle::Technical => format!(
                        "A small c-bet can work here but without range advantage you may be \
                         building a pot when your range is symmetric with villain's. Checking \
                         back and re-evaluating is higher EV from {pos_str}."
                    ),
                },
                match text_style {
                    TextStyle::Simple => format!(
                        "Betting big here over-commits your chips. Check or fold is better when you don't have the advantage."
                    ),
                    TextStyle::Technical => format!(
                        "A 75% pot c-bet on a {texture_str} board without range advantage \
                         over-commits with a polarized-looking line when your range may not \
                         support the sizing."
                    ),
                },
                match text_style {
                    TextStyle::Simple => format!(
                        "Overbetting without an advantage on this board is a big mistake. Check instead."
                    ),
                    TextStyle::Technical => format!(
                        "Overbetting a {texture_str} board without a clear nut advantage is a \
                         leak. Reserve overbets for boards where your range is significantly \
                         stronger than villain's."
                    ),
                },
            ),
            BoardTexture::SemiWet | BoardTexture::Wet => (
                "C",
                match text_style {
                    TextStyle::Simple => format!(
                        "Checking here lets your opponent draw to a better hand for free. Bet to make them pay."
                    ),
                    TextStyle::Technical => format!(
                        "Checking on a {texture_str} board ({board_str}) surrenders too much \
                         equity and gives villain free cards with flush/straight draws. From \
                         {pos_str} you should charge draws with a sizable bet."
                    ),
                },
                match text_style {
                    TextStyle::Simple => format!(
                        "A small bet is too cheap — your opponent can afford to call and try to improve. Bet bigger to make it expensive."
                    ),
                    TextStyle::Technical => format!(
                        "A 33% pot bet on a {texture_str} board is too small — it gives villain \
                         correct pot odds to call with flush draws (~35% equity) without paying \
                         a premium, diluting your fold equity."
                    ),
                },
                match text_style {
                    TextStyle::Simple => format!(
                        "Correct. Bet big! The board has draws (possible flush or straight). Make your opponent pay a lot to try to beat you."
                    ),
                    TextStyle::Technical => format!(
                        "A 75% pot c-bet on a {texture_str} board is correct. It charges draws \
                         incorrect pot odds ({:.0}% required equity vs ~35% actual for flush draw), \
                         protects your hand, and maintains fold equity against weak pairs.",
                        crate::training_engine::evaluator::required_equity(
                            (pot as f32 * 0.75) as u32, pot
                        ) * 100.0
                    ),
                },
                match text_style {
                    TextStyle::Simple => format!(
                        "An overbet on the first three cards is usually too much too soon. A big bet (75%) already does the job."
                    ),
                    TextStyle::Technical => format!(
                        "An overbet on a {texture_str} board with {stack_bb} BB remaining can \
                         be used as a polarized bluff, but is generally reserved for the river \
                         or specific high-equity situations. On the flop it often folds too \
                         much equity and collapses your range."
                    ),
                },
            ),
        };

    vec![
        AnswerOption {
            id: "A".to_string(),
            text: "Check".to_string(),
            is_correct: correct_id == "A",
            explanation: check_rationale,
        },
        AnswerOption {
            id: "B".to_string(),
            text: "Bet small".to_string(),
            is_correct: correct_id == "B",
            explanation: small_rationale,
        },
        AnswerOption {
            id: "C".to_string(),
            text: "Bet large".to_string(),
            is_correct: correct_id == "C",
            explanation: large_rationale,
        },
        AnswerOption {
            id: "D".to_string(),
            text: "Overbet".to_string(),
            is_correct: false,
            explanation: overbet_rationale,
        },
    ]
}
