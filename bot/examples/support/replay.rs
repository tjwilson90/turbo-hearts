//! Replay recorded events through the same redaction path used by live bots.
use turbo_hearts_api::{BotState, Cards, Game, GameEvent, GameState, Seat};

#[derive(Clone)]
pub struct Replay {
    pub game: Game<()>,
    pub states: [BotState; 4],
    pub observations: [GameState; 4],
}

impl Replay {
    pub fn new() -> Self {
        Self {
            game: Game::new(),
            states: Seat::VALUES.map(|s| BotState::new(s, Cards::NONE)),
            observations: std::array::from_fn(|_| GameState::new()),
        }
    }

    pub fn apply(
        &mut self,
        event: &GameEvent,
        mut observe: impl FnMut(&BotState, &GameState, &GameEvent),
    ) {
        let Self {
            game,
            states,
            observations,
        } = self;
        if let GameEvent::Play { seat, card } = event {
            assert_eq!(game.state.next_actor, Some(*seat));
            assert!(game
                .state
                .legal_plays(states[seat.idx()].post_pass_hand)
                .contains(*card));
        }
        game.apply(event, |_, event| {
            for i in 0..4 {
                let event = event.redact(Some(Seat::VALUES[i]), observations[i].rules);
                let bot = &mut states[i];
                match &event {
                    GameEvent::Deal {
                        north,
                        east,
                        south,
                        west,
                        ..
                    } => {
                        bot.pre_pass_hand = [*north, *east, *south, *west][i];
                        bot.post_pass_hand = bot.pre_pass_hand;
                    }
                    GameEvent::SendPass { from, cards } if from.idx() == i => {
                        bot.post_pass_hand -= *cards
                    }
                    GameEvent::RecvPass { to, cards } if to.idx() == i => {
                        bot.post_pass_hand |= *cards
                    }
                    _ => {}
                }
                bot.on_event(&observations[i], &event);
                observe(bot, &observations[i], &event);
                observations[i].apply(&event);
            }
        });
    }
}
