//! Build observer-only questions BEFORE recorded actions, without future outcomes.
#[path = "support/replay.rs"]
mod replay;
mod support;
use rand::{seq::SliceRandom, SeedableRng};
use rand_chacha::ChaCha12Rng;
use serde_json::{json, Value};
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};
use turbo_hearts_api::*;
use turbo_hearts_bot::*;

#[derive(Clone)]
struct Position {
    index: usize,
    hand_number: usize,
    kind: usize,
    bot: BotState,
    state: GameState,
    history: Vec<GameEvent>,
    player: support::Player,
    previous_money: [i32; 4],
}

fn relevant(event: &GameEvent) -> bool {
    matches!(
        event,
        GameEvent::Deal { .. }
            | GameEvent::SendPass { .. }
            | GameEvent::RecvPass { .. }
            | GameEvent::HiddenSendPass { .. }
            | GameEvent::HiddenRecvPass { .. }
            | GameEvent::Charge { .. }
            | GameEvent::BlindCharge { .. }
            | GameEvent::RevealCharges { .. }
            | GameEvent::Play { .. }
            | GameEvent::StartTrick { .. }
            | GameEvent::EndTrick { .. }
            | GameEvent::Claim { .. }
            | GameEvent::AcceptClaim { .. }
            | GameEvent::RejectClaim { .. }
            | GameEvent::StartPassing
            | GameEvent::StartCharging
    )
}

