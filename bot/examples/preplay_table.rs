//! Change all bot seats together, with accurate public human/bot identities.
mod support;
use turbo_hearts_api::*;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(
        args.len() >= 7,
        "preplay_table CANDIDATE BASELINE HUMAN_POLICY DEALS SEED HUMAN_COUNT"
    );
    let deals: usize = args[4].parse().unwrap();
    let first: u64 = args[5].parse().unwrap();
    let human_count: usize = args[6].parse().unwrap();
    assert!(deals >= 8 && (1..=3).contains(&human_count));
    let masks: Vec<_> = (1u8..15)
        .filter(|m| m.count_ones() as usize == human_count)
        .collect();
    let mut rows = Vec::new();
    let mut moons = [[0usize; 2]; 2];
    for seed in first..first + deals as u64 {
        let mut means = [0.0; 2];
        for version in 0..2 {
            // Cover every placement of the human seats with equal weight.
            for &mask in &masks {
                let humans = std::array::from_fn(|i| mask & (1 << i) != 0);
                let kinds = std::array::from_fn(|i| {
                    if humans[i] {
                        args[3].as_str()
                    } else {
                        args[1 + version].as_str()
                    }
                });
                let mut game = support::Match::dealt_with_humans(
                    seed,
                    PassDirection::from((seed % 4) as u8),
                    kinds,
                    ChargingRules::Classic,
                    humans,
                );
                game.finish_preplay();
                let scores = game.finish();
                let mut bot_money = 0.0;
                for seat in Seat::VALUES {
                    if !humans[seat.idx()] {
                        bot_money += scores.money(seat) as f64;
                    }
                    if game.game.state.won.queen(seat) && game.game.state.won.hearts(seat) == 13 {
                        moons[version][humans[seat.idx()] as usize] += 1;
                    }
                }
                means[version] += bot_money / (4 - human_count) as f64 / masks.len() as f64;
            }
        }
        rows.push(means);
        if rows.len() % 50 == 0 {
            eprintln!("{}/{} deals", rows.len(), deals);
        }
    }
    let n = deals as f64;
    let money: [f64; 2] = std::array::from_fn(|i| rows.iter().map(|r| r[i]).sum::<f64>() / n);
    let difference = money[0] - money[1];
    let ci = 1.96
        * (rows
            .iter()
            .map(|r| (r[0] - r[1] - difference).powi(2))
            .sum::<f64>()
            / (n - 1.0)
            / n)
            .sqrt();
    let directions: Vec<_> = (0..4).map(|d| {
        let group: Vec<_> = rows.iter().enumerate().filter(|(i,_)| (first as usize + i) % 4 == d).map(|(_,r)| r[0]-r[1]).collect();
        let k = group.len() as f64;
        let mean = group.iter().sum::<f64>() / k;
        let half = 1.96 * (group.iter().map(|v| (v-mean).powi(2)).sum::<f64>()/(k-1.0)/k).sqrt();
        serde_json::json!({"direction":(["left","right","across","keeper"][d]), "deals":group.len(), "paired_difference":mean, "difference_ci95":if group.len()>1 { Some([mean-half,mean+half]) } else { None }})
    }).collect();
    println!(
        "{}",
        serde_json::json!({"candidate":args[1],"baseline":args[2],"human_policy":args[3],
        "deals":deals,"seed":first,"human_count":human_count,"rules":"classic","public_types":"actual human/bot roles",
        "money_per_bot":money,"paired_difference":difference,"difference_ci95":[difference-ci,difference+ci],
        "by_direction":directions,"moons_bot_human_by_version":moons,"table_replays_per_version":deals*masks.len()})
    );
}
