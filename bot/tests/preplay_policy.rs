#[path = "../examples/support/mod.rs"]
mod support;
use turbo_hearts_api::*;
use turbo_hearts_bot::*;

fn network(kind: PreplayKind, inputs: &[(usize, f32)], bias: f32, output: f32) -> PreplayNetwork {
    let mut bytes = b"TPQ1".to_vec();
    for value in [PREPLAY_FEATURES as u32, PREPLAY_HIDDEN as u32, kind as u32] {
        bytes.extend(value.to_le_bytes());
    }
    let n = PREPLAY_FEATURES * PREPLAY_HIDDEN;
    let mut weights = vec![0.0f32; n + 2 * PREPLAY_HIDDEN + 1];
    for &(input, value) in inputs {
        weights[input * PREPLAY_HIDDEN] = value;
    }
    weights[n] = bias;
    weights[n + PREPLAY_HIDDEN] = output;
    for value in weights {
        bytes.extend(value.to_le_bytes());
    }
    PreplayNetwork::from_bytes(&bytes).unwrap()
}

#[test]
fn fitted_heads_match_numpy_and_reject_invalid_checkpoints() {
    let input = std::array::from_fn(|i| (i % 7) as f32 / 7.0);
    for (bytes, kind, expected) in [
        (
            include_bytes!("../../assets/models/preplay-pass1.tpq").as_slice(),
            PreplayKind::Pass,
            -11.754528f32,
        ),
        (
            include_bytes!("../../assets/models/preplay-charge1.tpq").as_slice(),
            PreplayKind::Charge,
            53.960182f32,
        ),
        (
            include_bytes!("../../assets/models/preplay-pass2.tpq").as_slice(),
            PreplayKind::Pass,
            -30.932823f32,
        ),
        (
            include_bytes!("../../assets/models/preplay-charge2.tpq").as_slice(),
            PreplayKind::Charge,
            59.33753f32,
        ),
    ] {
        let model = PreplayNetwork::from_bytes(bytes).unwrap();
        assert_eq!(model.kind(), kind);
        assert!((model.predict(&input) - expected).abs() < 0.0001);
        assert!(PreplayNetwork::from_bytes(&bytes[..bytes.len() - 1]).is_err());
        let mut invalid = bytes.to_vec();
        invalid[16..20].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(PreplayNetwork::from_bytes(&invalid).is_err());
        invalid = bytes.to_vec();
        invalid[12..16].copy_from_slice(&2u32.to_le_bytes());
        assert!(PreplayNetwork::from_bytes(&invalid).is_err());
    }
}

#[test]
fn additive_charge_export_matches_numpy() {
    let model = PreplayNetwork::from_bytes(include_bytes!(
        "../../assets/models/preplay-charge-additive.tpq"
    ))
    .unwrap();
    let mut input = std::array::from_fn(|i| (i % 7) as f32 / 7.0);
    for (feature, value) in [(112, 1.0), (126, 0.0), (142, 1.0), (153, 1.0)] {
        input[feature] = value;
    }
    assert!((model.predict(&input) - 33.933495).abs() < 0.003);
}

#[test]
fn passes_condition_on_direction_and_keep_exactly_three_legal_cards() {
    let bot = BotState::new(Seat::North, "AKQJT987S AH 234C 2D".parse().unwrap());
    let baseline: Cards = "234C".parse().unwrap();
    let ace_feature = 104 + Suit::Hearts.idx() * 13 + Rank::Ace.idx();
    let model = network(
        PreplayKind::Pass,
        &[(ace_feature, 1.0), (157, 1.0)],
        -1.5,
        1.0,
    );
    let mut state = GameState::new();
    assert_eq!(model.choose(&bot, &state, baseline, 0.0), baseline);
    state.phase = GamePhase::PassRight;
    let choice = model.choose(&bot, &state, baseline, 0.0);
    assert_eq!(choice.len(), 3);
    assert!(choice.contains(Card::AceHearts) && bot.pre_pass_hand.contains_all(choice));
    for phase in [
        GamePhase::PassLeft,
        GamePhase::PassRight,
        GamePhase::PassAcross,
        GamePhase::PassKeeper,
    ] {
        state.phase = phase;
        assert_eq!(preplay_actions(&bot, &state, PreplayKind::Pass).len(), 286);
    }
}

