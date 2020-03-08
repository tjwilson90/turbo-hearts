use crate::{
    bot::{duck::Duck, gottatry::GottaTry, heuristic::Heuristic, random::Random},
    card::Card,
    cards::Cards,
    error::CardsError,
    game::{event::GameEvent, id::GameId, state::GameState, Games},
    seat::Seat,
    user::UserId,
};
use log::debug;
use rand::distributions::Distribution;
use rand_distr::Gamma;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::{error::TryRecvError, UnboundedReceiver},
    time,
    time::Duration,
};

mod duck;
mod gottatry;
mod heuristic;
mod random;

pub struct Bot {
    game_id: GameId,
    user_id: UserId,
    state: BotState,
    algorithm: Box<dyn Algorithm + Send + Sync>,
}

#[repr(u8)]
#[serde(rename_all = "snake_case")]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Strategy {
    Duck,
    GottaTry,
    Heuristic,
    Random,
}

pub struct BotState {
    seat: Seat,
    pre_pass_hand: Cards,
    post_pass_hand: Cards,
    game: GameState,
}

impl Bot {
    pub fn new(game_id: GameId, user_id: UserId, strategy: Strategy) -> Self {
        let algorithm: Box<dyn Algorithm + Send + Sync> = match strategy {
            Strategy::Duck => Box::new(Duck::new()),
            Strategy::GottaTry => Box::new(GottaTry::new()),
            Strategy::Heuristic => Box::new(Heuristic::new()),
            Strategy::Random => Box::new(Random::new()),
        };
        Self {
            game_id,
            user_id,
            state: BotState {
                seat: Seat::North,
                pre_pass_hand: Cards::NONE,
                post_pass_hand: Cards::NONE,
                game: GameState::new(),
            },
            algorithm,
        }
    }

    pub async fn run(
        mut self,
        games: Games,
        mut rx: UnboundedReceiver<(GameEvent, usize)>,
        delay: Option<Gamma<f32>>,
    ) -> Result<(), CardsError> {
        let mut action = None;
        loop {
            loop {
                match rx.try_recv() {
                    Ok((event, _)) => {
                        action = self.handle(event);
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Closed) => return Ok(()),
                }
            }
            if action.is_some() {
                if let Some(delay) = delay {
                    let seconds = delay.sample(&mut rand::thread_rng());
                    time::delay_for(Duration::from_secs_f32(seconds)).await;
                }
            }
            match action {
                Some(Action::Pass(cards)) => {
                    games.pass_cards(self.game_id, self.user_id, cards).await?
                }
                Some(Action::Charge(cards)) => {
                    games
                        .charge_cards(self.game_id, self.user_id, cards)
                        .await?
                }
                Some(Action::Play(card)) => {
                    let complete = games.play_card(self.game_id, self.user_id, card).await?;
                    if complete {
                        return Ok(());
                    }
                }
                Some(Action::RejectClaim(seat)) => {
                    let _ = games.reject_claim(self.game_id, self.user_id, seat).await;
                }
                None => {}
            }
            match rx.recv().await {
                Some((event, _)) => {
                    action = self.handle(event);
                }
                None => return Ok(()),
            }
        }
    }

    fn handle(&mut self, event: GameEvent) -> Option<Action> {
        debug!(
            "handle: game_id={}, user_id={}, event={:?}",
            self.game_id, self.user_id, event
        );
        self.state.game.apply(&event);
        match &event {
            GameEvent::Sit {
                north,
                east,
                south,
                west,
                ..
            } => {
                self.state.seat = if self.user_id == north.user_id() {
                    Seat::North
                } else if self.user_id == east.user_id() {
                    Seat::East
                } else if self.user_id == south.user_id() {
                    Seat::South
                } else if self.user_id == west.user_id() {
                    Seat::West
                } else {
                    panic!("Bot {} is not a player in the game", self.user_id);
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