fn question(
    p: Position,
    group: &str,
    split: u64,
    humans: &[bool; 4],
    pass: &PreplayNetwork,
    charge: &PreplayNetwork,
    rng: &mut ChaCha12Rng,
) -> Value {
    let b = &p.bot;
    let s = &p.state;
    let hand = if p.kind == 0 {
        b.pre_pass_hand
    } else {
        b.post_pass_hand - s.played
    };
    let legal = match p.kind {
        0 => hand,
        1 => (hand & Cards::CHARGEABLE) - s.charges.all_charges(),
        _ => s.legal_plays(b.post_pass_hand),
    };
    let mut player = p.player.clone();
    let choices: Vec<Cards> = match p.kind {
        0 => {
            let base = HeuristicBot.pass(b, s);
            vec![base, pass.choose(b, s, base, 0.0)]
        }
        1 => {
            let base = HeuristicBot.charge_with_rng(b, s, rng);
            vec![base, charge.choose(b, s, base, 0.0)]
        }
        _ => vec![
            player.play(b, s).into(),
            HeuristicBot.play_with_rng(b, s, rng).into(),
            HumanActionPolicy::play_with_rng(b, s, rng).into(),
        ],
    };
    let disagreement = choices.iter().any(|c| *c != choices[0]);
    let current_start = p
        .history
        .iter()
        .rposition(|e| matches!(e, GameEvent::EndTrick { .. }))
        .map_or(0, |i| i + 1);
    let trick: Vec<_> = p.history[current_start..]
        .iter()
        .filter_map(|e| match e {
            GameEvent::Play { seat, card } => Some(json!({"seat":seat,"card":card})),
            _ => None,
        })
        .collect();
    assert_eq!(trick.len(), s.current_trick.len());
    let players: Vec<_> = Seat::VALUES.iter().map(|seat| json!({
        "seat":seat,"human":humans[seat.idx()],
        "charges":s.charges.charges(*seat),"declined":s.done.charged(*seat),
        "hearts":s.won.hearts(*seat),"queen":s.won.queen(*seat),
        "jack":s.won.jack(*seat),"ten":s.won.ten(*seat),
        "voids":Suit::VALUES.iter().filter(|suit| b.void.is_void(*seat, **suit)).map(|suit| format!("{:?}",suit)).collect::<Vec<_>>(),
        "previous_money":p.previous_money[seat.idx()]
    })).collect();
    let kind = ["pass", "charge", "play"][p.kind];
    let phase = format!("{:?}", s.phase);
    let view = json!({"kind":kind,"seat":b.seat,"hand_number":p.hand_number,
        "rules":s.rules,"direction":s.phase.direction(),"phase":phase,
        "receiver":s.phase.pass_receiver(b.seat),"sender":s.phase.pass_sender(b.seat),
        "hand":hand,"original_hand":b.pre_pass_hand,"legal_cards":legal,
        "min_cards":if p.kind==1 {0} else if p.kind==0 {3} else {1},
        "max_cards":if p.kind==1 {legal.len()} else if p.kind==0 {3} else {1},
        "players":players,"trick":trick,"played":s.played,"charge_count":s.charge_count,
        "history":p.history});
    // GameState's normal serialization omits phase/rules/done/charge_count.
    // Preserve these explicitly, alongside only this actor's cards and history.
    let context = json!({"state":s,"phase":phase,"rules":s.rules,
        "charge_count":s.charge_count,"declined":Seat::VALUES.map(|seat| s.done.charged(seat)),
        "sent_pass":Seat::VALUES.map(|seat| s.done.sent_pass(seat)),
        "pre_pass_hand":b.pre_pass_hand,"post_pass_hand":b.post_pass_hand,
        "voids":Seat::VALUES.map(|seat| Suit::VALUES.map(|suit| b.void.is_void(seat,suit)))});
    json!({"schema":"turbo-hearts-judgment-v1",
        "id":format!("{}:{}:{}:{}",group,p.index,b.seat.idx(),kind),
        "group_id":group,"source_split":split,"event_index":p.index,
        "view":view,"context":context,
        "selection":{"disagreement":disagreement,"policy_choices":choices,
            "policies":if p.kind==2 {vec!["round7:16:8","heuristic","human_action"]}
                else if p.kind==0 {vec!["heuristic","pass2"]} else {vec!["heuristic","charge2"]}}})
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(
        args.len() >= 4,
        "judgment_cases GAMES.jsonl OUTPUT.jsonl PER_KIND_PER_GAME [SEED]"
    );
    let quota: usize = args[3].parse().unwrap();
    assert!((1..=100).contains(&quota));
    let seed: u64 = args.get(4).map_or(42, |v| v.parse().unwrap());
    let pass = PreplayNetwork::from_bytes(include_bytes!("../../assets/models/preplay-pass2.tpq"))
        .unwrap();
    let charge =
        PreplayNetwork::from_bytes(include_bytes!("../../assets/models/preplay-charge2.tpq"))
            .unwrap();
    let mut out = BufWriter::new(File::create(&args[2]).unwrap());
    let mut counts = [0usize; 3];
    let mut games = 0;
    for line in BufReader::new(File::open(&args[1]).unwrap()).lines() {
        let row: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let group = row["group_id"].as_str().expect("anonymous group ID");
        let events: Vec<GameEvent> = serde_json::from_value(row["events"].clone()).unwrap();
        let humans = std::array::from_fn(|i| row["humans"][i].as_bool().unwrap());
        let mixed_seed = group
            .bytes()
            .fold(seed, |v, b| v.wrapping_mul(131).wrapping_add(b as u64));
        let mut rng = ChaCha12Rng::seed_from_u64(mixed_seed);
        let mut replay = replay::Replay::new();
        let mut histories: [Vec<GameEvent>; 4] = std::array::from_fn(|_| Vec::new());
        let mut players = std::array::from_fn::<_, 4, _>(|i| {
            support::Player::new("round7:16:8", mixed_seed.wrapping_add(i as u64))
        });
        let mut pools: [Vec<Position>; 3] = std::array::from_fn(|_| Vec::new());
        let mut hand_number = 0;
        let mut previous_money = [0i32; 4];
        for (index, event) in events.iter().enumerate() {
            if matches!(event, GameEvent::Deal { .. }) {
                hand_number += 1;
            }
            let decision = match event {
                GameEvent::SendPass { from, .. } => Some((0, *from)),
                GameEvent::Charge { seat, .. } => Some((1, *seat)),
                GameEvent::Play { seat, .. } => Some((2, *seat)),
                _ => None,
            };
            if let Some((kind, seat)) = decision {
                let b = &replay.states[seat.idx()];
                let s = &replay.observations[seat.idx()];
                let eligible = match kind {
                    0 => s.phase.is_passing() && b.pre_pass_hand.len() == 13,
                    1 => {
                        s.phase.is_charging()
                            && s.can_charge(seat)
                            && !s.done.charged(seat)
                            && !((b.post_pass_hand & Cards::CHARGEABLE) - s.charges.all_charges())
                                .is_empty()
                    }
                    _ => {
                        s.legal_plays(b.post_pass_hand)
                            .distinct_plays(s.played, s.current_trick)
                            .len()
                            > 1
                            && !should_claim(s, b.void, seat, b.post_pass_hand)
                    }
                };
                if eligible {
                    pools[kind].push(Position {
                        index,
                        hand_number,
                        kind,
                        bot: b.clone(),
                        state: s.clone(),
                        history: histories[seat.idx()].clone(),
                        player: players[seat.idx()].clone(),
                        previous_money,
                    });
                }
            }
            replay.apply(event, |b, s, e| {
                let i = b.seat.idx();
                if matches!(e, GameEvent::Deal { .. }) {
                    histories[i].clear();
                }
                if relevant(e) {
                    histories[i].push(e.clone());
                }
                players[i].on_event(b, s, e);
                if i == 0 {
                    if let GameEvent::HandComplete {
                        north_score,
                        east_score,
                        south_score,
                        west_score,
                    } = e
                    {
                        let scores =
                            Scores::new([*north_score, *east_score, *south_score, *west_score]);
                        for seat in Seat::VALUES {
                            previous_money[seat.idx()] += scores.money(seat) as i32;
                        }
                    }
                }
            });
        }
        for kind in 0..3 {
            pools[kind].shuffle(&mut rng);
            for p in pools[kind].drain(..).take(quota) {
                let case = question(
                    p,
                    group,
                    row["split"].as_u64().unwrap(),
                    &humans,
                    &pass,
                    &charge,
                    &mut rng,
                );
                writeln!(out, "{}", case).unwrap();
                counts[kind] += 1;
            }
        }
        games += 1;
    }
    out.flush().unwrap();
    println!(
        "{}",
        json!({"games":games,"pass_charge_play":counts,"seed":seed})
    );
}
