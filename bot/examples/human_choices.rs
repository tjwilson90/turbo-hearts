//! Compare real successful-human choices with a bot, and branch both choices
//! under two responsive policies. Known full hands are diagnostic only.
#[path = "support/counterfactual.rs"]
mod counterfactual;
#[path = "support/replay.rs"]
mod replay;
mod support;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};
use turbo_hearts_api::{Card, Cards, GameEvent, GameState};
use turbo_hearts_bot::Algorithm;

fn equivalent(legal: Cards, state: &GameState, a: Card, b: Card) -> bool {
    let bridges = legal - Cards::NINES - Cards::CHARGEABLE - a - b;
    (Cards::from(a) | b)
        .distinct_plays(state.played | bridges, state.current_trick)
        .len()
        == 1
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(
        (7..=8).contains(&args.len()),
        "human_choices RANKED.jsonl OUTPUT.jsonl PROFILE SPLIT GAME_LIMIT SEEDS [RESPONDERS]"
    );
    let policies: Vec<_> = args
        .get(7)
        .map_or("skilledclaim,default", String::as_str)
        .split(',')
        .collect();
    let split: u64 = args[4].parse().unwrap();
    let limit: usize = args[5].parse().unwrap();
    let seeds: u64 = args[6].parse().unwrap();
    assert!(seeds > 0);
    let mut output = BufWriter::new(File::create(&args[2]).unwrap());
    let mut games = 0;
    let mut decisions = 0;
    for line in BufReader::new(File::open(&args[1]).unwrap()).lines() {
        let row: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        if row["split"] != split
            || !row["skilled"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| *v == true)
        {
            continue;
        }
        let id = row["id"].as_u64().unwrap();
        let events: Vec<GameEvent> = serde_json::from_value(row["events"].clone()).unwrap();
        let mut replay = replay::Replay::new();
        let mut players: [support::Player; 4] =
            std::array::from_fn(|i| support::Player::new(&args[3], id * 4 + i as u64));
        let mut hand = 0;
        for (index, event) in events.iter().enumerate() {
            if matches!(event, GameEvent::Deal { .. }) {
                hand += 1;
            }
            if let GameEvent::Play { seat, card } = event {
                let own = &replay.states[seat.idx()];
                let state = &replay.observations[seat.idx()];
                let legal = state.legal_plays(own.post_pass_hand);
                // Fixed outcome-independent sample of information sets.
                if row["skilled"][seat.idx()] == true
                    && legal
                        .distinct_plays(state.played, state.current_trick)
                        .len()
                        > 1
                    && (id * 1_000_003 + index as u64 * 37 + seat.idx() as u64) % 11 == 0
                {
                    let choice = players[seat.idx()].clone().play(own, state);
                    let same = equivalent(legal, state, *card, choice);
                    let mut continuations = Vec::new();
                    if !same {
                        for &profile in &policies {
                            let mut values = [0.0; 2];
                            for seed in 0..seeds {
                                let mut responders: [support::Player; 4] =
                                    std::array::from_fn(|i| {
                                        support::Player::new(
                                            profile,
                                            (id * 1000 + index as u64) * 64 + seed * 4 + i as u64,
                                        )
                                    });
                                let mut prefix = replay::Replay::new();
                                for event in &events[..index] {
                                    prefix.apply(event, |own, state, event| {
                                        responders[own.seat.idx()].on_event(own, state, event)
                                    });
                                }
                                for (version, played) in [*card, choice].iter().enumerate() {
                                    values[version] += counterfactual::finish(
                                        prefix.clone(),
                                        responders.clone(),
                                        *played,
                                    )[seat.idx()]
                                        as f64
                                        / seeds as f64;
                                }
                            }
                            continuations.push(serde_json::json!({"responder":profile,"human_money":values[0],"bot_money":values[1],"human_minus_bot":values[0]-values[1]}));
                        }
                    }
                    let mode = if state.current_trick.is_empty() {
                        "lead"
                    } else if legal.contains_any(state.current_trick.suit().cards()) {
                        "follow"
                    } else {
                        "slough"
                    };
                    writeln!(output,"{}",serde_json::json!({"game":id,"event":index,"hand":hand,"seat":seat,"profile":args[3],
                        "human":card,"bot":choice,"equivalent":same,"mode":mode,"own_can_run":state.won.can_run(*seat),
                        "played_count":state.played.len(),"remaining":(own.post_pass_hand-state.played).to_string(),
                        "trick":format!("{:?}",state.current_trick),"recorded_hand_money":row["hand_money"][hand-1][seat.idx()],
                        "continuations":continuations,"seeds":seeds})).unwrap();
                    decisions += 1;
                }
            }
            replay.apply(event, |own, state, event| {
                players[own.seat.idx()].on_event(own, state, event)
            });
        }
        games += 1;
        if games % 5 == 0 {
            eprintln!("{}/{} games; {} decisions", games, limit, decisions);
        }
        if games == limit {
            break;
        }
    }
    assert_eq!(games, limit);
    println!(
        "{}",
        serde_json::json!({"games":games,"sampled_decisions":decisions})
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn comparison_ignores_equivalent_ranks_but_preserves_nines_points_and_suits() {
        let state = GameState::new();
        let legal = "KJT9S QH".parse().unwrap();
        assert!(equivalent(legal, &state, Card::JackSpades, Card::TenSpades));
        assert!(!equivalent(
            legal,
            &state,
            Card::JackSpades,
            Card::NineSpades
        ));
        assert!(!equivalent(
            legal,
            &state,
            Card::KingSpades,
            Card::JackSpades
        ));
        assert!(!equivalent(
            legal,
            &state,
            Card::QueenHearts,
            Card::JackSpades
        ));
    }
}
