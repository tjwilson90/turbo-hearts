#[path = "../examples/support/replay.rs"]
mod replay;
use turbo_hearts_api::*;
use turbo_hearts_bot::*;

fn recorded() -> Vec<GameEvent> {
    serde_json::from_str(include_str!("fixtures/dan-game.json")).unwrap()
}

fn continuation() -> Vec<GameEvent> {
    // Simulated continuation, not the human's recorded subsequent actions.
    serde_json::from_str(include_str!("fixtures/dan-kept-ace-continuation.json")).unwrap()
}

fn branched(plays: usize) -> replay::Replay {
    let mut replay = replay::Replay::new();
    for event in recorded()[..145].iter().chain(&continuation()[..plays]) {
        replay.apply(event, |_, _, _| {});
    }
    replay
}

fn play(replay: &mut replay::Replay, card: Card) {
    replay.apply(
        &GameEvent::Play {
            seat: replay.game.state.next_actor.unwrap(),
            card,
        },
        |_, _, _| {},
    );
}

#[test]
fn historical_replay_recovers_scores_and_only_public_opponent_cards() {
    let mut replay = replay::Replay::new();
    let mut maker = HandMaker::new();
    let mut scores = Vec::new();
    for (index, event) in recorded().iter().enumerate() {
        if index == 145 {
            assert_eq!(replay.game.state.next_actor, Some(Seat::West));
            assert_eq!(
                replay.states[3].post_pass_hand - replay.game.state.played,
                "T6S A862H AQJT92D".parse().unwrap()
            );
            // West passed these cards across, and saw East charge QS and play
            // 6C. The actual AS/9S ownership is still hidden from West.
            assert_eq!(maker.known_cards(Seat::East), "QS KH K65C".parse().unwrap());
            assert_eq!(
                maker.known_cards(Seat::West),
                replay.states[3].post_pass_hand
            );
        }
        replay.apply(event, |own, state, event| {
            if own.seat == Seat::West {
                maker.on_event(state, event);
                if let GameEvent::HandComplete {
                    north_score,
                    east_score,
                    south_score,
                    west_score,
                } = event
                {
                    scores.push([*north_score, *east_score, *south_score, *west_score]);
                }
            }
        });
    }
    assert_eq!(
        scores,
        [
            [34, 0, 0, 12],
            [0, 0, 26, 3],
            [0, -98, 0, 0],
            [-8, 14, 3, 14]
        ]
    );
    assert!(replay.game.state.phase.is_complete());
}

#[test]
fn charged_moon_stopper_preference_retains_raw_values_and_bounds_the_sacrifice() {
    let mut changed = 0;
    for seed in 0..16 {
        let config = SearchConfig {
            time_limit: None,
            ..SearchConfig::default()
        };
        let mut old = NeuralNetworkBot::with_config(config.clone(), seed)
            .with_experiments(SearchExperiments::round_three());
        let mut new = NeuralNetworkBot::with_config(config, seed)
            .with_experiments(SearchExperiments::round_four());
        let mut replay = replay::Replay::new();
        for event in &recorded()[..145] {
            replay.apply(event, |own, state, event| {
                if own.seat == Seat::West {
                    old.on_event(own, state, event);
                    new.on_event(own, state, event);
                }
            });
        }
        let own = &replay.states[3];
        let state = &replay.observations[3];
        let values = old.clone().evaluate(own, state);
        assert_eq!(values, new.clone().evaluate(own, state));
        let before = old.play(own, state);
        let after = new.play(own, state);
        if before != after {
            changed += 1;
            assert_eq!(before, Card::AceHearts);
            assert_ne!(after, Card::AceHearts);
        }
        let best = values.iter().map(|v| v.1).fold(f32::NEG_INFINITY, f32::max);
        let chosen = values.iter().find(|v| v.0 == after).unwrap().1;
        assert!(chosen >= best - 8.0);
    }
    assert!(changed > 0);
}

#[test]
fn cashing_the_retained_ace_breaks_the_moon_before_losing_control() {
    let mut replay = branched(15);
    assert_eq!(replay.game.state.next_actor, Some(Seat::West));
    assert!(replay.game.state.current_trick.is_empty());
    assert!(replay.game.state.won.jack(Seat::West));
    assert!(replay.game.state.won.can_run(Seat::East));
    for card in [
        Card::AceHearts,
        Card::FiveHearts,
        Card::KingHearts,
        Card::FourHearts,
    ] {
        play(&mut replay, card);
    }
    assert!(replay.game.state.current_trick.is_empty());
    assert_eq!(replay.game.state.next_actor, Some(Seat::West));
    assert!(!replay.game.state.won.can_run(Seat::East));
}

#[test]
fn nine_assisted_claim_can_bypass_an_uncashed_ace() {
    let mut replay = branched(23);
    let own = &replay.states[1];
    let state = &replay.observations[1];
    assert_eq!(state.next_actor, Some(Seat::East));
    assert_eq!(
        own.post_pass_hand - state.played,
        "A9S KH K85C".parse().unwrap()
    );
    assert!((replay.states[3].post_pass_hand - state.played).contains(Card::AceHearts));
    assert!(should_claim(state, own.void, own.seat, own.post_pass_hand));

    // Exhibit why AH no longer stops the run: AS removes the last opposing
    // spade, then 9S lets East slough its losing KH onto its own extended trick.
    for card in [
        Card::AceSpades,
        Card::FourDiamonds,
        Card::TenDiamonds,
        Card::TwoSpades,
        Card::NineSpades,
        Card::FourClubs,
        Card::QueenDiamonds,
        Card::SevenDiamonds,
    ] {
        play(&mut replay, card);
    }
    assert!(!replay.game.state.current_trick.is_empty());
    assert_eq!(replay.game.state.next_actor, Some(Seat::East));
    for card in [
        Card::KingHearts,
        Card::FourHearts,
        Card::TwoHearts,
        Card::ThreeHearts,
    ] {
        play(&mut replay, card);
    }
    assert!(replay.game.state.current_trick.is_empty());
    assert_eq!(replay.game.state.next_actor, Some(Seat::East));
    assert!(replay.game.state.won.can_run(Seat::East));
    assert!((replay.states[3].post_pass_hand - replay.game.state.played).contains(Card::AceHearts));

    // Separately replay the saved valid claim and verify the scoring engine.
    let replay = branched(continuation().len());
    let scores = replay.game.state.scores();
    assert_eq!(Seat::VALUES.map(|s| scores.money(s)), [-88, 224, -88, -48]);
    assert!(!replay.game.state.phase.is_playing());
}
