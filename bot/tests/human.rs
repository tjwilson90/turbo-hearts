#[path = "../examples/support/replay.rs"]
mod replay;
#[path = "../examples/support/mod.rs"]
mod support;
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;
use turbo_hearts_api::*;
use turbo_hearts_bot::*;

fn ace_position() -> (replay::Replay, Vec<GameEvent>) {
    let events: Vec<GameEvent> =
        serde_json::from_str(include_str!("fixtures/ace-slough.json")).unwrap();
    let mut replay = replay::Replay::new();
    for event in &events[..37] {
        replay.apply(event, |_, _, _| {});
    }
    (replay, events)
}

#[test]
fn recorded_slough_is_the_only_discard_allowing_the_claim() {
    let (replay, _) = ace_position();
    let bot = &replay.states[2];
    let state = &replay.observations[2];
    assert_eq!(
        bot.post_pass_hand - state.played,
        "AJ964H 742D".parse().unwrap()
    );
    assert_eq!(state.won, WonState::new());
    for card in state.legal_plays(bot.post_pass_hand) {
        let mut after = state.clone();
        let event = GameEvent::Play {
            seat: Seat::South,
            card,
        };
        let void = bot.void.on_event(state, &event);
        after.apply(&event);
        assert_eq!(
            can_claim(
                &after,
                void,
                Seat::West,
                replay.game.post_pass_hand[3] - after.played
            ),
            card == Card::AceHearts
        );
    }
}

#[test]
fn human_policy_is_normalized_legal_and_private() {
    let (replay, _) = ace_position();
    let bot = &replay.states[2];
    let state = &replay.observations[2];
    let legal = state.legal_plays(bot.post_pass_hand);
    let total: f64 = legal
        .into_iter()
        .map(|c| HumanPolicy::probability(bot, state, c))
        .sum();
    assert!((total - 1.0).abs() < 1e-12);
    // Independent NumPy evaluation of the exported coefficients at this
    // position, including the 23/52 public-game-progress feature.
    let expected = [
        0.4679194347637879,
        0.19461598600655386,
        0.20770451771499893,
        0.053901747239783665,
        0.0758583142748756,
    ];
    for (actual, expected) in HumanPolicy::weights(bot, state).iter().zip(expected) {
        assert!((actual - expected).abs() < 1e-12);
    }
    for choices in human_components(bot, state) {
        assert!(!choices.is_empty() && legal.contains_all(choices));
    }
    let mut a = ChaCha12Rng::seed_from_u64(1);
    let mut b = a.clone();
    for _ in 0..100 {
        let card = HumanPolicy::play_with_rng(bot, state, &mut a);
        assert!(legal.contains(card));
        assert_eq!(card, HumanPolicy::play_with_rng(bot, state, &mut b));
    }
    // Reconstruct only the actor's view. Opponent hands never enter features.
    let mut maker = HandMaker::new();
    let mut state = GameState::new();
    for event in &replay.game.events {
        let event = event.redact(Some(Seat::South), ChargingRules::Classic);
        maker.on_event(&state, &event);
        state.apply(&event);
    }
    assert!(maker.known_cards(Seat::West).contains(Card::QueenHearts));
    assert!(!maker.known_cards(Seat::West).contains(Card::KingHearts));
}

#[test]
fn bounded_stopper_preference_preserves_the_ace_across_seeds() {
    let (_, events) = ace_position();
    for seed in 0..32 {
        let mut bot = NeuralNetworkBot::with_config(
            SearchConfig {
                samples: 128,
                time_limit: None,
                ..SearchConfig::default()
            },
            seed,
        )
        .with_experiments(SearchExperiments {
            stopper_margin: 8.0,
            ..SearchExperiments::round_three()
        });
        let mut replay = replay::Replay::new();
        for event in &events[..37] {
            replay.apply(event, |own, state, event| {
                if own.seat == Seat::South {
                    bot.on_event(own, state, event);
                }
            });
        }
        let own = &replay.states[2];
        let state = &replay.observations[2];
        let values = bot.clone().evaluate(own, state);
        let choice = bot.play(own, state);
        assert_ne!(choice, Card::AceHearts, "seed {}", seed);
        let best = values.iter().map(|v| v.1).fold(f32::NEG_INFINITY, f32::max);
        let chosen = values.iter().find(|v| v.0 == choice).unwrap().1;
        assert!(chosen >= best - 8.0);
    }
}

#[test]
fn human_and_defensive_variants_finish_under_every_charging_rule() {
    for rules in [
        ChargingRules::Classic,
        ChargingRules::Blind,
        ChargingRules::Bridge,
        ChargingRules::BlindBridge,
        ChargingRules::Chain,
        ChargingRules::BlindChain,
    ] {
        for profile in [
            "plan:4:8",
            "claims:4:8",
            "humanmix:4:8",
            "preserve:4:8:none:0:32:8",
            "humanaware:4:8",
            "actionmix:4:8",
            "actionplan:4:8",
            "skilledfallback:4:8",
            "skilledadaptive:4:8",
        ] {
            let mut game = support::Match::new(
                492,
                PassDirection::Keeper,
                [profile, "humanclaim", "duck", "gottatry"],
                rules,
            );
            let scores = game.finish();
            assert!(!game.game.state.phase.is_playing());
            assert_eq!(
                Seat::VALUES.iter().map(|&s| scores.money(s)).sum::<i16>(),
                0
            );
        }
    }
}