#[test]
fn learned_passing_can_be_enabled_independently_in_each_round() {
    let bot = BotState::new(Seat::North, "AKQJT987S AH 234C 2D".parse().unwrap());
    for (index, phase) in [
        GamePhase::PassLeft,
        GamePhase::PassRight,
        GamePhase::PassAcross,
        GamePhase::PassKeeper,
    ]
    .iter()
    .copied()
    .enumerate()
    {
        let mut state = GameState::new();
        state.phase = phase;
        let baseline = HeuristicBot.pass(&bot, &state);
        let wanted = (bot.pre_pass_hand - baseline).max();
        let feature = 104 + wanted.suit().idx() * 13 + wanted.rank().idx();
        let policy = NeuralNetworkBot::with_seed(9).with_preplay_models(
            Some(network(PreplayKind::Pass, &[(feature, 1.0)], 0.0, 1.0)),
            None,
            0.0,
        );
        let mut enabled = [false; 4];
        enabled[index] = true;
        assert!(policy
            .clone()
            .with_preplay_pass_directions(enabled)
            .pass(&bot, &state)
            .contains(wanted));
        enabled[index] = false;
        assert_eq!(
            policy
                .with_preplay_pass_directions(enabled)
                .pass(&bot, &state),
            baseline
        );
    }
}

#[test]
fn recipient_restriction_uses_public_seat_types_and_actual_pass_direction() {
    let profile = format!(
        "prelearn:{}/../assets/models/preplay-pass2.tpq:none:4:8:none:0:7:1",
        env!("CARGO_MANIFEST_DIR")
    );
    let model = PreplayNetwork::from_bytes(include_bytes!("../../assets/models/preplay-pass2.tpq"))
        .unwrap();
    let mut changed = 0;
    for direction in [
        PassDirection::Left,
        PassDirection::Right,
        PassDirection::Across,
    ] {
        for humans in [
            [false; 4],
            [true, false, false, false],
            [false, true, false, false],
        ] {
            let mut game = support::Match::dealt_with_humans(
                194,
                direction,
                [&profile, "duck", "duck", "duck"],
                ChargingRules::Classic,
                humans,
            );
            let bot = &game.states[0];
            let state = &game.observations[0];
            let baseline = HeuristicBot.pass(bot, state);
            let expected = if humans[state.phase.pass_receiver(bot.seat).idx()] {
                model.choose(bot, state, baseline, 0.0)
            } else {
                baseline
            };
            assert_eq!(game.players[0].pass(bot, state), expected);
            changed += (expected != baseline) as usize;
        }
    }
    assert!(changed > 0);
}

#[test]
fn production_preplay_is_limited_to_confirmed_rules_and_table_composition() {
    let model = PreplayNetwork::from_bytes(include_bytes!("../../assets/models/preplay-pass2.tpq"))
        .unwrap();
    let mut changes = 0;
    for rules in [
        ChargingRules::Classic,
        ChargingRules::Blind,
        ChargingRules::Bridge,
        ChargingRules::BlindBridge,
        ChargingRules::Chain,
        ChargingRules::BlindChain,
    ] {
        for direction in [
            PassDirection::Left,
            PassDirection::Right,
            PassDirection::Across,
            PassDirection::Keeper,
        ] {
            for humans in [
                [false; 4],
                [true, true, true, false],
                [false, true, true, true],
            ] {
                let mut game = support::Match::dealt_with_humans(
                    194,
                    direction,
                    ["round7:4:8"; 4],
                    rules,
                    humans,
                );
                // Install the production preplay settings after Sit: existing
                // observations, identities and random streams must be retained.
                game.players[0].search = game.players[0]
                    .search
                    .take()
                    .map(|s| s.with_production_preplay());
                if direction == PassDirection::Keeper {
                    // The passing decision follows a unanimous first decline.
                    game.observations[0].phase = GamePhase::PassKeeper;
                }
                let bot = &game.states[0];
                let state = &game.observations[0];
                let baseline = HeuristicBot.pass(bot, state);
                let expected = if rules == ChargingRules::Classic
                    && humans[1..].iter().all(|v| *v)
                    && direction != PassDirection::Keeper
                {
                    model.choose(bot, state, baseline, 0.0)
                } else {
                    baseline
                };
                assert_eq!(game.players[0].pass(bot, state), expected);
                changes += (expected != baseline) as usize;
                let mut old = NeuralNetworkBot::with_config(SearchConfig::default(), 194 * 4);
                assert_eq!(game.players[0].charge(bot, state), old.charge(bot, state));
            }
        }
    }
    assert!(changes > 0);
}

