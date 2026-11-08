//! Inspect a recorded play without exposing opponents' hidden cards to search.
#[path = "support/replay.rs"]
mod replay;
mod support;
use std::{collections::BTreeMap, fs};
use turbo_hearts_api::{can_claim, GameEvent};
use turbo_hearts_bot::Algorithm;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(
        args.len() >= 4,
        "replay_decision EVENTS.json EVENT_INDEX PROFILE [SEEDS]"
    );
    let values: Vec<GameEvent> =
        serde_json::from_str(&fs::read_to_string(&args[1]).unwrap()).unwrap();
    let index: usize = args[2].parse().unwrap();
    let (seat, recorded) = match values[index] {
        GameEvent::Play { seat, card } => (seat, card),
        _ => panic!("expected Play"),
    };
    let seeds: u64 = args.get(4).map_or(100, |v| v.parse().unwrap());
    let mut counts = BTreeMap::new();
    let mut sums = BTreeMap::new();
    let mut claims = BTreeMap::new();
    for seed in 0..seeds {
        let mut player = support::Player::new(&args[3], seed);
        let mut replay = replay::Replay::new();
        for event in &values[..index] {
            replay.apply(event, |bot, state, event| {
                if bot.seat == seat {
                    player.on_event(bot, state, event);
                }
            });
        }
        let own = &replay.states[seat.idx()];
        let state = &replay.observations[seat.idx()];
        assert!(state.legal_plays(own.post_pass_hand).contains(recorded));
        if let Some(search) = &player.search {
            for (card, value) in search.clone().evaluate(own, state) {
                *sums.entry(card.to_string()).or_insert(0.0) += value as f64 / seeds as f64;
            }
        }
        let choice = player.play(own, state);
        *counts.entry(choice.to_string()).or_insert(0) += 1;
        if seed == 0 {
            for card in state.legal_plays(own.post_pass_hand) {
                let event = GameEvent::Play { seat, card };
                let void = own.void.on_event(state, &event);
                let mut after = state.clone();
                after.apply(&event);
                let winner = after.next_actor.unwrap();
                // Privileged information is used ONLY for this diagnostic.
                claims.insert(
                    card.to_string(),
                    after.current_trick.is_empty()
                        && winner != seat
                        && can_claim(
                            &after,
                            void,
                            winner,
                            replay.game.post_pass_hand[winner.idx()] - after.played,
                        ),
                );
            }
        }
    }
    println!(
        "{}",
        serde_json::json!({"profile":args[3],"seeds":seeds,"seat":seat,
        "event_index":index,"recorded":recorded,"choices":counts,"mean_values":sums,
        "opponent_can_claim_with_actual_hand":claims})
    );
}
