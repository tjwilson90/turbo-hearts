//! Responsive action comparisons on historical hands, not final-score imitation.
#[path = "support/counterfactual.rs"]
mod counterfactual;
#[path = "support/replay.rs"]
mod replay;
mod support;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};
use turbo_hearts_api::GameEvent;
use turbo_hearts_bot::{
    human_card_index, neural_human_observation, Algorithm, HumanMemory, HUMAN_ACTIONS,
    HUMAN_INPUTS, HUMAN_ROW,
};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(
        args.len(),
        6,
        "human_payoff_data RANKED INPUT.bin TARGETS.bin SAMPLE_MODULUS SEEDS"
    );
    let modulus: u64 = args[4].parse().unwrap();
    let seeds: u64 = args[5].parse().unwrap();
    assert!(modulus > 0 && seeds > 0);
    let mut out = BufWriter::new(File::create(&args[2]).unwrap());
    let mut targets = BufWriter::new(File::create(&args[3]).unwrap());
    let mut counts = [0; 4];
    for line in BufReader::new(File::open(&args[1]).unwrap()).lines() {
        let row: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let id = row["id"].as_u64().unwrap();
        let split = row["split"].as_u64().unwrap() as usize;
        if split > 2 {
            continue;
        }
        let events: Vec<GameEvent> = serde_json::from_value(row["events"].clone()).unwrap();
        let mut replay = replay::Replay::new();
        let mut memories: [HumanMemory; 4] = std::array::from_fn(|_| HumanMemory::default());
        let mut hand = 0;
        for (index, event) in events.iter().enumerate() {
            if matches!(event, GameEvent::Deal { .. }) {
                hand += 1;
            }
            if let GameEvent::Play { seat, card } = event {
                let own = &replay.states[seat.idx()];
                let state = &replay.observations[seat.idx()];
                if row["skilled"][seat.idx()] == true
                    && state.played.len() < 40
                    && state
                        .legal_plays(own.post_pass_hand)
                        .distinct_plays(state.played, state.current_trick)
                        .len()
                        > 1
                    && (id * 1_000_003 + index as u64 * 37 + seat.idx() as u64) % modulus == 0
                {
                    let (x, actions) = neural_human_observation(own, state, &memories[seat.idx()]);
                    let mut money = [0.0f32; 13];
                    for profile in ["actionclaim", "round7:16:8"] {
                        for seed in 0..seeds {
                            let mut players: [support::Player; 4] = std::array::from_fn(|i| {
                                support::Player::new(
                                    profile,
                                    (id * 1000 + index as u64) * 64 + seed * 4 + i as u64,
                                )
                            });
                            let mut prefix = replay::Replay::new();
                            for e in &events[..index] {
                                prefix.apply(e, |own, state, event| {
                                    players[own.seat.idx()].on_event(own, state, event)
                                });
                            }
                            for (i, (choice, _)) in actions.iter().enumerate() {
                                money[i] += counterfactual::finish(
                                    prefix.clone(),
                                    players.clone(),
                                    *choice,
                                )[seat.idx()] as f32
                                    / (2 * seeds) as f32;
                            }
                        }
                    }
                    let mean = money[..actions.len()].iter().sum::<f32>() / actions.len() as f32;
                    for value in &mut money[..actions.len()] {
                        *value = (*value - mean) / 100.0;
                    }
                    let mut data = vec![0.0f32; HUMAN_ROW];
                    data[..6].copy_from_slice(&[
                        id as f32,
                        split as f32,
                        1.0,
                        hand as f32,
                        actions.iter().position(|a| a.0 == *card).unwrap() as f32,
                        actions.len() as f32,
                    ]);
                    data[6..6 + HUMAN_INPUTS].copy_from_slice(&x);
                    for (i, (card, a)) in actions.into_iter().enumerate() {
                        let offset = 6 + HUMAN_INPUTS + i * (1 + HUMAN_ACTIONS);
                        data[offset] = human_card_index(card) as f32;
                        data[offset + 1..offset + 1 + HUMAN_ACTIONS].copy_from_slice(&a);
                    }
                    for v in data {
                        out.write_all(&v.to_le_bytes()).unwrap();
                    }
                    for v in money {
                        targets.write_all(&v.to_le_bytes()).unwrap();
                    }
                    counts[split] += 1;
                    if counts.iter().sum::<usize>() % 100 == 0 {
                        eprintln!("{:?} positions by split", counts);
                    }
                }
            }
            replay.apply(event, |own, state, event| {
                memories[own.seat.idx()].on_event(own, state, event)
            });
        }
    }
    println!(
        "{}",
        serde_json::json!({"positions":counts,"responders":["actionclaim","round7:16:8"],"seeds_per_responder":seeds,"modulus":modulus,"target":"paired responsive money advantage / 100"})
    );
}
