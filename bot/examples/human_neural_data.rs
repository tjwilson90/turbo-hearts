//! Export only actual actor observations; complete games retain their split.
#[path = "support/replay.rs"]
mod replay;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};
use turbo_hearts_api::GameEvent;
use turbo_hearts_bot::{
    human_card_index, neural_human_observation, HumanMemory, HUMAN_ACTIONS, HUMAN_INPUTS, HUMAN_ROW,
};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3, "human_neural_data RANKED.jsonl OUTPUT.bin");
    let mut out = BufWriter::new(File::create(&args[2]).unwrap());
    let mut counts = [0; 4];
    for line in BufReader::new(File::open(&args[1]).unwrap()).lines() {
        let row: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let mut replay = replay::Replay::new();
        let mut memories: [HumanMemory; 4] = std::array::from_fn(|_| HumanMemory::default());
        let mut hand = 0;
        for value in row["events"].as_array().unwrap() {
            let event: GameEvent = serde_json::from_value(value.clone()).unwrap();
            if matches!(event, GameEvent::Deal { .. }) {
                hand += 1;
            }
            if let GameEvent::Play { seat, card } = event {
                let own = &replay.states[seat.idx()];
                let state = &replay.observations[seat.idx()];
                if row["skilled"][seat.idx()] == true
                    && state.legal_plays(own.post_pass_hand).len() > 1
                {
                    let (x, actions) = neural_human_observation(own, state, &memories[seat.idx()]);
                    let mut data = vec![0.0f32; HUMAN_ROW];
                    let split = row["split"].as_u64().unwrap() as usize;
                    data[..6].copy_from_slice(&[
                        row["id"].as_u64().unwrap() as f32,
                        split as f32,
                        1.0,
                        hand as f32,
                        actions.iter().position(|a| a.0 == card).unwrap() as f32,
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
                    counts[split] += 1;
                }
            }
            replay.apply(&event, |own, state, event| {
                memories[own.seat.idx()].on_event(own, state, event)
            });
        }
    }
    println!(
        "{}",
        serde_json::json!({"rows_by_split":counts,"columns":HUMAN_ROW})
    );
}
