//! Compare policies at recorded information sets, without forcing later human
//! actions after a counterfactual move. Each decision receives paired RNG seeds.
#[path = "support/replay.rs"]
mod replay;
mod support;
use std::{collections::BTreeMap, fs};
use turbo_hearts_api::{Card, GameEvent, Seat};
use turbo_hearts_bot::Algorithm;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(
        args.len() >= 4,
        "audit_game EVENTS.json CANDIDATE BASELINE [SEEDS]"
    );
    let events: Vec<GameEvent> =
        serde_json::from_str(&fs::read_to_string(&args[1]).unwrap()).unwrap();
    let seeds: u64 = args.get(4).map_or(32, |s| s.parse().unwrap());
    assert!(seeds > 0, "SEEDS must be positive");
    let mut replay = replay::Replay::new();
    let mut hand = 0;
    let mut trick = 1;
    let mut decisions = Vec::new();
    let mut scores = Vec::new();
    let mut bots = [false; 4];
    for (index, event) in events.iter().enumerate() {
        match event {
            GameEvent::Sit {
                north,
                east,
                south,
                west,
                ..
            } => {
                bots = [north, east, south, west].map(|p| p.strategy().is_some());
            }
            GameEvent::Deal { .. } => {
                hand += 1;
                trick = 1;
            }
            GameEvent::Play { seat, card } => {
                let own = &replay.states[seat.idx()];
                let state = &replay.observations[seat.idx()];
                let legal = state.legal_plays(own.post_pass_hand);
                let slough = !state.current_trick.is_empty()
                    && !legal.contains_any(state.current_trick.suit().cards());
                let mut counts = [BTreeMap::new(), BTreeMap::new()];
                let mut means = [BTreeMap::new(), BTreeMap::new()];
                let mut changed = 0;
                if bots[seat.idx()]
                    && legal
                        .distinct_plays(state.played, state.current_trick)
                        .len()
                        > 1
                {
                    for seed in 0..seeds {
                        let mut choices = [Card::TwoClubs; 2];
                        for (version, profile) in [&args[2], &args[3]].iter().enumerate() {
                            let mut player = support::Player::new(profile, seed);
                            let mut fork = replay::Replay::new();
                            for event in &events[..index] {
                                fork.apply(event, |bot, state, event| {
                                    if bot.seat == *seat {
                                        player.on_event(bot, state, event);
                                    }
                                });
                            }
                            if let Some(search) = &player.search {
                                for (c, v) in search.clone().evaluate(own, state) {
                                    *means[version].entry(c.to_string()).or_insert(0.0) +=
                                        v as f64 / seeds as f64;
                                }
                            }
                            choices[version] = player.play(own, state);
                            *counts[version]
                                .entry(choices[version].to_string())
                                .or_insert(0) += 1;
                        }
                        changed += (choices[0] != choices[1]) as usize;
                    }
                }
                decisions.push(serde_json::json!({"event_index":index,"hand":hand,"trick":trick,
                    "seat":seat,"is_bot":bots[seat.idx()],"recorded":card,"slough":slough,
                    "remaining":(own.post_pass_hand-state.played).to_string(),
                    "trick_cards":format!("{:?}",state.current_trick),
                    "won":format!("{:?}",state.won),"legal":legal,"choices":counts,
                    "changed_seeds":changed,"mean_values":means,
                    "opposing_moon_possible":Seat::VALUES.iter().any(|s| *s!=*seat&&state.won.can_run(*s))}));
            }
            _ => {}
        }
        replay.apply(event, |_, _, e| {
            if let GameEvent::HandComplete {
                north_score,
                east_score,
                south_score,
                west_score,
            } = e
            {
                // Callback runs once per observer; retain just one copy.
                if scores.len() < hand {
                    scores.push([*north_score, *east_score, *south_score, *west_score]);
                }
            }
        });
        if matches!(event, GameEvent::Play { .. }) && replay.game.state.current_trick.is_empty() {
            trick += 1;
        }
    }
    assert!(!decisions.is_empty());
    println!(
        "{}",
        serde_json::json!({"candidate":args[2],"baseline":args[3],"seeds":seeds,
        "scores":scores,"decisions":decisions})
    );
}
