//! Describe learned passing on the same observable states as the parent policy.
mod support;
use turbo_hearts_api::*;
use turbo_hearts_bot::*;

fn describe(hand: Cards, pass: Cards) -> [f64; 6] {
    let kept = hand - pass;
    let voids = Suit::VALUES
        .iter()
        .filter(|s| (kept & s.cards()).is_empty())
        .count();
    let controls: usize = Suit::VALUES
        .iter()
        .map(|s| {
            s.cards()
                .into_iter()
                .take_while(|c| kept.contains(*c))
                .count()
        })
        .sum();
    [
        pass.into_iter()
            .map(|c| (c.rank().idx() + 2) as f64)
            .sum::<f64>()
            / 3.0,
        pass.into_iter().filter(|c| c.rank() >= Rank::King).count() as f64,
        voids as f64,
        controls as f64,
        (kept & Cards::NINES).len() as f64,
        pass.contains(Card::QueenSpades) as u8 as f64,
    ]
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(args.len() >= 4, "preplay_choices PASS_MODEL DEALS SEED");
    let model = PreplayNetwork::load(&args[1]).unwrap();
    assert_eq!(model.kind(), PreplayKind::Pass);
    let count: u64 = args[2].parse().unwrap();
    let first: u64 = args[3].parse().unwrap();
    let mut n = [0usize; 4];
    let mut changed = [0usize; 4];
    let mut queen = [0usize; 4];
    let mut totals = [[[0.0; 6]; 2]; 4];
    for seed in first..first + count {
        let direction = PassDirection::from((seed % 4) as u8);
        let mut game =
            support::Match::dealt(seed, direction, ["round7:4:8"; 4], ChargingRules::Classic);
        if direction == PassDirection::Keeper {
            game.charge_from(0);
            if !game.game.state.phase.is_passing() {
                continue;
            }
        }
        let d = direction as usize;
        for i in 0..4 {
            let bot = &game.states[i];
            let state = &game.observations[i];
            let parent = HeuristicBot.pass(bot, state);
            let learned = model.choose(bot, state, parent, 0.0);
            assert_eq!(learned.len(), 3);
            assert!(bot.pre_pass_hand.contains_all(learned));
            n[d] += 1;
            changed[d] += (learned != parent) as usize;
            queen[d] += bot.pre_pass_hand.contains(Card::QueenSpades) as usize;
            for (version, action) in [learned, parent].iter().copied().enumerate() {
                for (j, value) in describe(bot.pre_pass_hand, action).iter().enumerate() {
                    totals[d][version][j] += value;
                }
            }
        }
    }
    let directions: Vec<_> = (0..4)
        .filter(|d| n[*d] > 0)
        .map(|d| {
            serde_json::json!({"direction":(["left", "right", "across", "keeper"][d]),
            "decisions": n[d], "changed_fraction": changed[d] as f64 / n[d] as f64,
            "mean_metrics_learned_parent": totals[d].map(|v| v.map(|x| x/n[d] as f64)),
            "passed_queen_fraction_when_held": totals[d].map(|v| v[5] / queen[d] as f64)})
        })
        .collect();
    println!(
        "{}",
        serde_json::json!({"model":args[1],"deals":count,"seed":first,
        "state_distribution":"parent preplay, Classic; keeper only after all decline",
        "metrics":["mean_passed_rank_2_to_14","passed_aces_or_kings","retained_void_suits","retained_top_sequence_cards","retained_nines","passed_queen"],
        "by_direction":directions})
    );
}
