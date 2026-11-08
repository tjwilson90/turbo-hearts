#[path = "support/replay.rs"]
mod replay;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};
use turbo_hearts_api::GameEvent;
use turbo_hearts_bot::{human_context, human_likelihoods};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3, "human_data GAMES.jsonl OUTPUT.bin");
    let mut output = BufWriter::new(File::create(&args[2]).unwrap());
    let mut counts = [0usize; 3];
    for line in BufReader::new(File::open(&args[1]).unwrap()).lines() {
        let row: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let id = row["id"].as_u64().unwrap() as f32;
        let split = row["split"].as_u64().unwrap() as usize;
        let mut replay = replay::Replay::new();
        for value in row["events"].as_array().unwrap() {
            let event: GameEvent = serde_json::from_value(value.clone()).unwrap();
            if let GameEvent::Play { seat, card } = event {
                let bot = &replay.states[seat.idx()];
                let state = &replay.observations[seat.idx()];
                if row["humans"][seat.idx()] == true
                    && state.legal_plays(bot.post_pass_hand).len() > 1
                {
                    let data = vec![id, split as f32]
                        .into_iter()
                        .chain(
                            human_context(bot, state)
                                .iter()
                                .map(|v| *v as f32)
                                .collect::<Vec<_>>(),
                        )
                        .chain(
                            human_likelihoods(bot, state, card)
                                .iter()
                                .map(|v| *v as f32)
                                .collect::<Vec<_>>(),
                        );
                    for value in data {
                        output.write_all(&value.to_le_bytes()).unwrap();
                    }
                    counts[split] += 1;
                }
            }
            replay.apply(&event, |_, _, _| {});
        }
    }
    output.flush().unwrap();
    println!(
        "{}",
        serde_json::json!({"decisions_by_split":counts,"columns":15})
    );
}
