use crate::Games;
use log::debug;
use rand::distributions::Distribution;
use rand_distr::Gamma;
use std::time::Instant;
use tokio::{
    sync::mpsc::{error::TryRecvError, UnboundedReceiver},
    time,
    time::Duration,
};
use turbo_hearts_api::{
    can_claim, BotState, BotStrategy, Card, Cards, CardsError, GameEvent, GameId, GameState, Seat,
    UserId,
};
use turbo_hearts_bot::{Bot, DuckBot, GottaTryBot, HeuristicBot, RandomBot, SimulateBot};

macro_rules! dispatch {
    ($self:expr, $fun:ident, $($arg:expr),*) => {
        match &mut $self {
            Bot::Duck(bot) => bot.$fun($($arg),+),
            Bot::GottaTry(bot) => bot.$fun($($arg),+),
            Bot::Heuristic(bot) => bot.$fun($($arg),+),
            Bot::Random(bot) => bot.$fun($($arg),+),
            Bot::Simulate(bot) => bot.$fun($($arg),+),
        }
    };
}

macro_rules! dispatch_async {
    ($self:expr, $fun:ident, $($arg:expr),*) => {
        match &mut $self {
            Bot::Duck(bot) => bot.$fun($($arg),+).await,
            Bot::GottaTry(bot) => bot.$fun($($arg),+).await,
            Bot::Heuristic(bot) => bot.$fun($($arg),+).await,
            Bot::Random(bot) => bot.$fun($($arg),+).await,
            Bot::Simulate(bot) => bot.$fun($($arg),+).await,
        }
    };
}

pub struct BotRunner {
    game_id: GameId,
    user_id: UserId,
    bot_state: BotState,
    game_state: GameState,
    claim_hands: [Cards; 4],
    bot: Bot,
}

