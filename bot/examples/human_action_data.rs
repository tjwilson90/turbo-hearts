#[path = "support/replay.rs"]
mod replay;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};
use turbo_hearts_api::GameEvent;
use turbo_hearts_bot::{human_action_observation, HumanPolicy, SkilledPolicy};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3, "human_action_data RANKED.jsonl OUTPUT.bin");
    let mut output = BufWriter::new(File::create(&args[2]).unwrap());
    let mut decision = 0;
    let mut count = 0;
    for line in BufReader::new(File::open(&args[1]).unwrap()).lines() {
        let row: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let mut replay = replay::Replay::new();
        for value in row["events"].as_array().unwrap() {
            let event: GameEvent = serde_json::from_value(value.clone()).unwrap();
            if let GameEvent::Play { seat, card } = event {
                let own = &replay.states[seat.idx()];
                let state = &replay.observations[seat.idx()];
                if row["humans"][seat.idx()] == true
                    && state.legal_plays(own.post_pass_hand).len() > 1
                {
                    let (context, actions) = human_action_observation(own, state);
                    let old = HumanPolicy::probability(own, state, card) as f32;
                    let skilled = SkilledPolicy::probability(own, state, card) as f32;
                    for (choice, features) in actions {
                        let data: Vec<f32> = [
                            row["id"].as_u64().unwrap() as f32,
                            row["split"].as_u64().unwrap() as f32,
                            (row["skilled"][seat.idx()] == true) as u8 as f32,
                            decision as f32,
                            (choice == card) as u8 as f32,
                            old,
                            skilled,
                        ]
                        .iter()
                        .copied()
                        .chain(context.iter().map(|v| *v as f32))
                        .chain(features.iter().map(|v| *v as f32))
                        .collect();
                        for value in data {
                            output.write_all(&value.to_le_bytes()).unwrap();
                        }
                        count += 1;
                    }
                    decision += 1;
                }
            }
            replay.apply(&event, |_, _, _| {});
        }
    }
    println!(
        "{}",
        serde_json::json!({"decisions":decision,"actions":count,"columns":49})
    );
}
