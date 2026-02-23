use rand::Rng;
use crate::training_engine::{
    deck::Deck,
    models::{
        AnswerOption, Card, DifficultyLevel, GameType, PlayerState,
        Position, TableSetup, TextStyle, TrainingScenario, TrainingTopic,
    },
};

#[derive(Debug, Clone, Copy)]
pub enum TournamentStage {
    EarlyLevels,
    MiddleStages,
    Bubble,
    FinalTable,
}

impl std::fmt::Display for TournamentStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TournamentStage::EarlyLevels  => write!(f, "Early Levels"),
            TournamentStage::MiddleStages => write!(f, "Middle Stages"),
            TournamentStage::Bubble       => write!(f, "Bubble"),
            TournamentStage::FinalTable   => write!(f, "Final Table"),
        }
    }
}

fn random_stage<R: Rng>(rng: &mut R) -> TournamentStage {
    match rng.gen_range(0..4) {
        0 => TournamentStage::EarlyLevels,
        1 => TournamentStage::MiddleStages,
        2 => TournamentStage::Bubble,
        _ => TournamentStage::FinalTable,
    }
}

/// Hand strength tiers for push/fold (simplified).
#[derive(Debug, Clone, Copy)]
enum PushTier {
    Premium,  // AA, KK, QQ, AKs — always push short stacks
    Strong,   // JJ, TT, AQ, AK — push at moderate depths
    Playable, // Mid pairs, suited broadways — push only when short
    Weak,     // Everything else — only push when desperate
}

fn classify_push_tier(hand: [Card; 2]) -> PushTier {
    let (r1, r2) = {
        let mut ranks = [hand[0].rank.0, hand[1].rank.0];
        ranks.sort_unstable_by(|a, b| b.cmp(a));
        (ranks[0], ranks[1])
    };
    let suited = hand[0].suit == hand[1].suit;
    let pair = r1 == r2;

    if pair && r1 >= 12 { return PushTier::Premium; }       // QQ+
    if r1 == 14 && r2 == 13 && suited { return PushTier::Premium; } // AKs
    if pair && r1 >= 10 { return PushTier::Strong; }         // JJ, TT
    if r1 == 14 && r2 >= 12 { return PushTier::Strong; }    // AK, AQ
    if pair && r1 >= 7 { return PushTier::Playable; }        // 77-99
    if r1 == 14 && r2 >= 10 && suited { return PushTier::Playable; } // ATs+
    if r1 >= 12 && r2 >= 11 && suited { return PushTier::Playable; } // KQs, KJs, QJs
    PushTier::Weak
}

/// Simplified ICM pressure: base threshold in BB modified by hand strength.
/// Real ICM requires knowing payouts; here we use simplified thresholds.
fn push_threshold_bb(stage: TournamentStage, tier: PushTier) -> u32 {
    let base = match stage {
        TournamentStage::EarlyLevels  => 20,
        TournamentStage::MiddleStages => 15,
        TournamentStage::Bubble       => 10,
        TournamentStage::FinalTable   => 12,
    };
    // Premium hands can push at deeper stacks; weak hands need more desperation
    match tier {
        PushTier::Premium  => base + 8,
        PushTier::Strong   => base + 3,
        PushTier::Playable => base,
        PushTier::Weak     => base.saturating_sub(4),
    }
}

