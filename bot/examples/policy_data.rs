mod support;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{
    fs::File,
    io::{self, BufWriter, Write},
};
use support::Match;
use turbo_hearts_api::{ChargingRules, PassDirection};
use turbo_hearts_bot::policy_features;

// Rows: deal id, decision id, 256 features, advantage/100, all little-endian f32.
// Split by DEAL, never by row: candidates and positions from one hand correlate.
fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let path = args
        .get(1)
        .expect("policy_data OUTPUT [HANDS] [SEED] [SAMPLES] [STRATEGY]");
    let hands: u64 = args.get(2).map(|v| v.parse().unwrap()).unwrap_or(2000);
    let seed: u64 = args.get(3).map(|v| v.parse().unwrap()).unwrap_or(10000);
    let samples: usize = args.get(4).map(|v| v.parse().unwrap()).unwrap_or(64);
    assert!(
        seed + hands < (1 << 24),
        "deal ids must be exactly representable in float32"
    );
    let mut writer = BufWriter::new(File::create(path)?);
    let strategy = args.get(5).map(String::as_str).unwrap_or("defend");
    let kind = format!("{}:{}:12", strategy, samples);
    let mut rows = 0;
    let mut rng = StdRng::seed_from_u64(seed);
    for deal in seed..seed + hands {
        let mut game = Match::new(
            deal,
            PassDirection::from((deal % 4) as u8),
            [&kind; 4],
            ChargingRules::Classic,
        );
        let mut decision = 0;
        while game.game.state.phase.is_playing() {
            let s = game.game.state.next_actor.unwrap().idx();
            let state = &game.game.state;
            let bot = &game.states[s];
            let distinct = state
                .legal_plays(bot.post_pass_hand)
                .distinct_plays(state.played, state.current_trick);
            if distinct.len() == 1 {
                game.play_card(distinct.max());
                continue;
            }
            let values = game.players[s]
                .search
                .as_mut()
                .unwrap()
                .evaluate(bot, state);
            let mean = values.iter().map(|v| v.1).sum::<f32>() / values.len() as f32;
            for &(card, value) in &values {
                for v in [deal as f32, decision as f32]
                    .iter()
                    .copied()
                    .chain(policy_features(bot, state, card).iter().copied())
                    .chain(std::iter::once((value - mean) / 100.0))
                {
                    writer.write_all(&v.to_le_bytes())?;
                }
                rows += 1;
            }
            let card = if rng.gen_bool(0.15) {
                values[rng.gen_range(0..values.len())].0
            } else {
                values
                    .iter()
                    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    .unwrap()
                    .0
            };
            game.play_card(card);
            decision += 1;
        }
        if (deal - seed + 1) % 250 == 0 {
            eprintln!("{} hands, {} action rows", deal - seed + 1, rows);
        }
    }
    writer.flush()?;
    eprintln!("Wrote {} rows to {}", rows, path);
    Ok(())
}
