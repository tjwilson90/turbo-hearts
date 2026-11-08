//! Counterfactual continuation from a recorded position. Opponents respond
//! through policies after the branch; their recorded future moves are not used.
#[path = "support/replay.rs"]
mod replay;
mod support;
use std::{collections::BTreeMap, fs};
use turbo_hearts_api::{should_claim, Card, GameEvent, Seat};
use turbo_hearts_bot::Algorithm;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(
        args.len(),
        7,
        "branch_game EVENTS.json EVENT_INDEX BOT_PROFILE HUMAN_PROFILE CARD_OR_auto SEEDS"
    );
    let events: Vec<GameEvent> =
        serde_json::from_str(&fs::read_to_string(&args[1]).unwrap()).unwrap();
    let index: usize = args[2].parse().unwrap();
    let seeds: u64 = args[6].parse().unwrap();
    assert!(seeds > 0, "SEEDS must be positive");
    let forced: Option<Card> = if args[5] == "auto" {
        None
    } else {
        Some(args[5].parse().unwrap())
    };
    let bots = match &events[0] {
        GameEvent::Sit {
            north,
            east,
            south,
            west,
            ..
        } => [north, east, south, west].map(|p| p.strategy().is_some()),
        _ => panic!("first event must be Sit"),
    };
    assert!(matches!(events[index], GameEvent::Play { .. }));
    let mut scores = [0.0; 4];
    let mut money = [0.0; 4];
    let mut moons = [0usize; 4];
    let mut first = BTreeMap::new();
    let mut samples = Vec::new();
    let mut trace = Vec::new();
    for seed in 0..seeds {
        let mut players: [support::Player; 4] = std::array::from_fn(|i| {
            support::Player::new(
                if bots[i] { &args[3] } else { &args[4] },
                seed * 4 + i as u64,
            )
        });
        let mut replay = replay::Replay::new();
        for event in &events[..index] {
            replay.apply(event, |bot, state, event| {
                players[bot.seat.idx()].on_event(bot, state, event)
            });
        }
        let seat = replay.game.state.next_actor.unwrap();
        let card = forced.unwrap_or_else(|| {
            players[seat.idx()].play(&replay.states[seat.idx()], &replay.observations[seat.idx()])
        });
        *first.entry(card.to_string()).or_insert(0) += 1;
        replay.apply(&GameEvent::Play { seat, card }, |bot, state, event| {
            players[bot.seat.idx()].on_event(bot, state, event)
        });
        while replay.game.state.phase.is_playing() {
            let seat = replay.game.state.next_actor.unwrap();
            let own = &replay.states[seat.idx()];
            let state = &replay.observations[seat.idx()];
            if should_claim(state, own.void, seat, own.post_pass_hand) {
                let event = GameEvent::Claim {
                    seat,
                    hand: own.post_pass_hand - state.played,
                };
                replay.apply(&event, |bot, state, event| {
                    players[bot.seat.idx()].on_event(bot, state, event)
                });
                for acceptor in Seat::VALUES {
                    if acceptor != seat {
                        replay.apply(
                            &GameEvent::AcceptClaim {
                                claimer: seat,
                                acceptor,
                            },
                            |bot, state, event| players[bot.seat.idx()].on_event(bot, state, event),
                        );
                    }
                }
            } else {
                let card = players[seat.idx()].play(own, state);
                replay.apply(&GameEvent::Play { seat, card }, |bot, state, event| {
                    players[bot.seat.idx()].on_event(bot, state, event)
                });
            }
        }
        let result = replay.game.state.scores();
        if seed == 0 {
            trace = replay.game.events[index..].to_vec();
        }
        for s in Seat::VALUES {
            scores[s.idx()] += result.score(s) as f64 / seeds as f64;
            money[s.idx()] += result.money(s) as f64 / seeds as f64;
            moons[s.idx()] +=
                (replay.game.state.won.queen(s) && replay.game.state.won.hearts(s) == 13) as usize;
        }
        samples.push(Seat::VALUES.map(|s| result.money(s)));
    }
    println!(
        "{}",
        serde_json::json!({"event_index":index,"bot":args[3],"human_model":args[4],
        "forced":args[5],"seeds":seeds,"first_choices":first,"mean_scores":scores,"mean_money":money,
        "moons":moons,"money_samples":samples,"first_seed_continuation":trace,
        "interpretation":"simulated adaptive continuations on actual hands, not observed human counterfactual outcomes"})
    );
}
