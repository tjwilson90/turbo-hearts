use rand::seq::SliceRandom;
use turbo_hearts_api::{
    BotState, Card, Cards, GameEvent, GameState, PassDirection, Seat, WonState,
};
use turbo_hearts_bot::{Algorithm, BruteForce, HeuristicBot};

fn bot(seat: Seat, deck: &[Card]) -> BotState {
    let hand = deck[13 * seat.idx()..13 * (seat.idx() + 1)]
        .iter()
        .cloned()
        .collect();
    BotState {
        seat,
        pre_pass_hand: hand,
        post_pass_hand: hand,
    }
}

fn main() {
    env_logger::init();
    let mut deck = Cards::ALL.into_iter().collect::<Vec<_>>();
    deck.shuffle(&mut rand::thread_rng());
    let mut bots = [
        bot(Seat::North, &deck),
        bot(Seat::East, &deck),
        bot(Seat::South, &deck),
        bot(Seat::West, &deck),
    ];
    let mut heuristic = HeuristicBot::new();
    let mut state = GameState::new();
    state.apply(&GameEvent::Deal {
        north: bots[0].pre_pass_hand,
        east: bots[1].pre_pass_hand,
        south: bots[2].pre_pass_hand,
        west: bots[3].pre_pass_hand,
        pass: PassDirection::Left,
    });
    for bot in &mut bots {
        let pass = heuristic.pass(&bot, &state);
        state.apply(&GameEvent::SendPass {
            from: bot.seat,
            cards: pass,
        });
        bot.post_pass_hand -= pass;
    }
    for i in 0..4 {
        let passer = &bots[bots[i].seat.right().idx()];
        let pass = passer.pre_pass_hand - passer.post_pass_hand;
        state.apply(&GameEvent::RecvPass {
            to: bots[i].seat,
            cards: pass,
        });
        bots[i].post_pass_hand |= pass;
    }
    while state.phase.is_charging() {
        for bot in &bots {
            if state.can_charge(bot.seat) {
                let cards = heuristic.charge(&bot, &state);
                state.apply(&GameEvent::Charge {
                    seat: bot.seat,
                    cards,
                });
            }
        }
    }
    state.next_actor = bots.iter().find_map(|bot| {
        if bot.post_pass_hand.contains(Card::TwoClubs) {
            Some(bot.seat)
        } else {
            None
        }
    });
    for _ in 0..28 {
        let seat = state.next_actor.unwrap();
        let bot = &bots[seat.idx()];
        let card = heuristic.play(bot, &state);
        state.apply(&GameEvent::Play { seat, card });
    }
    let mut brute_force = BruteForce::new([
        bots[0].post_pass_hand,
        bots[1].post_pass_hand,
        bots[2].post_pass_hand,
        bots[3].post_pass_hand,
    ]);
    while state.played.len() < 48 {
        let seat = state.next_actor.unwrap();
        let mut best_card = Card::TwoClubs;
        let mut best_won = WonState::new();
        let mut best_money = i16::MIN;
        for card in state
            .legal_plays(bots[seat.idx()].post_pass_hand)
            .distinct_plays(state.played, state.current_trick)
        {
            let mut state = state.clone();
            state.apply(&GameEvent::Play { seat, card });
            let won = brute_force.solve(&mut state);
            let money = won.scores(state.charges).money(seat);
            if money > best_money {
                best_card = card;
                best_won = won;
                best_money = money;
            }
        }
        println!("north {}", bots[0].post_pass_hand - state.played);
        println!("east  {}", bots[1].post_pass_hand - state.played);
        println!("south {}", bots[2].post_pass_hand - state.played);
        println!("west  {}", bots[3].post_pass_hand - state.played);
        println!("{}, {}, {:?}", seat, best_card, best_won);
        state.apply(&GameEvent::Play {
            seat,
            card: best_card,
        });
    }
}