#[test]
fn charging_candidates_and_keeper_redraw_cover_all_rules() {
    let inputs: Vec<_> = Cards::CHARGEABLE
        .into_iter()
        .map(|c| (104 + c.suit().idx() * 13 + c.rank().idx(), 1.0))
        .collect();
    for rules in [
        ChargingRules::Classic,
        ChargingRules::Blind,
        ChargingRules::Bridge,
        ChargingRules::BlindBridge,
        ChargingRules::Chain,
        ChargingRules::BlindChain,
    ] {
        let mut game = support::Match::dealt(194, PassDirection::Keeper, ["round7:4:8"; 4], rules);
        for player in &mut game.players {
            player.search = player.search.take().map(|s| {
                s.with_preplay_models(
                    Some(network(PreplayKind::Pass, &[], 0.0, 0.0)),
                    Some(network(PreplayKind::Charge, &inputs, 0.0, -1.0)),
                    0.0,
                )
            });
        }
        let x = preplay_features(
            &game.states[0],
            &game.observations[0],
            Cards::NONE,
            PreplayKind::Charge,
        );
        assert_eq!((x[162], x[163]), (1.0, 0.0));
        game.charge_from(0);
        assert_eq!(game.game.state.phase, GamePhase::PassKeeper);
        let passes =
            std::array::from_fn(|i| game.players[i].pass(&game.states[i], &game.observations[i]));
        game.apply_passes(passes);
        assert_eq!(game.game.state.phase, GamePhase::ChargeKeeper2);
        let x = preplay_features(
            &game.states[0],
            &game.observations[0],
            Cards::NONE,
            PreplayKind::Charge,
        );
        assert_eq!((x[162], x[163]), (0.0, 1.0));
        let mut all = Cards::NONE;
        for (i, hand) in game.game.post_pass_hand.iter().enumerate() {
            assert_eq!(hand.len(), 13);
            assert!(!all.contains_any(*hand));
            all |= *hand;
            assert_eq!(*hand, game.states[i].post_pass_hand);
        }
        assert_eq!(all, Cards::ALL);
        game.finish_preplay();
        assert_eq!(game.game.state.phase, GamePhase::PlayKeeper);
        assert!(game.game.state.charges.all_charges().is_empty());
        game.finish();
    }
}

#[test]
fn blind_charge_identity_does_not_enter_another_actors_features() {
    for seed in 0..100 {
        let game = support::Match::dealt(
            seed,
            PassDirection::Keeper,
            ["round7:4:8"; 4],
            ChargingRules::Blind,
        );
        for owner in Seat::VALUES {
            let chargeable = game.states[owner.idx()].post_pass_hand & Cards::CHARGEABLE;
            if chargeable.len() < 2 {
                continue;
            }
            let observer = owner.left();
            let mut a = game.clone();
            let mut b = game;
            a.apply(GameEvent::Charge {
                seat: owner,
                cards: chargeable.min().into(),
            });
            b.apply(GameEvent::Charge {
                seat: owner,
                cards: chargeable.max().into(),
            });
            assert_ne!(a.game.state.charges, b.game.state.charges);
            assert_eq!(
                preplay_features(
                    &a.states[observer.idx()],
                    &a.observations[observer.idx()],
                    Cards::NONE,
                    PreplayKind::Charge
                ),
                preplay_features(
                    &b.states[observer.idx()],
                    &b.observations[observer.idx()],
                    Cards::NONE,
                    PreplayKind::Charge
                )
            );
            return;
        }
    }
    panic!("fixture with two chargeable cards not found");
}

#[test]
fn directed_passes_transfer_to_the_correct_seat_and_observation() {
    for direction in [
        PassDirection::Left,
        PassDirection::Right,
        PassDirection::Across,
    ] {
        let mut game =
            support::Match::dealt(194, direction, ["round7:4:8"; 4], ChargingRules::Classic);
        let original = game.states.clone();
        let passes: [Cards; 4] =
            std::array::from_fn(|i| game.players[i].pass(&game.states[i], &game.observations[i]));
        let phase = game.game.state.phase;
        game.apply_passes(passes);
        for from in Seat::VALUES {
            let to = phase.pass_receiver(from);
            let expected =
                (original[to.idx()].pre_pass_hand - passes[to.idx()]) | passes[from.idx()];
            assert_eq!(game.game.post_pass_hand[to.idx()], expected);
            assert_eq!(game.states[to.idx()].post_pass_hand, expected);
            assert_eq!(expected.len(), 13);
        }
    }
}
