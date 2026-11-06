#[path = "../examples/support/mod.rs"]
mod support;
use support::Match;
use turbo_hearts_api::*;
use turbo_hearts_bot::*;

#[test]
fn completes_legal_reproducible_games_under_every_ruleset() {
    for (i, rules) in [
        ChargingRules::Classic,
        ChargingRules::Blind,
        ChargingRules::Bridge,
        ChargingRules::BlindBridge,
        ChargingRules::Chain,
        ChargingRules::BlindChain,
    ]
    .iter()
    .enumerate()
    {
        for direction in 0..4 {
            let mut a = Match::new(
                (i * 4 + direction) as u64,
                PassDirection::from(direction as u8),
                ["defend:4:12"; 4],
                *rules,
            );
            let mut b = Match::new(
                (i * 4 + direction) as u64,
                PassDirection::from(direction as u8),
                ["defend:4:12"; 4],
                *rules,
            );
            a.finish();
            b.finish();
            assert_eq!(a.game.state, b.game.state);
            assert_eq!(a.game.state.played, Cards::ALL);
            assert_eq!(
                Seat::VALUES
                    .iter()
                    .map(|s| a.game.state.scores().money(*s))
                    .sum::<i16>(),
                0
            );
        }
    }
}

#[test]
fn observer_receives_only_own_deal_and_pass_information() {
    let game = Match::new(
        22,
        PassDirection::Left,
        ["search:2:0"; 4],
        ChargingRules::Classic,
    );
    // Rebuild each observer from the actual redacted event stream.
    for seat in Seat::VALUES {
        let mut state = GameState::new();
        let mut maker = HandMaker::new();
        for event in &game.game.events {
            let redacted = event.redact(Some(seat), ChargingRules::Classic);
            maker.on_event(&state, &redacted);
            state.apply(&redacted);
        }
        assert_eq!(
            maker.known_cards(seat),
            game.states[seat.idx()].post_pass_hand
        );
        for other in Seat::VALUES {
            assert!(game.states[other.idx()]
                .post_pass_hand
                .contains_all(maker.known_cards(other)));
            if other != seat {
                assert!(maker.known_cards(other).len() <= 7);
            }
        }
    }
}

#[test]
fn zero_time_budget_still_returns_a_legal_move() {
    let mut game = Match::new(
        22,
        PassDirection::Left,
        ["heuristic"; 4],
        ChargingRules::Classic,
    );
    game.play_one();
    let seat = game.game.state.next_actor.unwrap();
    let mut bot = NeuralNetworkBot::with_config(
        SearchConfig {
            samples: 4,
            moon: false,
            opponent_moon: false,
            exact_cards: 0,
            time_limit: Some(std::time::Duration::ZERO),
        },
        1,
    );
    let mut state = GameState::new();
    for event in &game.game.events {
        let event = event.redact(Some(seat), ChargingRules::Classic);
        bot.on_event(&game.states[seat.idx()], &state, &event);
        state.apply(&event);
    }
    let card = bot.play(&game.states[seat.idx()], &game.game.state);
    assert!(game
        .game
        .state
        .legal_plays(game.states[seat.idx()].post_pass_hand)
        .contains(card));
}

