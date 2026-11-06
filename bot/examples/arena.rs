mod support;
use support::Match;
use turbo_hearts_api::{ChargingRules, PassDirection, Seat};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("arena [CANDIDATE] [BASELINE] [OPPONENT] [DEALS] [SEED] [RULES]\n\
Players: default, heuristic, duck, gottatry, random, search:SAMPLES:EXACT, moon:SAMPLES:EXACT, defend:SAMPLES:EXACT.\n\
Append :MODEL:PRIOR_SAMPLES to a search player to test a learned prior.\n\
OPPONENT also accepts mixed (heuristic, duck, gottatry). Each deal is replayed in all four seats.\n\
Rules: classic, blind, bridge, blind_bridge, chain, blind_chain.");
        return;
    }
    let candidate = args.get(1).map(String::as_str).unwrap_or("default");
    let baseline = args.get(2).map(String::as_str).unwrap_or("heuristic");
    let opponent = args.get(3).map(String::as_str).unwrap_or("heuristic");
    let deals: usize = args.get(4).map(|v| v.parse().unwrap()).unwrap_or(100);
    let seed: u64 = args.get(5).map(|v| v.parse().unwrap()).unwrap_or(0);
    assert!(deals >= 2);
    let rules_name = args.get(6).map(String::as_str).unwrap_or("classic");
    let rules = match rules_name {
        "classic" => ChargingRules::Classic,
        "blind" => ChargingRules::Blind,
        "bridge" => ChargingRules::Bridge,
        "blind_bridge" => ChargingRules::BlindBridge,
        "chain" => ChargingRules::Chain,
        "blind_chain" => ChargingRules::BlindChain,
        _ => panic!("unknown rules"),
    };
    let mut sums = [0.0; 2];
    let mut differences = Vec::new();
    let mut wins = [0.0; 2];
    let mut moons = [0; 2];
    let mut opponent_moons = [0; 2];
    let mut near_moons = [0; 2];
    let mut micros = [0u128; 2];
    let mut max_micros = [0u128; 2];
    for deal in seed..seed + deals as u64 {
        let mut means = [0.0; 2];
        for (version, kind) in [candidate, baseline].iter().enumerate() {
            for &seat in &Seat::VALUES {
                let mut kinds = [opponent; 4];
                if opponent == "mixed" {
                    kinds[seat.left().idx()] = "heuristic";
                    kinds[seat.across().idx()] = "duck";
                    kinds[seat.right().idx()] = "gottatry";
                }
                kinds[seat.idx()] = kind;
                let mut game =
                    Match::new(deal, PassDirection::from((deal % 4) as u8), kinds, rules);
                let scores = game.finish();
                let won = game.game.state.won;
                if won.queen(seat) && won.hearts(seat) == 13 {
                    moons[version] += 1;
                }
                if won.queen(seat) && (8..13).contains(&won.hearts(seat)) {
                    near_moons[version] += 1;
                }
                if Seat::VALUES
                    .iter()
                    .any(|s| *s != seat && won.queen(*s) && won.hearts(*s) == 13)
                {
                    opponent_moons[version] += 1;
                }
                let money = scores.money(seat) as f64;
                means[version] += money / 4.0;
                let best = Seat::VALUES.iter().map(|s| scores.money(*s)).max().unwrap();
                if scores.money(seat) == best {
                    wins[version] += 1.0
                        / Seat::VALUES
                            .iter()
                            .filter(|s| scores.money(**s) == best)
                            .count() as f64;
                }
                micros[version] += game.play_micros[seat.idx()];
                max_micros[version] = max_micros[version].max(game.max_play_micros[seat.idx()]);
            }
            sums[version] += means[version];
        }
        differences.push(means[0] - means[1]);
        if (deal - seed + 1) % 50 == 0 {
            eprintln!(
                "{}/{} deals: candidate {:.2}, baseline {:.2}",
                deal - seed + 1,
                deals,
                sums[0] / differences.len() as f64,
                sums[1] / differences.len() as f64
            );
        }
    }
    let n = deals as f64;
    let difference = differences.iter().sum::<f64>() / n;
    let ci = 1.96
        * (differences
            .iter()
            .map(|v| (v - difference).powi(2))
            .sum::<f64>()
            / (n - 1.0)
            / n)
            .sqrt();
    println!(
        "{}",
        serde_json::json!({
            "candidate": candidate, "baseline": baseline, "opponent": opponent,
            "seed": seed, "deals": deals, "rules": rules_name, "hands_per_version": deals*4,
            "money": [sums[0]/n, sums[1]/n], "win_share": [wins[0]/(n*4.0), wins[1]/(n*4.0)],
            "paired_difference": difference, "difference_ci95": [difference-ci,difference+ci],
            "moons": moons, "opponent_moons": opponent_moons, "queen_and_8_to_12_hearts": near_moons,
            "ms_per_hand": [micros[0] as f64/(n*4000.0),micros[1] as f64/(n*4000.0)],
            "max_move_ms": [max_micros[0] as f64/1000.0,max_micros[1] as f64/1000.0]
        })
    );
}