pub fn generate<R: Rng>(
    rng: &mut R,
    difficulty: DifficultyLevel,
    scenario_id: String,
    text_style: TextStyle,
) -> TrainingScenario {
    let stage = random_stage(rng);
    let bb = 100u32; // tournament chips, 100 = 1 BB

    let hero_stack_bb = match difficulty {
        DifficultyLevel::Beginner     => rng.gen_range(6..=18u32),
        DifficultyLevel::Intermediate => rng.gen_range(4..=25),
        DifficultyLevel::Advanced     => rng.gen_range(3..=30),
    };

    let villain_stack_bb: u32 = rng.gen_range(20..=60);
    let hero_stack = hero_stack_bb * bb;
    let villain_stack = villain_stack_bb * bb;

    let players_remaining = match stage {
        TournamentStage::EarlyLevels  => rng.gen_range(60..=120u32),
        TournamentStage::MiddleStages => rng.gen_range(25..=60),
        TournamentStage::Bubble       => rng.gen_range(10..=18),
        TournamentStage::FinalTable   => rng.gen_range(3..=9),
    };

    let paid_spots = (players_remaining as f32 * 0.15).ceil() as u32;

    let mut deck = Deck::new_shuffled(rng);
    let hero_hand: [Card; 2] = [deck.deal(), deck.deal()];
    let hero_pos = Position::BTN;
    let pos_str = format!("{}", hero_pos);
    let hand_str = format!("{}{}", hero_hand[0], hero_hand[1]);

    let push_tier = classify_push_tier(hero_hand);
    let threshold = push_threshold_bb(stage, push_tier);
    let should_push = hero_stack_bb <= threshold;

    let stage_name = match stage {
        TournamentStage::EarlyLevels  => "Early",
        TournamentStage::MiddleStages => "Middle",
        TournamentStage::Bubble       => "Bubble",
        TournamentStage::FinalTable   => "FinalTable",
    };
    let branch_key = format!("{}:{}", stage_name, if should_push { "Push" } else { "Fold" });

    let pot = bb + bb / 2; // standard antes + blinds estimate

    let risk_premium_pct: f32 = match stage {
        TournamentStage::Bubble       => 20.0,
        TournamentStage::FinalTable   => 15.0,
        TournamentStage::MiddleStages => 8.0,
        TournamentStage::EarlyLevels  => 3.0,
    };

    let question = match text_style {
        TextStyle::Simple => format!(
            "Tournament: {stage}. {players_remaining} players left, top {paid_spots} get paid. \
             You have {hand_str} on the Button with {hero_stack_bb} big blinds. \
             Your opponent in the Big Blind has {villain_stack_bb} big blinds. \
             Everyone else folded. Go all-in or fold?"
        ),
        TextStyle::Technical => format!(
            "Tournament: {stage}. {players_remaining} players remain, top {paid_spots} paid. \
             You hold {hand_str} on the {pos_str} with {hero_stack_bb} BB. \
             Villain on the BB has {villain_stack_bb} BB. \
             Action folds to you. Do you shove all-in or fold?"
        ),
    };

    let push_body = if should_push {
        format!(
            "Correct. At {hero_stack_bb} BB, your stack faces significant blind pressure \
             (you'll lose ~{:.0}% per orbit). ICM risk premium at this stage is ~{risk_premium_pct:.0}%, \
             but your hand still has enough equity to profitably shove against a \
             wide BB calling range. Stack preservation via folding only deepens the \
             blinds crisis.",
            100.0 / hero_stack_bb as f32
        )
    } else {
        format!(
            "Shoving with {hero_stack_bb} BB is premature. At this stack depth the \
             ICM risk premium (~{risk_premium_pct:.0}% at {stage}) means you \
             over-risk your tournament equity. Wait for a better spot or a stronger \
             hand."
        )
    };
    let push_explanation = match text_style {
        TextStyle::Simple => if should_push {
            format!("Correct — go all-in! With only {hero_stack_bb} big blinds, your stack is shrinking fast. Waiting for a perfect hand will cost you too much. Shove now.")
        } else {
            format!("Going all-in too early at {hero_stack_bb} big blinds risks your tournament life needlessly. You still have time to find a better spot.")
        },
        TextStyle::Technical => format!(
            "Shoving {hero_stack_bb} BB with {hand_str} from {pos_str} during {stage}: {push_body}"
        ),
    };

    let fold_body = if !should_push {
        format!(
            "Correct. With {hero_stack_bb} BB you are not yet in desperation territory. \
             Preserving your stack when ICM pressure is ~{risk_premium_pct:.0}% \
             is rational — a marginal shove risks your entire tournament life \
             for a modest chip gain."
        )
    } else {
        format!(
            "Folding is too passive here. With only {hero_stack_bb} BB and increasing \
             blind levels, you must find spots to accumulate chips. Folding here \
             leaves you critically short and forces even worse all-in spots later \
             with less fold equity."
        )
    };
    let fold_explanation = match text_style {
        TextStyle::Simple => if !should_push {
            format!("Correct — fold. You still have enough chips ({hero_stack_bb} big blinds) to wait for a better spot. Don't risk elimination unnecessarily.")
        } else {
            format!("Folding here is wrong — with {hero_stack_bb} big blinds your stack is getting dangerously low. You need to shove while you still have some chips to be scary.")
        },
        TextStyle::Technical => format!(
            "Folding {hand_str} from {pos_str} with {hero_stack_bb} BB during {stage}: {fold_body}"
        ),
    };

    let answers = vec![
        AnswerOption {
            id: "A".to_string(),
            text: "All-in".to_string(),
            is_correct: should_push,
            explanation: push_explanation,
        },
        AnswerOption {
            id: "B".to_string(),
            text: "Fold".to_string(),
            is_correct: !should_push,
            explanation: fold_explanation,
        },
    ];

    let players = vec![
        PlayerState {
            seat: 1,
            position: Position::BB,
            stack: villain_stack,
            is_hero: false,
            is_active: true,
        },
        PlayerState {
            seat: 2,
            position: hero_pos,
            stack: hero_stack,
            is_hero: true,
            is_active: true,
        },
    ];

    let table_setup = TableSetup {
        game_type: GameType::Tournament,
        hero_position: hero_pos,
        hero_hand,
        board: vec![],
        players,
        pot_size: pot,
        current_bet: 0,
    };

    TrainingScenario {
        scenario_id,
        topic: TrainingTopic::ICMAndTournamentDecision,
        branch_key,
        table_setup,
        question,
        answers,
    }
}