#[test]
fn experimental_search_respects_legality_budgets_and_replay() {
    for profile in [
        "model:8:8",
        "belief:8:8",
        "posterior:8:8",
        "tree:8:8",
        "allocate:8:8:none:0:2",
        "guided:8:8:../assets/models/policy-moon.thp:0:2",
        "guard:8:8",
        "dynamic:8:8",
        "cautious:8:8:none:0:32:0:0.6",
        "consensus:8:8",
        "canonical:8:16",
    ] {
        let mut game = Match::new(
            983,
            PassDirection::Across,
            [profile, "heuristic", "duck", "gottatry"],
            ChargingRules::Classic,
        );
        while game.game.state.phase.is_playing() {
            let seat = game.game.state.next_actor.unwrap();
            if seat == Seat::North {
                let state = &game.game.state;
                let bot = &game.states[0];
                let mut copy = game.players[0].search.as_ref().unwrap().clone();
                let legal = state
                    .legal_plays(bot.post_pass_hand)
                    .distinct_plays(state.played, state.current_trick);
                let expected = copy.play(bot, state);
                let count = copy.last_search_evaluations();
                assert!(count <= 8 * legal.len() * 2);
                let actual = game.players[0].search.as_mut().unwrap().play(bot, state);
                assert_eq!(actual, expected);
                if profile.starts_with("belief") || profile.starts_with("posterior") {
                    let ess = copy.last_belief_ess();
                    assert!(ess.is_finite());
                    if legal.len() > 1 {
                        assert!(ess >= 1.0 && ess <= 64.0 + 1e-8);
                    }
                }
                game.play_card(actual);
            } else {
                game.play_one();
            }
        }
        assert_eq!(game.game.state.played, Cards::ALL);
    }
}

#[test]
fn uncertain_behavior_preserves_baseline_values_and_random_stream() {
    let mut game = Match::new(
        500,
        PassDirection::Left,
        ["heuristic"; 4],
        ChargingRules::Classic,
    );
    game.play_one();
    let seat = game.game.state.next_actor.unwrap();
    let bot = &game.states[seat.idx()];
    let config = SearchConfig {
        samples: 16,
        time_limit: None,
        ..SearchConfig::default()
    };
    let mut baseline = NeuralNetworkBot::with_config(config.clone(), 91);
    let mut cautious =
        NeuralNetworkBot::with_config(config, 91).with_experiments(SearchExperiments {
            conservative_model: true,
            style_confidence: 0.6,
            ..SearchExperiments::default()
        });
    let mut state = GameState::new();
    for event in &game.game.events {
        let event = event.redact(Some(seat), ChargingRules::Classic);
        baseline.on_event(bot, &state, &event);
        cautious.on_event(bot, &state, &event);
        state.apply(&event);
    }
    for _ in 0..2 {
        assert_eq!(
            baseline.evaluate(bot, &state),
            cautious.evaluate(bot, &state)
        );
    }
}

#[test]
fn arena_default_matches_confirmed_policy_under_every_ruleset() {
    for (i, rules) in [
        ChargingRules::Classic,
        ChargingRules::Blind,
        ChargingRules::Bridge,
        ChargingRules::BlindBridge,
        ChargingRules::Chain,
        ChargingRules::BlindChain,
    ]
    .iter()
    .enumerate()
    {
        let make = |profile| {
            Match::new(
                150000 + i as u64,
                PassDirection::from((i % 4) as u8),
                [profile, "heuristic", "duck", "gottatry"],
                *rules,
            )
        };
        let mut production = make("default");
        let mut confirmed = make("cautiousdynamic:128:12:none:0:32:0:0.6");
        production.finish();
        confirmed.finish();
        assert_eq!(production.game.state, confirmed.game.state);
    }
}

#[test]
fn experimental_preplay_and_beliefs_complete_every_ruleset() {
    for (i, rules) in [
        ChargingRules::Classic,
        ChargingRules::Blind,
        ChargingRules::Bridge,
        ChargingRules::BlindBridge,
        ChargingRules::Chain,
        ChargingRules::BlindChain,
    ]
    .iter()
    .enumerate()
    {
        for direction in 0..4 {
            for profile in [
                "combined:4:8",
                "dynamiccharge:4:8",
                "cautiousdynamic:4:8:none:0:32:0:0.6",
            ] {
                let mut game = Match::new(
                    320 + (i * 4 + direction) as u64,
                    PassDirection::from(direction as u8),
                    [profile, "heuristic", "duck", "gottatry"],
                    *rules,
                );
                game.finish();
                assert_eq!(game.game.state.played, Cards::ALL);
            }
        }
    }
}
