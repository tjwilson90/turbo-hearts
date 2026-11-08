//! Paired, responsive terminal-money labels for iterative preplay improvement.
mod support;
use rand::{seq::SliceRandom, RngExt, SeedableRng};
use rand_chacha::ChaCha12Rng;
use std::{
    fs::File,
    io::{BufWriter, Write},
};
use turbo_hearts_api::*;
use turbo_hearts_bot::*;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(
        args.len() >= 7,
        "preplay_data OUTPUT HANDS SEED LEARNER REPEATS PASS_CANDIDATES [ARCHIVED_PLAYERS ...]"
    );
    let count: u64 = args[2].parse().unwrap();
    let first: u64 = args[3].parse().unwrap();
    let repeats: u64 = args[5].parse().unwrap();
    let candidates: usize = args[6].parse().unwrap();
    assert!(first + count < 1 << 24 && repeats > 0 && (4..=32).contains(&candidates));
    let mut population = vec![
        "actionclaim",
        "skilledclaim",
        "round7:8:8",
        "heuristic",
        "gottatry",
    ];
    population.extend(args[7..].iter().map(String::as_str));
    let mut out = BufWriter::new(File::create(&args[1]).unwrap());
    let mut rng = ChaCha12Rng::seed_from_u64(first ^ 0x505245504c4159);
    let mut groups = [[0usize; 4]; 2];
    let mut rows = 0;
    let mut charge_stages = [0usize; 3];
    for deal in first..first + count {
        let hero = Seat::VALUES[(deal % 4) as usize];
        let direction = PassDirection::from((deal / 4 % 4) as u8);
        let rules = [
            ChargingRules::Classic,
            ChargingRules::Blind,
            ChargingRules::Bridge,
            ChargingRules::BlindBridge,
            ChargingRules::Chain,
            ChargingRules::BlindChain,
        ][(deal / 16 % 6) as usize];
        let mut kinds = std::array::from_fn(|_| population[rng.random_range(0..population.len())]);
        kinds[hero.idx()] = &args[4];
        let mut game = support::Match::dealt(deal, direction, kinds, rules);
        let mut decision = 0;
        let mut turns = 0;
        while !game.game.state.phase.is_playing() {
            if game.game.state.phase.is_passing() {
                let passes: [Cards; 4] = std::array::from_fn(|i| {
                    game.players[i].pass(&game.states[i], &game.observations[i])
                });
                let bot = &game.states[hero.idx()];
                let state = &game.observations[hero.idx()];
                let mut choices = vec![passes[hero.idx()]];
                for c in [
                    HeuristicBot.pass(bot, state),
                    GottaTryBot.pass(bot, state),
                    DuckBot.pass(bot, state),
                ] {
                    if !choices.contains(&c) {
                        choices.push(c);
                    }
                }
                let mut near = Vec::new();
                for out in passes[hero.idx()] {
                    for incoming in bot.pre_pass_hand - passes[hero.idx()] {
                        near.push((passes[hero.idx()] - out) | incoming);
                    }
                }
                near.shuffle(&mut rng);
                let mut all = preplay_actions(bot, state, PreplayKind::Pass);
                all.shuffle(&mut rng);
                for c in near.into_iter().take(candidates / 3).chain(all) {
                    if choices.len() >= candidates {
                        break;
                    }
                    if !choices.contains(&c) {
                        choices.push(c);
                    }
                }
                write_comparison(
                    &mut out,
                    &game,
                    deal,
                    decision,
                    hero,
                    PreplayKind::Pass,
                    &choices,
                    passes[hero.idx()],
                    Some(passes),
                    repeats,
                );
                rows += choices.len();
                groups[0][direction as usize] += 1;
                decision += 1;
                game.apply_passes(passes);
            } else {
                for i in 0..4 {
                    if !game.game.state.phase.is_charging() {
                        break;
                    }
                    let seat = Seat::VALUES[i];
                    if !game.game.state.can_charge(seat) || game.game.state.done.charged(seat) {
                        continue;
                    }
                    let baseline = game.players[i].charge(&game.states[i], &game.observations[i]);
                    if seat == hero {
                        let choices = preplay_actions(
                            &game.states[i],
                            &game.observations[i],
                            PreplayKind::Charge,
                        );
                        if choices.len() > 1 {
                            write_comparison(
                                &mut out,
                                &game,
                                deal,
                                decision,
                                hero,
                                PreplayKind::Charge,
                                &choices,
                                baseline,
                                None,
                                repeats,
                            );
                            rows += choices.len();
                            groups[1][direction as usize] += 1;
                            decision += 1;
                            charge_stages[match game.game.state.phase {
                                GamePhase::ChargeKeeper1 => 1,
                                GamePhase::ChargeKeeper2 => 2,
                                _ => 0,
                            }] += 1;
                        }
                    }
                    game.apply(GameEvent::Charge {
                        seat,
                        cards: baseline,
                    });
                    turns += 1;
                    assert!(turns <= 64);
                }
            }
        }
        if (deal - first + 1) % 100 == 0 {
            eprintln!(
                "{}/{} deals, {} candidate rows, {:?} groups",
                deal - first + 1,
                count,
                rows,
                groups
            );
        }
    }
    out.flush().unwrap();
    println!(
        "{}",
        serde_json::json!({"hands":count,"first_seed":first,"learner":args[4],"repeats":repeats,
        "pass_candidates":candidates,"population":population,"rows":rows,"groups_by_task_direction":groups,
        "charge_stages_normal_keeper1_keeper2":charge_stages,"row_floats":262,"target":"centered terminal money / 100"})
    );
}

fn write_comparison(
    out: &mut impl Write,
    game: &support::Match,
    deal: u64,
    decision: usize,
    hero: Seat,
    kind: PreplayKind,
    choices: &[Cards],
    baseline: Cards,
    passes: Option<[Cards; 4]>,
    repeats: u64,
) {
    assert!(choices.contains(&baseline));
    let mut values = vec![0.0f32; choices.len()];
    for repeat in 0..repeats {
        for (index, &action) in choices.iter().enumerate() {
            let mut fork = game.clone();
            fork.reseed_players((deal * 128 + decision as u64) * 16 + repeat);
            if let Some(mut passes) = passes {
                passes[hero.idx()] = action;
                fork.apply_passes(passes);
            } else {
                fork.apply(GameEvent::Charge {
                    seat: hero,
                    cards: action,
                });
                fork.charge_from(hero.idx() + 1);
            }
            fork.finish_preplay();
            values[index] += fork.finish().money(hero) as f32 / repeats as f32;
        }
    }
    let mean = values.iter().sum::<f32>() / values.len() as f32;
    for (&action, value) in choices.iter().zip(values) {
        let features = preplay_features(
            &game.states[hero.idx()],
            &game.observations[hero.idx()],
            action,
            kind,
        );
        for v in [
            deal as f32,
            decision as f32,
            kind as u8 as f32,
            game.game.state.phase.direction() as u8 as f32,
            (action == baseline) as u8 as f32,
            (value - mean) / 100.0,
        ]
        .iter()
        .chain(features.iter())
        {
            out.write_all(&v.to_le_bytes()).unwrap();
        }
    }
}
