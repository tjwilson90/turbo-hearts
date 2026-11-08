mod support;
use support::Match;
use turbo_hearts_api::{ChargingRules, PassDirection, Seat};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("arena [CANDIDATE] [BASELINE] [OPPONENT] [DEALS] [SEED] [RULES] [DIRECTION]\n\
Players: default, heuristic, duck, gottatry, random, search:SAMPLES:EXACT, moon:SAMPLES:EXACT, defend:SAMPLES:EXACT.\n\
Experiments with the same :SAMPLES:EXACT suffix: model, belief, posterior, tree, modeltree, allocate, guided, preplay, pass, charge, combined, modelcharge, posteriorcharge.\n\
Follow-ups: guard, cautious, cautiousguard, selective, consensus, guardconsensus, dynamic, dynamicguard, cautiousdynamic, dynamiccharge, cached, canonical.\n\
Human-play experiments: plan, claims, early, preserve, safe, planclaims, earlyclaims, safeclaims, safeearly, humanlike, humanmodel, humanmix, safemix (same :SAMPLES:EXACT suffix).\n\
Early-signal experiments on frozen round4: chargebelief, intent, disrupt, humanaware (same :SAMPLES:EXACT suffix).\n\
Successful-human profiles: skilledmix, skilledmodel, skilledintent, skilledclaims, skilledfallback (production), skilledadaptive.\n\
Direct-human profiles: actionmix, actionmodel, actionintent, actionclaims, actionplan (same :SAMPLES:EXACT suffix).\n\
Fitted responders: skilled, skilledclaim, skilledhybrid, action, actionclaim, actionhybrid. Hybrids use frozen round4 search.\n\
Capacity/history experiments: round7[:SAMPLES:EXACT:TIME_MS], coherent[:SAMPLES:EXACT:TIME_MS], neural:MODEL[:SAMPLES:EXACT:TIME_MS], neuralmix:MODEL[:SAMPLES:EXACT:TIME_MS].\n\
Payoff blend: payoff:MODEL[:SAMPLES:EXACT:TIME_MS:WEIGHT] (default weight 0.25). Omitted TIME_MS or none disables the soft cap.\n\
Learned preplay: prelearn:PASS_MODEL:CHARGE_MODEL[:SAMPLES:EXACT:TIME_MS:MARGIN:PASS_MASK:HUMAN_RECIPIENTS]; use none for either disabled head; PASS_MASK defaults to 15 (left=1, right=2, across=4, keeper=8); HUMAN_RECIPIENTS=1 restricts learned passes to public human destinations.\n\
Neural responders: neuralplayer:MODEL (with claims), neuralhybrid:MODEL[:SAMPLES:EXACT:TIME_MS] (half neural, half frozen round7 search).\n\
round4 freezes the earlier supported-stopper policy; all research profiles retain their frozen baselines.\n\
chargebelief/humanaware use slot 6 as charging-likelihood power (default 0.5); all retain stopper margin 8.\n\
preserve/safe profiles use the DEFENDER_FRACTION slot as the maximum stopper preference in money (default 4).\n\
human is the fitted human action model; humanclaim also makes provable claims. Neither represents live human opponents.\n\
Append :MODEL:PRIOR_SAMPLES to a search player to test a learned prior.\n\
Allocation: append :MODEL:0:MIN_ROUNDS (use none for an unguided allocator; default MIN_ROUNDS=32).\n\
After MIN_ROUNDS, append :DEFENDER_FRACTION:STYLE_SUPPORT for applicable follow-up profiles (defaults 1 and 0.9).\n\
Direct networks: policy:MODEL:TEMPERATURE (0 selects the highest score).\n\
OPPONENT also accepts mixed (heuristic, duck, gottatry). Each deal is replayed in all four seats.\n\
Rules: classic, blind, bridge, blind_bridge, chain, blind_chain. DIRECTION optionally fixes left, right, across, or keeper.");
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
    let fixed_direction: Option<PassDirection> = args.get(7).map(|v| v.parse().unwrap());
    let mut by_direction: [Vec<[f64; 2]>; 4] = std::array::from_fn(|_| Vec::new());
    let mut charge_counts = [[0usize; 4]; 2];
    let mut keeper_redraws = [0usize; 2];
    let mut sums = [0.0; 2];
    let mut differences = Vec::new();
    let mut wins = [0.0; 2];
    let mut moons = [0; 2];
    let mut opponent_moons = [0; 2];
    let mut near_moons = [0; 2];
    let mut micros = [0u128; 2];
    let mut preplay_micros = [0u128; 2];
    let mut root_evaluations = [0usize; 2];
    let mut max_micros = [0u128; 2];
    for deal in seed..seed + deals as u64 {
        let mut means = [0.0; 2];
        let direction = fixed_direction.unwrap_or_else(|| PassDirection::from((deal % 4) as u8));
        for (version, kind) in [candidate, baseline].iter().enumerate() {
            for &seat in &Seat::VALUES {
                let mut kinds = [opponent; 4];
                if opponent == "mixed" {
                    kinds[seat.left().idx()] = "heuristic";
                    kinds[seat.across().idx()] = "duck";
                    kinds[seat.right().idx()] = "gottatry";
                }
                kinds[seat.idx()] = kind;
                let mut game = Match::new(deal, direction, kinds, rules);
                let scores = game.finish();
                for card in game.game.state.charges.charges(seat) {
                    charge_counts[version][card.suit().idx()] += 1;
                }
                if direction == PassDirection::Keeper
                    && game
                        .game
                        .events
                        .iter()
                        .any(|e| matches!(e, turbo_hearts_api::GameEvent::SendPass { .. }))
                {
                    keeper_redraws[version] += 1;
                }
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
                preplay_micros[version] += game.preplay_micros[seat.idx()];
                root_evaluations[version] += game.root_evaluations[seat.idx()];
                max_micros[version] = max_micros[version].max(game.max_play_micros[seat.idx()]);
            }
            sums[version] += means[version];
        }
        differences.push(means[0] - means[1]);
        by_direction[direction as usize].push(means);
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
    let direction_results: Vec<_> = by_direction.iter().enumerate().filter(|(_, rows)| !rows.is_empty()).map(|(d, rows)| {
        let n = rows.len() as f64;
        let mean = rows.iter().map(|v| v[0]-v[1]).sum::<f64>() / n;
        let ci = if rows.len() > 1 { 1.96 * (rows.iter().map(|v| (v[0]-v[1]-mean).powi(2)).sum::<f64>()/(n-1.0)/n).sqrt() } else { 0.0 };
        serde_json::json!({"direction":(["left","right","across","keeper"][d]),"deals":rows.len(),
            "money":[rows.iter().map(|r| r[0]).sum::<f64>()/n,rows.iter().map(|r| r[1]).sum::<f64>()/n],
            "paired_difference":mean,"difference_ci95":if rows.len()>1 { Some([mean-ci,mean+ci]) } else { None }})
    }).collect();
    println!(
        "{}",
        serde_json::json!({
            "candidate": candidate, "baseline": baseline, "opponent": opponent,
            "seed": seed, "deals": deals, "rules": rules_name, "hands_per_version": deals*4,
            "by_direction":direction_results,"charges_by_suit":charge_counts,"keeper_redraws":keeper_redraws,
            "money": [sums[0]/n, sums[1]/n], "win_share": [wins[0]/(n*4.0), wins[1]/(n*4.0)],
            "paired_difference": difference, "difference_ci95": [difference-ci,difference+ci],
            "moons": moons, "opponent_moons": opponent_moons, "queen_and_8_to_12_hearts": near_moons,
            "ms_per_hand": [micros[0] as f64/(n*4000.0),micros[1] as f64/(n*4000.0)],
            "preplay_ms_per_hand": [preplay_micros[0] as f64/(n*4000.0),preplay_micros[1] as f64/(n*4000.0)],
        "root_evaluations_per_hand": [root_evaluations[0] as f64/(n*4.0),root_evaluations[1] as f64/(n*4.0)],
        "max_move_ms": [max_micros[0] as f64/1000.0,max_micros[1] as f64/1000.0]
        })
    );
}
