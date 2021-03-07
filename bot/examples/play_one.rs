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
        "AJ94S 762H KQ5D K97C".parse().unwrap(),
        "Q85S QT43H A64D 863C".parse().unwrap(),
        "72S 985H 98732D QJ4C".parse().unwrap(),
        "KT63S AKJH JTD AT52C".parse().unwrap(),
    ]);
    state.send_pass(Seat::North, "76H KC".parse().unwrap());
    state.send_pass(Seat::East, "QS QTH".parse().unwrap());
    state.send_pass(Seat::South, "8H QJC".parse().unwrap());
    state.send_pass(Seat::West, "KS KH TC".parse().unwrap());

    state.recv_pass(Seat::North, "KS KH TC".parse().unwrap());
    state.recv_pass(Seat::East, "76H KC".parse().unwrap());
    state.recv_pass(Seat::South, "QS QTH".parse().unwrap());
    state.recv_pass(Seat::West, "8H QJC".parse().unwrap());

    state.charge(Seat::North, "".parse().unwrap());
    state.charge(Seat::East, "".parse().unwrap());
    state.charge(Seat::South, "".parse().unwrap());
    state.charge(Seat::West, "".parse().unwrap());

    for c in &["2C", "9C", "6C", "4C", "QC", "TC", "3C", "7D"] {
        state.play(c.parse().unwrap());
    }
    println!("{}", state.bot.play(&state.bot_state, &state.game.state));
}
