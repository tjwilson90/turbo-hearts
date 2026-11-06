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
