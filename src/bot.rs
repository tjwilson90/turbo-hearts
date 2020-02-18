use crate::{
    bot::{duck::Duck, random::Random},
    cards::{Card, Cards, GameState},
    error::CardsError,
    game::GameEvent,
    server::Server,
    types::{GameId, Seat},
};
use log::info;
use rand::Rng;
use tokio::sync::mpsc::error::TryRecvError;

mod duck;
mod random;

static NAMES: &[&str] = &include!("../data/names.json");

pub fn name() -> String {
    let mut rng = rand::thread_rng();
    let initial = ('a' as u8 + rng.gen_range(0, 26)) as char;
    let mut name = initial.to_string();
    name.push_str(NAMES[rng.gen_range(0, NAMES.len())]);
    name.push_str(" (bot)");
    name
}

pub struct Bot {
    state: BotState,
    algorithm: Box<dyn Algorithm + Send + Sync>,
}

pub struct BotState {
    name: String,
    seat: Seat,
    pre_pass_hand: Cards,
    post_pass_hand: Cards,
    game: GameState,
}

impl Bot {
    pub fn new(name: String, algorithm: &str) -> Self {
        let algorithm: Box<dyn Algorithm + Send + Sync> = match algorithm {
            Random::NAME => Box::new(Random::new()),
            Duck::NAME => Box::new(Duck::new()),
            _ => panic!("Unknown algorithm"),
        };
        Self {
            state: BotState {
                name,
                seat: Seat::North,
                pre_pass_hand: Cards::NONE,
                post_pass_hand: Cards::NONE,
                game: GameState::new(),
            },
            algorithm,
        }
    }

    pub async fn run(mut self, server: Server, id: GameId) -> Result<(), CardsError> {
        let mut rx = server.subscribe_game(id, self.state.name.clone()).await?;
        let mut action = None;
        loop {
            loop {
                match rx.try_recv() {
                    Ok(event) => {
                        action = self.handle(event);
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Closed) => return Ok(()),
                }
            }
            match action {
                Some(Action::Pass(cards)) => server.pass_cards(id, &self.state.name, cards).await?,
                Some(Action::Charge(cards)) => {
                    server.charge_cards(id, &self.state.name, cards).await?
                }
                Some(Action::Play(card)) => {
                    let complete = server.play_card(id, &self.state.name, card).await?;
                    if complete {
                        return Ok(());
                    }
                }
                None => {}
            }
            match rx.recv().await {
                Some(event) => {
                    action = self.handle(event);
                }
                None => break,
            }
        }

        Ok(())
    }

    fn handle(&mut self, event: GameEvent) -> Option<Action> {
        info!("{} handling event {:?}", self.state.name, event);
        self.state.game.apply(&event);
        match event {
            GameEvent::Sit {
                north,
                east,
                south,
                west,
                ..
            } => {
                self.state.seat = if &self.state.name == north.name() {
                    Seat::North
                } else if &self.state.name == east.name() {
                    Seat::East
                } else if &self.state.name == south.name() {
                    Seat::South
                } else if &self.state.name == west.name() {
                    Seat::West
                } else {
                    panic!("{} is not a player in the game", self.state.name);
                };
                self.algorithm.on_sit(&self.state);
            }
            GameEvent::Deal {
                north,
                east,
                south,
                west,
                ..
            } => {
                self.state.pre_pass_hand = north | east | south | west;
                self.state.post_pass_hand = self.state.pre_pass_hand;
                self.algorithm.on_deal(&self.state);
            }
            GameEvent::SendPass { from, cards } => {
                self.state.post_pass_hand -= cards;
                self.algorithm.on_send_pass(&self.state, from, cards);
            }
            GameEvent::RecvPass { to, cards } => {
                self.state.post_pass_hand |= cards;
                self.algorithm.on_recv_pass(&self.state, to, cards);
            }
            GameEvent::BlindCharge { seat, count } => {
                self.algorithm.on_blind_charge(&self.state, seat, count);
            }
            GameEvent::Charge { seat, cards } => {
                self.algorithm.on_charge(&self.state, seat, cards);
            }
            GameEvent::RevealCharges { .. } => {
                self.algorithm.on_reveal_charges(&self.state);
            }
            GameEvent::Play { seat, card } => {
                self.algorithm.on_play(&self.state, seat, card);
            }
            _ => {}
        }

        if self.state.game.phase.is_charging() {
            if self.state.game.can_charge(self.state.seat)
                && !self.state.game.done_charging[self.state.seat.idx()]
            {
                Some(Action::Charge(self.algorithm.charge(&self.state)))
            } else {
                None
            }
        } else if self.state.game.phase.is_passing() {
            if !self.state.game.sent_pass[self.state.seat.idx()] {
                Some(Action::Pass(self.algorithm.pass(&self.state)))
            } else {
                None
            }
        } else if self.state.game.phase.is_playing() {
            if (self.state.post_pass_hand - self.state.game.played).contains(Card::TwoClubs)
                || Some(self.state.seat) == self.state.game.next_player
            {
                Some(Action::Play(self.algorithm.play(&self.state)))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Action {
    Pass(Cards),
    Charge(Cards),
    Play(Card),
}

#[allow(unused_variables)]
trait Algorithm {
    fn pass(&mut self, state: &BotState) -> Cards;
    fn charge(&mut self, state: &BotState) -> Cards;
    fn play(&mut self, state: &BotState) -> Card;

    fn on_sit(&mut self, state: &BotState) {}
    fn on_deal(&mut self, state: &BotState) {}
    fn on_send_pass(&mut self, state: &BotState, from: Seat, cards: Cards) {}
    fn on_recv_pass(&mut self, state: &BotState, to: Seat, cards: Cards) {}
    fn on_charge(&mut self, state: &BotState, seat: Seat, cards: Cards) {}
    fn on_blind_charge(&mut self, state: &BotState, seat: Seat, count: usize) {}
    fn on_reveal_charges(&mut self, state: &BotState) {}
    fn on_play(&mut self, state: &BotState, seat: Seat, card: Card) {}
}
