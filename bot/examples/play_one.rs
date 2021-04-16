use log::LevelFilter;
use turbo_hearts_api::{
    BotState, Card, Cards, ChargingRules, Game, GameEvent, GamePhase, PassDirection, Seat,
};
use turbo_hearts_bot::{Algorithm, NeuralNetworkBot};

#[derive(Clone)]
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
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Info)
        .filter_module("turbo_hearts_bot", LevelFilter::Debug)
        .is_test(true)
        .try_init();
    let mut state = State::new(Seat::North, PassDirection::Right);
    state.deal([
        "QT64S Q982H K74D 86C".parse().unwrap(),
        "A975S J653H AJ3D AJC".parse().unwrap(),
        "KH 9852D QT975432C".parse().unwrap(),
        "KJ832S AT74H QT6D KC".parse().unwrap(),
    ]);
    state.send_pass(Seat::North, "QS 9H 8C".parse().unwrap());
    state.send_pass(Seat::East, "AS AJC".parse().unwrap());
    state.send_pass(Seat::South, "852D".parse().unwrap());
    state.send_pass(Seat::West, "TH TD KC".parse().unwrap());

    state.recv_pass(Seat::North, "AS AJC".parse().unwrap());
    state.recv_pass(Seat::East, "852D".parse().unwrap());
    state.recv_pass(Seat::South, "TH TD KC".parse().unwrap());
    state.recv_pass(Seat::West, "QS 9H 8C".parse().unwrap());

    state.charge(Seat::South, "TC".parse().unwrap());
    state.charge(Seat::West, "QS".parse().unwrap());
    state.charge(Seat::North, "".parse().unwrap());
    state.charge(Seat::East, "".parse().unwrap());
    state.charge(Seat::South, "".parse().unwrap());

    for c in &["2C", "8C", "AC", "7S"] {
        state.play(c.parse().unwrap());
    }
    for c in &["4S", "5S", "9D", "JS"] {
        state.play(c.parse().unwrap());
    }
    for c in &["8S", "6S", "9S", "KH", "3S"] {
        state.play(c.parse().unwrap());
    }
    println!("{}", state.bot.play(&state.bot_state, &state.game.state));
}
