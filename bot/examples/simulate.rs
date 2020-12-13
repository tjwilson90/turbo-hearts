use rand::seq::SliceRandom;
use turbo_hearts_api::{
    BotState, Card, Cards, ChargingRules, GameEvent, GameState, PassDirection, Seat,
};
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

struct Game {
    state: GameState,
    bot_states: [BotState; 4],
    bots: [NeuralNetworkBot; 4],
}

impl Game {
    fn new() -> Self {
        let mut deck = Cards::ALL.into_iter().collect::<Vec<_>>();
        deck.shuffle(&mut rand::thread_rng());
        Self {
            state: GameState::new(),
            bot_states: [
                bot(Seat::North, &deck),
                bot(Seat::East, &deck),
                bot(Seat::South, &deck),
                bot(Seat::West, &deck),
            ],
            bots: [
                NeuralNetworkBot::new(),
                NeuralNetworkBot::new(),
                NeuralNetworkBot::new(),
                NeuralNetworkBot::new(),
            ],
        }
    }

    fn apply(&mut self, event: &GameEvent) {
        self.state.apply(event);
        for &s in &Seat::VALUES {
            self.bots[s.idx()].on_event(
                &self.bot_states[s.idx()],
                &self.state,
                &event.redact(Some(s), ChargingRules::Classic),
            );
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut game = Game::new();
    game.apply(&GameEvent::Deal {
        north: game.bot_states[0].pre_pass_hand,
        east: game.bot_states[1].pre_pass_hand,
        south: game.bot_states[2].pre_pass_hand,
        west: game.bot_states[3].pre_pass_hand,
        pass: PassDirection::Left,
    });
    for &s in &Seat::VALUES {
        let pass = game.bots[s.idx()].pass(&game.bot_states[s.idx()], &game.state);
        game.apply(&GameEvent::SendPass {
            from: s,
            cards: pass,
        });
        game.bot_states[s.idx()].post_pass_hand -= pass;
        println!(
            "{} passes {} from {}",
            s,
            pass,
            game.bot_states[s.idx()].pre_pass_hand
        );
    }
    for &s in &Seat::VALUES {
        let passer = &game.bot_states[s.right().idx()];
        let pass = passer.pre_pass_hand - passer.post_pass_hand;
        game.apply(&GameEvent::RecvPass { to: s, cards: pass });
        game.bot_states[s.idx()].post_pass_hand |= pass;
    }
    println!("-------------------------------");
    println!("After Pass");
    println!("North {}", game.bot_states[0].post_pass_hand);
    println!("East  {}", game.bot_states[1].post_pass_hand);
    println!("South {}", game.bot_states[2].post_pass_hand);
    println!("West  {}", game.bot_states[3].post_pass_hand);
    println!("-------------------------------");
    while game.state.phase.is_charging() {
        for &s in &Seat::VALUES {
            if game.state.phase.is_charging() && !game.state.done.charged(s) {
                let cards = game.bots[s.idx()].charge(&game.bot_states[s.idx()], &game.state);
                game.apply(&GameEvent::Charge { seat: s, cards });
                println!(
                    "{} charges {} from {}",
                    s,
                    cards,
                    game.bot_states[s.idx()].post_pass_hand
                );
            }
        }
    }
    game.state.next_actor = game.bot_states.iter().find_map(|bot| {
        if bot.post_pass_hand.contains(Card::TwoClubs) {
            Some(bot.seat)
        } else {
            None
        }
    });
    let seat = game.state.next_actor.unwrap();
    game.apply(&GameEvent::StartTrick { leader: seat });
    while game.state.played != Cards::ALL {
        if game.state.current_trick.is_empty() {
            println!("-------------------------------");
        }
        let seat = game.state.next_actor.unwrap();
        let card = game.bots[seat.idx()].play(&game.bot_states[seat.idx()], &game.state);
        println!(
            "{} plays {} from {}",
            seat,
            card,
            game.bot_states[seat.idx()].post_pass_hand - game.state.played
        );
        game.apply(&GameEvent::Play { seat, card });
    }
    println!("-------------------------------");
    let scores = game.state.won.scores(game.state.charges);
    for &s in &Seat::VALUES {
        println!(
            "{}: score {}, money {}",
            s,
            scores.score(s),
            scores.money(s)
        );
    }
}
