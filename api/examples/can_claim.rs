use rand::{prelude::IndexedRandom, RngExt};
use std::time::Instant;
use turbo_hearts_api::{can_claim, Cards, GameState, Seat, VoidState};

fn main() {
    let mut rng = rand::rng();
    let mut max = 0;
    let deck = Cards::ALL.into_iter().collect::<Vec<_>>();
    let state = GameState {
        next_actor: Some(Seat::North),
        ..GameState::new()
    };
    for i in 0u64.. {
        let seat = Seat::VALUES[rng.random_range(0..4)];
        let hand = deck.sample(&mut rng, 13).cloned().collect();
        let now = Instant::now();
        can_claim(&state, VoidState::new(), seat, hand);
        let elapsed = now.elapsed().as_nanos();
        if elapsed > max {
            max = elapsed;
            println!(
                "iter = {}, nanos = {}, seat = {}, hand = {}, ",
                i, max, seat, hand
            );
        }
    }
}
