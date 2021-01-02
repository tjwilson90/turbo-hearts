use turbo_hearts_api::{
    BotState, Card, Cards, ChargingRules, Game, GameEvent, GamePhase, PassDirection, Seat,
};
use turbo_hearts_bot::{Algorithm, NeuralNetworkBot};

struct State {
    game: Game<()>,
    bot_state: BotState,
    bot: NeuralNetworkBot,
}

impl State {
    fn new(seat: Seat, dir: PassDirection) -> State {
        let mut game = Game::new();
        match dir {
            PassDirection::Left => {}
            PassDirection::Right => game.state.phase = GamePhase::PassRight,
            PassDirection::Across => game.state.phase = GamePhase::PassAcross,
            PassDirection::Keeper => game.state.phase = GamePhase::ChargeKeeper1,
        };
        Self {
            game,
            bot_state: BotState {
                seat,
                pre_pass_hand: Cards::NONE,
                post_pass_hand: Cards::NONE,
            },
            bot: NeuralNetworkBot::new(),
        }
    }

    fn deal(&mut self, hands: [Cards; 4]) {
        self.apply(&GameEvent::Deal {
            north: hands[0],
            east: hands[1],
            south: hands[2],
            west: hands[3],
            pass: self.game.state.phase.direction(),
        });
        self.bot_state.pre_pass_hand = hands[self.bot_state.seat.idx()];
        self.bot_state.post_pass_hand = hands[self.bot_state.seat.idx()];
    }

    fn send_pass(&mut self, from: Seat, cards: Cards) {
        self.apply(&GameEvent::SendPass { from, cards });
        if from == self.bot_state.seat {
            self.bot_state.post_pass_hand -= cards;
        }
    }

    fn recv_pass(&mut self, to: Seat, cards: Cards) {
        self.apply(&GameEvent::RecvPass { to, cards });
        if to == self.bot_state.seat {
            self.bot_state.post_pass_hand |= cards;
        }
    }

    fn charge(&mut self, seat: Seat, cards: Cards) {
        self.apply(&GameEvent::Charge { seat, cards });
    }

    fn play(&mut self, card: Card) {
        self.apply(&GameEvent::Play {
            seat: self.game.state.next_actor.unwrap(),
            card,
        });
    }

    fn apply(&mut self, event: &GameEvent) {
        let (game, bot_state, bot) = (&mut self.game, &self.bot_state, &mut self.bot);
        game.apply(event, |g, e| {
            bot.on_event(
                &bot_state,
                &g.state,
                &e.redact(Some(bot_state.seat), ChargingRules::Classic),
            )
        });
    }
}

fn main() {
    env_logger::init();
    let mut state = State::new(Seat::West, PassDirection::Left);
    state.deal([
        "AJ3S Q9873H 853D 75C".parse().unwrap(),
        "852S J6H QJD KQJ864C".parse().unwrap(),
        "9764S AK42H TD AT93C".parse().unwrap(),
        "KQTS T5H AK97642D 2C".parse().unwrap(),
    ]);
    state.send_pass(Seat::North, "AS 9H 7C".parse().unwrap());
    state.send_pass(Seat::East, "J6H QD".parse().unwrap());
    state.send_pass(Seat::South, "KH TD TC".parse().unwrap());
    state.send_pass(Seat::West, "T5H 2C".parse().unwrap());

    state.recv_pass(Seat::North, "T5H 2C".parse().unwrap());
    state.recv_pass(Seat::East, "AS 9H 7C".parse().unwrap());
    state.recv_pass(Seat::South, "J6H QD".parse().unwrap());
    state.recv_pass(Seat::West, "KH TD TC".parse().unwrap());

    state.charge(Seat::North, "".parse().unwrap());
    state.charge(Seat::East, "".parse().unwrap());
    state.charge(Seat::South, "AH".parse().unwrap());
    state.charge(Seat::West, "".parse().unwrap());
    state.charge(Seat::North, "".parse().unwrap());
    state.charge(Seat::East, "".parse().unwrap());

    for c in &["2C", "4C", "9C", "TC", "5C", "7C", "3C", "TD"] {
        state.play(c.parse().unwrap());
    }
    for c in &["9D", "8D", "JD", "QD", "KD", "5D", "AS", "AH"] {
        state.play(c.parse().unwrap());
    }
    for c in &["KS", "JS", "5S", "9S", "TS", "3S", "8S", "6S"] {
        state.play(c.parse().unwrap());
    }
    println!("{}", state.bot.play(&state.bot_state, &state.game.state));
}
