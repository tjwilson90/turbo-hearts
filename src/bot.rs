use crate::{
    bot::{duck::Duck, gottatry::GottaTry, random::Random},
    cards::{Card, Cards, GameState},
    error::CardsError,
    game::event::GameEvent,
    server::Server,
    types::{GameId, Seat, UserId},
};
use log::info;
use rand::distributions::Distribution;
use tokio::{sync::mpsc::error::TryRecvError, time, time::Duration};

mod duck;
mod gottatry;
mod random;

pub struct Bot {
    state: BotState,
    algorithm: Box<dyn Algorithm + Send + Sync>,
}

pub struct BotState {
    user_id: UserId,
    seat: Seat,
    pre_pass_hand: Cards,
    post_pass_hand: Cards,
    game: GameState,
}

impl Bot {
    pub fn new(user_id: UserId, algorithm: &str) -> Self {
        let algorithm: Box<dyn Algorithm + Send + Sync> = match algorithm {
            Duck::NAME => Box::new(Duck::new()),
            GottaTry::NAME => Box::new(GottaTry::new()),
            Random::NAME => Box::new(Random::new()),
            _ => panic!("Unknown algorithm"),
        };
        Self {
            state: BotState {
                user_id,
                seat: Seat::North,
                pre_pass_hand: Cards::NONE,
                post_pass_hand: Cards::NONE,
                game: GameState::new(),
            },
            algorithm,
        }
    }

    pub async fn run(mut self, server: Server, game_id: GameId) -> Result<(), CardsError> {
        let mut rx = server.subscribe_game(game_id, self.state.user_id).await?;
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
            if action.is_some() {
                if let Some(delay) = &server.bot_delay {
                    let seconds = delay.sample(&mut rand::thread_rng());
                    time::delay_for(Duration::from_secs_f32(seconds)).await;
                }
            }
            match action {
                Some(Action::Pass(cards)) => {
                    server
                        .pass_cards(game_id, self.state.user_id, cards)
                        .await?
                }
                Some(Action::Charge(cards)) => {
                    server
                        .charge_cards(game_id, self.state.user_id, cards)
                        .await?
                }
                Some(Action::Play(card)) => {
                    let complete = server.play_card(game_id, self.state.user_id, card).await?;
                    if complete {
                        return Ok(());
                    }
                }
                Some(Action::RejectClaim(seat)) => {
                    let _ = server.reject_claim(game_id, self.state.user_id, seat).await;
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
        info!("Bot {} handling event {:?}", self.state.user_id, event);
        self.state.game.apply(&event);
        match &event {
            GameEvent::Sit {
                north,
                east,
                south,
                west,
                ..
            } => {
                self.state.seat = if self.state.user_id == north.user_id() {
                    Seat::North
                } else if self.state.user_id == east.user_id() {
                    Seat::East
                } else if self.state.user_id == south.user_id() {
                    Seat::South
                } else if self.state.user_id == west.user_id() {
                    Seat::West
                } else {
                    panic!("Bot {} is not a player in the game", self.state.user_id);
                };
            }
            GameEvent::Deal {
                north,
                east,
                south,
                west,
                ..
            } => {
                self.state.pre_pass_hand = *north | *east | *south | *west;
                self.state.post_pass_hand = self.state.pre_pass_hand;
            }
            GameEvent::SendPass { cards, .. } => {
                self.state.post_pass_hand -= *cards;
            }
            GameEvent::RecvPass { cards, .. } => {
                self.state.post_pass_hand |= *cards;
            }
            GameEvent::Claim { seat, .. } => {
                return Some(Action::RejectClaim(*seat));
            }
            _ => {}
        }

        self.algorithm.on_event(&self.state, &event);

        if self.state.game.phase.is_charging() {
            if self.state.game.can_charge(self.state.seat)
                && !self.state.game.done_charging[self.state.seat.idx()]
            {
                Some(Action::Charge(self.algorithm.charge(&self.state)))
            } else {
                None
            }
        } else if self.state.game.phase.is_passing() {
            if !self.state.pre_pass_hand.is_empty()
                && !self.state.game.sent_pass[self.state.seat.idx()]
            {
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
    RejectClaim(Seat),
}

#[allow(unused_variables)]
trait Algorithm {
    fn pass(&mut self, state: &BotState) -> Cards;
    fn charge(&mut self, state: &BotState) -> Cards;
    fn play(&mut self, state: &BotState) -> Card;

    fn on_event(&mut self, state: &BotState, event: &GameEvent);
}
