#[path = "../examples/support/replay.rs"]
mod replay;
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;
use turbo_hearts_api::*;
use turbo_hearts_bot::*;

fn events() -> Vec<GameEvent> {
    serde_json::from_str(include_str!("fixtures/ace-slough.json")).unwrap()
}

#[test]
fn direct_human_model_matches_numpy_and_returns_legal_reproducible_moves() {
    let mut replay = replay::Replay::new();
    for e in &events()[..37] {
        replay.apply(e, |_, _, _| {});
    }
    let own = &replay.states[2];
    let state = &replay.observations[2];
    let distribution = HumanActionPolicy::distribution(own, state);
    let expected = [
        0.23282142167651118,
        0.342392306627639,
        0.0597286609432525,
        0.04731332280615433,
        0.027006932183459776,
        0.1516997632780924,
        0.06949879761613878,
        0.06953879486875207,
    ];
    assert_eq!(distribution.len(), expected.len());
    for ((card, p), e) in distribution.iter().zip(expected) {
        assert!(state.legal_plays(own.post_pass_hand).contains(*card));
        assert!((p - e).abs() < 1e-7); // training export uses float32 features
    }
    assert!((distribution.iter().map(|v| v.1).sum::<f64>() - 1.0).abs() < 1e-12);
    let mut a = ChaCha12Rng::seed_from_u64(99);
    let mut b = a.clone();
    for _ in 0..50 {
        assert_eq!(
            HumanActionPolicy::play_with_rng(own, state, &mut a),
            HumanActionPolicy::play_with_rng(own, state, &mut b)
        );
    }
    let p: f64 = state
        .legal_plays(own.post_pass_hand)
        .into_iter()
        .map(|c| SkilledPolicy::probability(own, state, c))
        .sum();
    assert!((p - 1.0).abs() < 1e-12);
    // Independent NumPy softmax from the JSON checkpoint and this hand's
    // context: one master, one controlled nine, two voids, seven losers.
    let expected_weights = [
        0.4836616647061775,
        0.20231855007832367,
        0.18944981118994741,
        0.07666017619666321,
        0.02501146996354546,
        0.02289832786534275,
    ];
    for (actual, expected) in SkilledPolicy::weights(own, state)
        .iter()
        .zip(expected_weights)
    {
        assert!((actual - expected).abs() < 1e-12);
    }
}

#[test]
fn successful_human_continuations_change_only_publicly_identified_human_modeling() {
    let config = SearchConfig {
        samples: 16,
        time_limit: None,
        ..SearchConfig::default()
    };
    let mut base = NeuralNetworkBot::with_config(config.clone(), 23)
        .with_experiments(SearchExperiments::round_four());
    let mut candidate =
        NeuralNetworkBot::with_config(config, 23).with_experiments(SearchExperiments {
            skilled_model: true,
            human_rollouts: 1.0,
            ..SearchExperiments::round_four()
        });
    let mut replay = replay::Replay::new();
    for event in &events()[..37] {
        replay.apply(event, |own, state, event| {
            if own.seat == Seat::South {
                base.on_event(own, state, event);
                candidate.on_event(own, state, event);
            }
        });
    }
    let before = base.evaluate(&replay.states[2], &replay.observations[2]);
    let after = candidate.evaluate(&replay.states[2], &replay.observations[2]);
    assert_eq!(
        before.iter().map(|v| v.0).collect::<Vec<_>>(),
        after.iter().map(|v| v.0).collect::<Vec<_>>()
    );
    assert_ne!(before, after);
}

#[test]
fn human_continuations_leave_all_bot_information_sets_unchanged() {
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
    let config = SearchConfig {
        samples: 16,
        time_limit: None,
        ..SearchConfig::default()
    };
    for experiments in [
        SearchExperiments {
            human_rollouts: 0.5,
            action_model: true,
            ..SearchExperiments::round_four()
        },
        SearchExperiments {
            human_rollouts: 1.0,
            skilled_model: true,
            ..SearchExperiments::round_four()
        },
        SearchExperiments {
            human_rollouts: 1.0,
            skilled_model: true,
            human_model_fallback: true,
            early_intent: true,
            ..SearchExperiments::round_four()
        },
        SearchExperiments::production(),
    ] {
        let mut base = NeuralNetworkBot::with_config(config.clone(), 23)
            .with_experiments(SearchExperiments::round_four());
        let mut candidate =
            NeuralNetworkBot::with_config(config.clone(), 23).with_experiments(experiments);
        let mut replay = replay::Replay::new();
        for e in &events[..37] {
            replay.apply(e, |own, state, event| {
                if own.seat == Seat::South {
                    base.on_event(own, state, event);
                    candidate.on_event(own, state, event);
                }
            });
        }
        for _ in 0..2 {
            assert_eq!(
                base.evaluate(&replay.states[2], &replay.observations[2]),
                candidate.evaluate(&replay.states[2], &replay.observations[2])
            );
        }
    }
}