impl BotRunner {
    pub fn new(game_id: GameId, user_id: UserId, strategy: BotStrategy) -> Self {
        let bot = match strategy {
            BotStrategy::Duck => Bot::Duck(DuckBot::new()),
            BotStrategy::GottaTry => Bot::GottaTry(GottaTryBot::new()),
            BotStrategy::Heuristic => Bot::Heuristic(HeuristicBot::new()),
            BotStrategy::Random => Bot::Random(RandomBot::new()),
            BotStrategy::Simulate => Bot::Simulate(SimulateBot::new()),
        };
        Self {
            game_id,
            user_id,
            bot_state: BotState {
                seat: Seat::North,
                pre_pass_hand: Cards::NONE,
                post_pass_hand: Cards::NONE,
            },
            game_state: GameState::new(),
            claim_hands: [Cards::NONE; 4],
            bot,
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
            let now = Instant::now();
            loop {
                match rx.try_recv() {
                    Ok((event, _)) => {
                        action = self.handle(event);
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Closed) => return Ok(()),
                }
            }
            let delay =
                delay.map(|delay| Duration::from_secs_f32(delay.sample(&mut rand::thread_rng())));
            for &seat in &Seat::VALUES {
                if seat != self.bot_state.seat
                    && self.game_state.claims.is_claiming(seat)
                    && !self
                        .game_state
                        .claims
                        .has_accepted(seat, self.bot_state.seat)
                {
                    let accept = can_claim(
                        seat,
                        self.claim_hands[seat.idx()] - self.game_state.played,
                        &self.game_state,
                    );
                    BotRunner::delay(delay, now).await;
                    if accept {
                        match games.accept_claim(self.game_id, self.user_id, seat).await {
                            Ok(true) => return Ok(()),
                            _ => {}
                        }
                    } else {
                        let _ = games.reject_claim(self.game_id, self.user_id, seat).await;
                    }
                }
                self.claim_hands[seat.idx()] = Cards::NONE;
            }
            match action {
                Some(Action::Pass) => {
                    let cards = dispatch_async!(self.bot, pass, &self.bot_state, &self.game_state);
                    BotRunner::delay(delay, now).await;
                    let _ = games.pass_cards(self.game_id, self.user_id, cards).await;
                }
                Some(Action::Charge) => {
                    let cards =
                        dispatch_async!(self.bot, charge, &self.bot_state, &self.game_state);
                    BotRunner::delay(delay, now).await;
                    let _ = games.charge_cards(self.game_id, self.user_id, cards).await;
                }
                Some(Action::Play) => {
                    let card = dispatch_async!(self.bot, play, &self.bot_state, &self.game_state);
                    if card != Card::TwoClubs {
                        BotRunner::delay(delay, now).await;
                    }
                    match games.play_card(self.game_id, self.user_id, card).await {
                        Ok(true) => return Ok(()),
                        _ => {}
                    }
                }
                Some(Action::Claim) => {
                    BotRunner::delay(delay, now).await;
                    let _ = games.claim(self.game_id, self.user_id).await;
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

    async fn delay(delay: Option<Duration>, start: Instant) {
        let delay = delay.and_then(|delay| delay.checked_sub(start.elapsed()));
        if let Some(delay) = delay {
            time::delay_for(delay).await;
        }
    }

    fn handle(&mut self, event: GameEvent) -> Option<Action> {
        debug!(
            "handle: game_id={}, user_id={}, event={:?}",
            self.game_id, self.user_id, event
        );
        dispatch!(
            self.bot,
            on_event,
            &self.bot_state,
            &self.game_state,
            &event
        );
        let phase = self.game_state.phase;
        self.game_state.apply(&event);
        if phase.is_playing() && !self.game_state.phase.is_playing() {
            self.bot_state.pre_pass_hand = Cards::NONE;
            self.bot_state.post_pass_hand = Cards::NONE;
        }
        match &event {
            GameEvent::Sit {
                north,
                east,
                south,
                west,
                ..
            } => {
                self.bot_state.seat = if self.user_id == north.user_id() {
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
                self.bot_state.pre_pass_hand = *north | *east | *south | *west;
                self.bot_state.post_pass_hand = self.bot_state.pre_pass_hand;
            }
            GameEvent::SendPass { cards, .. } => {
                self.bot_state.post_pass_hand -= *cards;
            }
            GameEvent::RecvPass { cards, .. } => {
                self.bot_state.post_pass_hand |= *cards;
            }
            GameEvent::Claim { seat, hand } => {
                self.claim_hands[seat.idx()] = *hand;
            }
            _ => {}
        }

        if self.game_state.phase.is_charging() {
            if !self.bot_state.pre_pass_hand.is_empty()
                && self.game_state.can_charge(self.bot_state.seat)
                && !self.game_state.done.charged(self.bot_state.seat)
            {
                Some(Action::Charge)
            } else {
                None
            }
        } else if self.game_state.phase.is_passing() {
            if !self.bot_state.pre_pass_hand.is_empty()
                && !self.game_state.done.sent_pass(self.bot_state.seat)
            {
                Some(Action::Pass)
            } else {
                None
            }
        } else if self.game_state.phase.is_playing() {
            if (self.bot_state.post_pass_hand - self.game_state.played).contains(Card::TwoClubs)
                || Some(self.bot_state.seat) == self.game_state.next_actor
            {
                Some(
                    if self.game_state.current_trick.is_empty()
                        && !self.game_state.claims.is_claiming(self.bot_state.seat)
                        && self.game_state.played.len() < 48
                        && must_claim(
                            self.bot_state.post_pass_hand - self.game_state.played,
                            self.game_state.played,
                        )
                    {
                        Action::Claim
                    } else {
                        Action::Play
                    },
                )
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
    Pass,
    Charge,
    Play,
    Claim,
}

fn must_claim(hand: Cards, played: Cards) -> bool {
    let remaining = Cards::ALL - hand - played;
    for suit in &[Cards::SPADES, Cards::HEARTS, Cards::DIAMONDS, Cards::CLUBS] {
        let hand = hand & *suit;
        let remaining = remaining & *suit;
        if !hand.is_empty() && !remaining.is_empty() && hand.min() < remaining.max() {
            return false;
        }
    }
    true
}
