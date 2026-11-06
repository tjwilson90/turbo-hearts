//! On-policy trajectories with terminal-money rewards and a paired greedy
//! reference baseline. Opponents are sampled from a population, including
//! archived policies, rather than supplied with the learner's private cards.
use rand_chacha::ChaCha12Rng;
mod support;
use rand::{RngExt, SeedableRng};
use std::{
    fs::File,
    io::{self, BufWriter, Write},
};
use support::Match;
use turbo_hearts_api::{ChargingRules, PassDirection, Seat};
use turbo_hearts_bot::{policy_features, Algorithm};

fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    assert!(
        args.len() >= 6,
        "self_play OUTPUT MODEL HANDS SEED TEMPERATURE [ARCHIVE ...]"
    );
    let mut out = BufWriter::new(File::create(&args[1])?);
    let hands: u64 = args[3].parse().unwrap();
    let seed: u64 = args[4].parse().unwrap();
    assert!(seed + hands < 1 << 24);
    let learner = format!("policy:{}:{}", args[2], args[5]);
    let reference = format!("policy:{}:0", args[2]);
    let mut population = vec![
        "heuristic".to_owned(),
        "duck".to_owned(),
        "gottatry".to_owned(),
        "defend:16:12".to_owned(),
    ];
    for archive in &args[6..] {
        population.push(format!("policy:{}:0", archive));
    }
    let mut rng = ChaCha12Rng::seed_from_u64(seed ^ 0x504f50);
    let mut scores = [0.0; 2];
    let mut rows = 0;
    for deal in seed..seed + hands {
        let hero = Seat::VALUES[(deal % 4) as usize];
        let mut kinds: [&str; 4] =
            std::array::from_fn(|_| population[rng.random_range(0..population.len())].as_str());
        kinds[hero.idx()] = &reference;
        let direction = PassDirection::from((deal / 4 % 4) as u8);
        let mut control = Match::new(deal, direction, kinds, ChargingRules::Classic);
        let baseline = control.finish().money(hero) as f32;
        kinds[hero.idx()] = &learner;
        let mut game = Match::new(deal, direction, kinds, ChargingRules::Classic);
        let mut examples = Vec::new();
        while game.game.state.phase.is_playing() {
            let seat = game.game.state.next_actor.unwrap();
            if seat != hero {
                game.play_one();
                continue;
            }
            let bot = &game.states[hero.idx()];
            let state = &game.game.state;
            let distribution = game.players[hero.idx()].policy_distribution(bot, state);
            let card = game.players[hero.idx()].play(bot, state);
            if distribution.len() > 1 {
                let decision = state.played.len() as f32;
                for (candidate, probability) in distribution {
                    examples.push((
                        decision,
                        (candidate == card) as u8 as f32,
                        probability as f32,
                        policy_features(bot, state, candidate),
                    ));
                }
            }
            game.play_card(card);
        }
        let money = game.game.state.scores().money(hero) as f32;
        scores[0] += money as f64;
        scores[1] += baseline as f64;
        for (decision, chosen, probability, features) in examples {
            // deal, decision, chosen, behavior probability, paired terminal
            // advantage /100, then the 256 Rust-encoded features.
            for v in [
                deal as f32,
                decision,
                chosen,
                probability,
                (money - baseline) / 100.0,
            ]
            .iter()
            .chain(features.iter())
            {
                out.write_all(&v.to_le_bytes())?;
            }
            rows += 1;
        }
        if (deal - seed + 1) % 500 == 0 {
            eprintln!(
                "{} hands, learner {:.2}, greedy control {:.2}",
                deal - seed + 1,
                scores[0] / (deal - seed + 1) as f64,
                scores[1] / (deal - seed + 1) as f64
            );
        }
    }
    out.flush()?;
    println!(
        "{}",
        serde_json::json!({"hands":hands,"seed":seed,"rows":rows,"money":scores[0]/hands as f64,"greedy_reference_money":scores[1]/hands as f64,"population":population})
    );
    Ok(())
}
