use rand::seq::SliceRandom;
use turbo_hearts_api::{BotState, Card, Cards, GameEvent, GameState, PassDirection, Seat};
use turbo_hearts_bot::{Algorithm, NeuralNetworkBot};

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

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut deck = Cards::ALL.into_iter().collect::<Vec<_>>();
    deck.shuffle(&mut rand::thread_rng());
    let mut sim = NeuralNetworkBot::new();
    let mut bots = [
        bot(Seat::North, &deck),
        bot(Seat::East, &deck),
        bot(Seat::South, &deck),
        bot(Seat::West, &deck),
    ];
    let mut state = GameState::new();
    state.apply(&GameEvent::Deal {
        north: bots[0].pre_pass_hand,
        east: bots[1].pre_pass_hand,
        south: bots[2].pre_pass_hand,
        west: bots[3].pre_pass_hand,
        pass: PassDirection::Left,
    });
    for bot in &mut bots {
        let pass = sim.pass(&bot, &state);
        state.apply(&GameEvent::SendPass {
            from: bot.seat,
            cards: pass,
        });
        bot.post_pass_hand -= pass;
        println!("{} passes {} from {}", bot.seat, pass, bot.pre_pass_hand);
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
    println!("-------------------------------");
    println!("After Pass");
    println!("North {}", bots[0].post_pass_hand);
    println!("East  {}", bots[1].post_pass_hand);
    println!("South {}", bots[2].post_pass_hand);
    println!("West  {}", bots[3].post_pass_hand);
    println!("-------------------------------");
    while state.phase.is_charging() {
        for bot in &bots {
            if state.phase.is_charging() && !state.done.charged(bot.seat) {
                let cards = sim.charge(&bot, &state);
                state.apply(&GameEvent::Charge {
                    seat: bot.seat,
                    cards,
                });
                println!("{} charges {} from {}", bot.seat, cards, bot.post_pass_hand);
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
    while state.played != Cards::ALL {
        if state.current_trick.is_empty() {
            println!("-------------------------------");
        }
        let seat = state.next_actor.unwrap();
        let bot = &bots[seat.idx()];
        let card = sim.play(bot, &state);
        println!(
            "{} plays {} from {}",
            bot.seat,
            card,
            bot.post_pass_hand - state.played
        );
        let event = GameEvent::Play { seat, card };
        state.apply(&event);
        sim.on_event(&bot, &state, &event);
    }
    println!("-------------------------------");
    let scores = state.won.scores(state.charges.all_charges());
    for &s in &Seat::VALUES {
        println!(
            "{}: score {}, money {}",
            s,
            scores.score(s),
            scores.money(s)
        );
    }
}
