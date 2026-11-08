//! Paired simulated games starting from historical post-pass hands and charges.
//! Recorded human actions are never reused after a counterfactual move.
#[path = "support/replay.rs"]
mod replay;
mod support;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};
use turbo_hearts_api::{GameEvent, Seat};
use turbo_hearts_bot::Algorithm;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(
        (7..=8).contains(&args.len()),
        "historical_arena GAMES.jsonl CANDIDATE BASELINE OPPONENT GAME_LIMIT SPLIT [successful]"
    );
    let successful = args.get(7).map_or(false, |s| {
        assert_eq!(s, "successful");
        true
    });
    let limit: usize = args[5].parse().unwrap();
    let split = args[6].parse::<u64>().unwrap();
    assert!(limit >= 2);
    let mut differences = Vec::new();
    let mut totals = [0.0; 2];
    let mut opponent_moons = [0; 2];
    let mut moons = [0; 2];
    let mut hands = 0;
    let mut seat_replays = 0;
    for line in BufReader::new(File::open(&args[1]).unwrap()).lines() {
        let row: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        if row["split"] != split {
            continue;
        }
        let seats: Vec<Seat> = Seat::VALUES
            .iter()
            .copied()
            .filter(|&seat| {
                !successful
                    || Seat::VALUES
                        .iter()
                        .any(|&other| other != seat && row["skilled"][other.idx()] == true)
            })
            .collect();
        if seats.is_empty() {
            continue;
        }
        let id = row["id"].as_u64().unwrap();
        let events: Vec<GameEvent> = serde_json::from_value(row["events"].clone()).unwrap();
        let mut replay = replay::Replay::new();
        let mut started = false;
        let mut hand = 0;
        let mut game_totals = [0.0; 2];
        for (index, event) in events.iter().enumerate() {
            if matches!(event, GameEvent::Deal { .. }) {
                started = false;
            }
            if matches!(event, GameEvent::Play { .. }) && !started {
                started = true;
                for (version, profile) in [&args[2], &args[3]].iter().enumerate() {
                    for &seat in &seats {
                        let mut players = std::array::from_fn(|i| {
                            support::Player::new(
                                if i == seat.idx() { profile } else { &args[4] },
                                id * 16 + hand * 4 + i as u64,
                            )
                        });
                        let mut fork = replay::Replay::new();
                        for event in &events[..index] {
                            fork.apply(event, |bot, state, event| {
                                players[bot.seat.idx()].on_event(bot, state, event)
                            });
                        }
                        let mut game = support::Match {
                            game: fork.game,
                            states: fork.states,
                            observations: fork.observations,
                            players,
                            play_micros: [0; 4],
                            preplay_micros: [0; 4],
                            root_evaluations: [0; 4],
                            max_play_micros: [0; 4],
                        };
                        let scores = game.finish();
                        game_totals[version] += scores.money(seat) as f64 / seats.len() as f64;
                        if version == 0 {
                            seat_replays += 1;
                        }
                        let won = game.game.state.won;
                        moons[version] += (won.queen(seat) && won.hearts(seat) == 13) as usize;
                        opponent_moons[version] += Seat::VALUES
                            .iter()
                            .any(|&s| s != seat && won.queen(s) && won.hearts(s) == 13)
                            as usize;
                    }
                }
                hand += 1;
                hands += 1;
            }
            replay.apply(event, |_, _, _| {});
        }
        assert_eq!(hand, 4, "complete recorded game must have four hands");
        for v in 0..2 {
            totals[v] += game_totals[v] / 4.0;
        }
        differences.push((game_totals[0] - game_totals[1]) / 4.0);
        if differences.len() % 10 == 0 {
            eprintln!("{}/{} games", differences.len(), limit);
        }
        if differences.len() >= limit {
            break;
        }
    }
    assert_eq!(
        differences.len(),
        limit,
        "requested more games than available"
    );
    let n = differences.len() as f64;
    let mean = differences.iter().sum::<f64>() / n;
    let ci =
        1.96 * (differences.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / (n - 1.0) / n).sqrt();
    println!(
        "{}",
        serde_json::json!({"candidate":args[2],"baseline":args[3],"opponent":args[4],
        "split":split,"games":n,"original_hands":hands,"hands_per_version":seat_replays,
        "cohort":if successful {"at least one other seat belongs to training-selected successful humans"} else {"all seats"},
        "money":totals.map(|v|v/n),"paired_difference":mean,"difference_ci95":[mean-ci,mean+ci],
        "moons":moons,"opponent_moons":opponent_moons,
        "interpretation":"simulated opponents on historical post-pass hands; confidence clustered by original game"})
    );
}
