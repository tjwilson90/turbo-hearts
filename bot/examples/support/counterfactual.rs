//! Continue from an information-set replay; never reuse recorded future moves.
use super::{replay::Replay, support::Player};
use turbo_hearts_api::{should_claim, Card, GameEvent, Seat};
use turbo_hearts_bot::Algorithm;

pub fn finish(mut replay: Replay, mut players: [Player; 4], card: Card) -> [i16; 4] {
    let seat = replay.game.state.next_actor.unwrap();
    replay.apply(&GameEvent::Play { seat, card }, |own, state, event| {
        players[own.seat.idx()].on_event(own, state, event)
    });
    while replay.game.state.phase.is_playing() {
        let seat = replay.game.state.next_actor.unwrap();
        let own = &replay.states[seat.idx()];
        let state = &replay.observations[seat.idx()];
        if should_claim(state, own.void, seat, own.post_pass_hand) {
            let event = GameEvent::Claim {
                seat,
                hand: own.post_pass_hand - state.played,
            };
            replay.apply(&event, |own, state, event| {
                players[own.seat.idx()].on_event(own, state, event)
            });
            for acceptor in Seat::VALUES {
                if acceptor != seat {
                    replay.apply(
                        &GameEvent::AcceptClaim {
                            claimer: seat,
                            acceptor,
                        },
                        |own, state, event| players[own.seat.idx()].on_event(own, state, event),
                    );
                }
            }
        } else {
            let card = players[seat.idx()].play(own, state);
            replay.apply(&GameEvent::Play { seat, card }, |own, state, event| {
                players[own.seat.idx()].on_event(own, state, event)
            });
        }
    }
    Seat::VALUES.map(|s| replay.game.state.scores().money(s))
}
