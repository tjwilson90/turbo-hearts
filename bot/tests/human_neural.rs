#[path = "../examples/support/replay.rs"]
mod replay;
#[path = "../examples/support/mod.rs"]
mod support;
use turbo_hearts_api::*;
use turbo_hearts_bot::*;
fn model() -> HumanNetwork {
    HumanNetwork::from_bytes(include_bytes!("../../assets/models/human-history32.thh")).unwrap()
}
fn events() -> Vec<GameEvent> {
    serde_json::from_str(include_str!("fixtures/ace-slough.json")).unwrap()
}
#[test]
fn neural_coefficients_match_numpy_and_reject_malformed_models() {
    for (network, json) in [
        (
            model(),
            include_str!("../evaluation/round8/neural-reference.json"),
        ),
        (
            HumanNetwork::from_bytes(include_bytes!("../../assets/models/human-capacity32.thh"))
                .unwrap(),
            include_str!("../evaluation/round8/capacity-reference.json"),
        ),
    ] {
        let reference: serde_json::Value = serde_json::from_str(json).unwrap();
        let x: Vec<f32> = serde_json::from_value(reference["context"].clone()).unwrap();
        let expected: Vec<f32> = serde_json::from_value(reference["coefficients"].clone()).unwrap();
        let mut input = [0.0; HUMAN_INPUTS];
        input.copy_from_slice(&x);
        for (actual, expected) in network.coefficients(&input).iter().zip(expected) {
            assert!((actual - expected).abs() < 2e-5);
        }
    }
    let bytes = include_bytes!("../../assets/models/human-history32.thh");
    assert!(HumanNetwork::from_bytes(&bytes[..19]).is_err());
    let mut bad = bytes.to_vec();
    bad[20..24].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(HumanNetwork::from_bytes(&bad).is_err());
    let mut bad = bytes.to_vec();
    bad[16..20].copy_from_slice(&9u32.to_le_bytes());
    assert!(HumanNetwork::from_bytes(&bad).is_err());
}
#[test]
fn public_history_and_private_passes_have_different_visibility() {
    let mut replay = replay::Replay::new();
    let mut memory = HumanMemory::default();
    for event in &events()[..37] {
        replay.apply(event, |own, state, event| {
            if own.seat == Seat::South {
                memory.on_event(own, state, event);
            }
        });
    }
    let own = &replay.states[2];
    let state = &replay.observations[2];
    let (x, _) = neural_human_observation(own, state, &memory);
    assert_eq!(x[174 + human_card_index(Card::QueenHearts)], 1.0);
    assert_eq!(x[362], 1.0);
    assert!(x[330..362].iter().any(|x| *x > 0.0));
    let (public, _) = neural_human_observation(own, state, &memory.without_private());
    assert_eq!(&x[..174], &public[..174]);
    assert_eq!(&x[330..362], &public[330..362]);
    assert!(public[174..330].iter().all(|x| *x == 0.0));
    assert_eq!(public[362], 0.0);
    let (opponent, _) =
        neural_human_observation(&replay.states[3], &replay.observations[3], &memory);
    assert!(opponent[174..330].iter().all(|x| *x == 0.0));
    assert_eq!(opponent[362], 0.0);
    let probabilities = model().distribution(own, state, &memory);
    assert!((probabilities.iter().map(|p| p.1).sum::<f64>() - 1.0).abs() < 1e-12);
    assert!(probabilities
        .iter()
        .all(|(c, p)| state.legal_plays(own.post_pass_hand).contains(*c) && p.is_finite()));
}
#[test]
fn neural_human_policy_and_coherent_plans_preserve_all_bot_values() {
    let mut events = events();
    if let GameEvent::Sit {
        north,
        east,
        south,
        west,
        ..
    } = &mut events[0]
    {
        for player in [north, east, south, west] {
            *player = Player::Bot {
                user_id: player.user_id(),
                strategy: BotStrategy::NeuralNet,
            };
        }
    }
    for coherent in [false, true] {
        let config = SearchConfig {
            samples: 8,
            time_limit: None,
            ..SearchConfig::default()
        };
        let mut base = NeuralNetworkBot::with_config(config.clone(), 17)
            .with_experiments(SearchExperiments::round_seven());
        let mut candidate = NeuralNetworkBot::with_config(config, 17)
            .with_experiments(SearchExperiments {
                coherent_humans: coherent,
                ..SearchExperiments::round_seven()
            })
            .with_human_network(model())
            .with_payoff_network(
                HumanNetwork::from_bytes(include_bytes!("../../assets/models/human-payoff32.thh"))
                    .unwrap(),
                0.5,
            );
        let mut replay = replay::Replay::new();
        for event in &events[..37] {
            replay.apply(event, |own, state, event| {
                if own.seat == Seat::South {
                    base.on_event(own, state, event);
                    candidate.on_event(own, state, event);
                }
            });
        }
        assert_eq!(
            base.evaluate(&replay.states[2], &replay.observations[2]),
            candidate.evaluate(&replay.states[2], &replay.observations[2])
        );
        assert_eq!(
            base.play(&replay.states[2], &replay.observations[2]),
            candidate.play(&replay.states[2], &replay.observations[2])
        );
    }
}

#[test]
fn large_rollout_ceiling_keeps_time_bounded_inference_preparation() {
    let make = |samples| {
        NeuralNetworkBot::with_config(
            SearchConfig {
                samples,
                time_limit: Some(std::time::Duration::ZERO),
                ..SearchConfig::default()
            },
            17,
        )
        .with_experiments(SearchExperiments::round_seven())
    };
    let mut base = make(128);
    let mut large = make(65536);
    let mut replay = replay::Replay::new();
    for e in &events()[..37] {
        replay.apply(e, |own, state, e| {
            if own.seat == Seat::South {
                base.on_event(own, state, e);
                large.on_event(own, state, e);
            }
        });
    }
    assert_eq!(
        base.evaluate(&replay.states[2], &replay.observations[2]),
        large.evaluate(&replay.states[2], &replay.observations[2])
    );
    assert!(large.last_belief_ess() > 0.0 && large.last_belief_ess() <= 256.0);
    assert_eq!(
        base.last_search_evaluations(),
        large.last_search_evaluations()
    );
}
#[test]
fn modern_profiles_honor_time_arguments_and_finish_every_charging_rule() {
    let root = env!("CARGO_MANIFEST_DIR");
    let profiles = [
        "round7:8:8:0".to_string(),
        "coherent:8:8:0".to_string(),
        format!("neural:{}/../assets/models/human-history32.thh:8:8:0", root),
        format!(
            "payoff:{}/../assets/models/human-payoff32.thh:8:8:0:0.5",
            root
        ),
        format!(
            "payoff:{}/../assets/models/human-payoff32.thh:8:8:none:0.5",
            root
        ),
    ];
    for rules in [
        ChargingRules::Classic,
        ChargingRules::Blind,
        ChargingRules::Bridge,
        ChargingRules::BlindBridge,
        ChargingRules::Chain,
        ChargingRules::BlindChain,
    ] {
        for profile in &profiles {
            let mut game = support::Match::new(
                88,
                PassDirection::Keeper,
                [profile, "humanclaim", "duck", "gottatry"],
                rules,
            );
            let scores = game.finish();
            assert_eq!(
                Seat::VALUES.iter().map(|&s| scores.money(s)).sum::<i16>(),
                0
            );
        }
    }
}
