//! Matched-budget hard-position and uniform-position search distillation.
use rand_chacha::ChaCha12Rng;
mod support;
use rand::{seq::SliceRandom, SeedableRng};
use std::{
    fs::File,
    io::{self, BufWriter, Write},
};
use support::Match;
use turbo_hearts_api::{BotState, ChargingRules, GameState, PassDirection, Seat};
use turbo_hearts_bot::{policy_features, NeuralNetworkBot, PolicyNetwork};

struct Position {
    search: NeuralNetworkBot,
    bot: BotState,
    state: GameState,
    difficulty: f32,
}

fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    assert!(
        args.len() >= 3,
        "hard_data HARD_OUTPUT UNIFORM_OUTPUT [HANDS=2000] [SEED=120000] [TEACHER_SAMPLES=256]"
    );
    let hands: u64 = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(2000);
    let seed: u64 = args.get(4).map(|s| s.parse().unwrap()).unwrap_or(120000);
    let samples = args.get(5).map(|s| s.parse().unwrap()).unwrap_or(256);
    assert!(seed + hands < 1 << 24);
    let mut writers = [
        BufWriter::new(File::create(&args[1])?),
        BufWriter::new(File::create(&args[2])?),
    ];
    let network = PolicyNetwork::load("assets/models/policy-moon.thp")?;
    let mut rng = ChaCha12Rng::seed_from_u64(seed ^ 0x48415244);
    let mut rows = [0; 2];
    let mut positions = 0;
    let mut overlaps = 0;
    let mut selected_difficulty = [0.0; 2];
    for deal in seed..seed + hands {
        let root = Seat::VALUES[(deal % 4) as usize];
        let mut kinds = ["heuristic"; 4];
        kinds[root.idx()] = "defend:32:12";
        kinds[root.across().idx()] = "duck";
        kinds[root.right().idx()] = "gottatry";
        let mut game = Match::new(
            deal,
            PassDirection::from(((deal / 4) % 4) as u8),
            kinds,
            ChargingRules::Classic,
        );
        let mut snapshots = Vec::new();
        while game.game.state.phase.is_playing() {
            let state = &game.game.state;
            let seat = state.next_actor.unwrap();
            let bot = &game.states[seat.idx()];
            let legal = state
                .legal_plays(bot.post_pass_hand)
                .distinct_plays(state.played, state.current_trick);
            if seat != root || legal.len() == 1 {
                game.play_one();
                continue;
            }
            // The snapshot contains the learner's redacted observations only.
            let search = game.players[root.idx()].search.as_ref().unwrap().clone();
            let mut values = game.players[root.idx()]
                .search
                .as_mut()
                .unwrap()
                .evaluate(bot, state);
            values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            let predicted = legal
                .into_iter()
                .max_by(|a, b| {
                    network
                        .predict(&policy_features(bot, state, *a))
                        .partial_cmp(&network.predict(&policy_features(bot, state, *b)))
                        .unwrap()
                })
                .unwrap();
            let gap = values[0].1 - values[1].1;
            let moon_points = state.won.hearts(root) + state.won.queen(root) as u8;
            let difficulty = 2.0 * (predicted != values[0].0) as u8 as f32
                + 1.0 / (1.0 + gap / 10.0)
                + if state.won.can_run(root) && moon_points > 0 {
                    1.0
                } else {
                    0.0
                };
            snapshots.push(Position {
                search,
                bot: bot.clone(),
                state: state.clone(),
                difficulty,
            });
            game.play_card(values[0].0);
        }
        let mut hard: Vec<_> = (0..snapshots.len()).collect();
        let mut uniform = hard.clone();
        hard.sort_by(|&a, &b| {
            snapshots[b]
                .difficulty
                .partial_cmp(&snapshots[a].difficulty)
                .unwrap()
        });
        uniform.shuffle(&mut rng);
        hard.truncate(4);
        uniform.truncate(4);
        positions += hard.len();
        overlaps += hard.iter().filter(|i| uniform.contains(i)).count();
        for (decision, mut position) in snapshots.into_iter().enumerate() {
            let selected = [hard.contains(&decision), uniform.contains(&decision)];
            if !selected[0] && !selected[1] {
                continue;
            }
            position.search = position.search.with_samples(samples);
            let values = position.search.evaluate(&position.bot, &position.state);
            let mean = values.iter().map(|v| v.1).sum::<f32>() / values.len() as f32;
            for dataset in 0..2 {
                if !selected[dataset] {
                    continue;
                }
                selected_difficulty[dataset] += position.difficulty as f64;
                for &(card, value) in &values {
                    for v in [deal as f32, decision as f32]
                        .iter()
                        .copied()
                        .chain(
                            policy_features(&position.bot, &position.state, card)
                                .iter()
                                .copied(),
                        )
                        .chain(std::iter::once((value - mean) / 100.0))
                    {
                        writers[dataset].write_all(&v.to_le_bytes())?;
                    }
                    rows[dataset] += 1;
                }
            }
        }
        if (deal - seed + 1) % 250 == 0 {
            eprintln!("{} hands, {:?} rows", deal - seed + 1, rows);
        }
    }
    for writer in &mut writers {
        writer.flush()?;
    }
    println!(
        "{}",
        serde_json::json!({"hands":hands,"seed":seed,"teacher_samples":samples,
        "student_samples":32,"positions_per_dataset":positions,"overlapping_positions":overlaps,
        "rows":rows,"mean_difficulty":selected_difficulty.map(|s|s/positions as f64)})
    );
    Ok(())
}
