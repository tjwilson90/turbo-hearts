use log::LevelFilter;
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
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Info)
        .filter_module("turbo_hearts_bot", LevelFilter::Debug)
        .is_test(true)
        .try_init();
    let mut state = State::new(Seat::South, PassDirection::Across);
    state.deal([
        "J43S 8765H 83D AK74C".parse().unwrap(),
        "KT82S KQ942H 42D 63C".parse().unwrap(),
        "Q65S T3H JT65D Q852C".parse().unwrap(),
        "A97S AJH AKQ97D JT9C".parse().unwrap(),
    ]);
    state.send_pass(Seat::North, "83D AC".parse().unwrap());
    state.send_pass(Seat::East, "KS KQH".parse().unwrap());
    state.send_pass(Seat::South, "QS T3H".parse().unwrap());
    state.send_pass(Seat::West, "7S JH JC".parse().unwrap());

    state.recv_pass(Seat::North, "QS T3H".parse().unwrap());
    state.recv_pass(Seat::East, "7S JH JC".parse().unwrap());
    state.recv_pass(Seat::South, "83D AC".parse().unwrap());
    state.recv_pass(Seat::West, "KS KQH".parse().unwrap());

    state.charge(Seat::West, "TC".parse().unwrap());
    state.charge(Seat::North, "".parse().unwrap());
    state.charge(Seat::East, "".parse().unwrap());
    state.charge(Seat::South, "".parse().unwrap());

    for c in &["2C", "9C", "7C", "6C", "8C", "TC", "4C", "3C"] {
        state.play(c.parse().unwrap());
    }
    for c in &["AD", "QS", "4D", "3D"] {
        state.play(c.parse().unwrap());
    }
    for c in &["AS", "JS", "8S", "6S"] {
        state.play(c.parse().unwrap());
    }
    for c in &["KD", "KC", "2D", "TD"] {
        state.play(c.parse().unwrap());
    }
    for c in &["9D", "4S", "JC", "5D", "QD", "3S", "7S", "6D"] {
        state.play(c.parse().unwrap());
    }
    for c in &["KS", "8H", "2S", "5S"] {
        state.play(c.parse().unwrap());
    }
    for c in &["AH", "7H", "4H", "AC"] {
        state.play(c.parse().unwrap());
    }
    for c in &["KH", "6H", "2H", "8D"] {
        state.play(c.parse().unwrap());
    }
    for c in &["QH", "5H", "JH"] {
        state.play(c.parse().unwrap());
    }
    println!("{}", state.bot.play(&state.bot_state, &state.game.state));
}
