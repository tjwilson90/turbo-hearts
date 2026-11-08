//! Recover exact historical payoffs using the game engine, including claims.
#[path = "support/replay.rs"]
mod replay;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};
use turbo_hearts_api::{GameEvent, Scores, Seat};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3, "corpus_scores INPUT.jsonl OUTPUT.jsonl");
    let mut output = BufWriter::new(File::create(&args[2]).unwrap());
    let mut count = 0;
    for line in BufReader::new(File::open(&args[1]).unwrap()).lines() {
        let mut row: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let events: Vec<GameEvent> = serde_json::from_value(row["events"].clone()).unwrap();
        let mut replay = replay::Replay::new();
        let mut money = Vec::new();
        for event in events {
            replay.apply(&event, |own, _, event| {
                if own.seat == Seat::North {
                    if let GameEvent::HandComplete {
                        north_score,
                        east_score,
                        south_score,
                        west_score,
                    } = event
                    {
                        let scores =
                            Scores::new([*north_score, *east_score, *south_score, *west_score]);
                        money.push(Seat::VALUES.map(|s| scores.money(s)));
                    }
                }
            });
        }
        assert_eq!(money.len(), 4);
        row["hand_money"] = serde_json::json!(money);
        writeln!(output, "{}", row).unwrap();
        count += 1;
    }
    println!("{} games scored", count);
}
