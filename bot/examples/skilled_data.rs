#[path = "support/replay.rs"]
mod replay;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};
use turbo_hearts_api::GameEvent;
use turbo_hearts_bot::{skilled_context, skilled_likelihoods, HumanPolicy};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3, "skilled_data RANKED.jsonl OUTPUT.bin");
    let mut output = BufWriter::new(File::create(&args[2]).unwrap());
    let mut counts = [[0; 2]; 4];
    for line in BufReader::new(File::open(&args[1]).unwrap()).lines() {
        let row: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let id = row["id"].as_u64().unwrap() as f32;
        let split = row["split"].as_u64().unwrap() as usize;
        let mut replay = replay::Replay::new();
        for value in row["events"].as_array().unwrap() {
            let event: GameEvent = serde_json::from_value(value.clone()).unwrap();
            if let GameEvent::Play { seat, card } = event {
                let own = &replay.states[seat.idx()];
                let state = &replay.observations[seat.idx()];
                if row["humans"][seat.idx()] == true
                    && state.legal_plays(own.post_pass_hand).len() > 1
                {
                    let skilled = (row["skilled"][seat.idx()] == true) as usize;
                    let data: Vec<f32> = [id, split as f32, skilled as f32]
                        .iter()
                        .copied()
                        .chain(skilled_context(own, state).iter().map(|v| *v as f32))
                        .chain(
                            skilled_likelihoods(own, state, card)
                                .iter()
                                .map(|v| *v as f32),
                        )
                        .chain(std::iter::once(
                            HumanPolicy::probability(own, state, card) as f32
                        ))
                        .collect();
                    for value in data {
                        output.write_all(&value.to_le_bytes()).unwrap();
                    }
                    counts[split][skilled] += 1;
                }
            }
            replay.apply(&event, |_, _, _| {});
        }
    }
    println!(
        "{}",
        serde_json::json!({"counts_by_split_other_selected":counts,"columns":26})
    );
}
