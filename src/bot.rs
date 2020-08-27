use crate::{
    bot::{
        duck::Duck, gottatry::GottaTry, heuristic::Heuristic, random::Random, simulate::Simulate,
    },
    card::Card,
    cards::Cards,
    error::CardsError,
    game::{event::GameEvent, id::GameId, state::GameState, Games},
    rank::Rank,
    seat::Seat,
    suit::Suit,
    user::UserId,
};
use log::debug;
use rand::distributions::Distribution;
use rand_distr::Gamma;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tokio::{
    sync::mpsc::{error::TryRecvError, UnboundedReceiver},
    task, time,
    time::Duration,
};

mod duck;
mod gottatry;
mod heuristic;
mod random;
mod simulate;
mod void;

pub struct Bot {
    game_id: GameId,
    user_id: UserId,
    bot_state: BotState,
    game_state: GameState,
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
    Simulate,
}

pub struct BotState {
    seat: Seat,
    pre_pass_hand: Cards,
    post_pass_hand: Cards,
}

impl Bot {
    pub fn new(game_id: GameId, user_id: UserId, strategy: Strategy) -> Self {
        let algorithm: Box<dyn Algorithm + Send + Sync> = match strategy {
            Strategy::Duck => Box::new(Duck::new()),
            Strategy::GottaTry => Box::new(GottaTry::new()),
            Strategy::Heuristic => Box::new(Heuristic::new()),
            Strategy::Random => Box::new(Random::new()),
            Strategy::Simulate => Box::new(Simulate::new()),
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
            match action {
                Some(Action::Pass) => {
                    let cards = self.algorithm.pass(&self.bot_state, &self.game_state);
                    Bot::delay(delay, now).await;
                    let _ = games.pass_cards(self.game_id, self.user_id, cards).await;
                }
                Some(Action::Charge) => {
                    let cards = self.algorithm.charge(&self.bot_state, &self.game_state);
                    Bot::delay(delay, now).await;
                    let _ = games.charge_cards(self.game_id, self.user_id, cards).await;
                }
                Some(Action::Play) => {
                    let card = task::block_in_place(|| {
                        self.algorithm.play(&self.bot_state, &self.game_state)
                    });
                    if card != Card::TwoClubs {
                        Bot::delay(delay, now).await;
                    }
                    match games.play_card(self.game_id, self.user_id, card).await {
                        Ok(true) => return Ok(()),
                        _ => {}
                    }
                }
                Some(Action::Claim) => {
                    Bot::delay(delay, now).await;
                    let _ = games.claim(self.game_id, self.user_id).await;
                }
                Some(Action::AcceptClaim(seat)) => {
                    Bot::delay(delay, now).await;
                    match games.accept_claim(self.game_id, self.user_id, seat).await {
                        Ok(true) => return Ok(()),
                        _ => {}
                    }
                }
                Some(Action::RejectClaim(seat)) => {
                    Bot::delay(delay, now).await;
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
                return if *seat == self.bot_state.seat {
                    None
                } else if can_claim(*hand, &self.game_state) {
                    Some(Action::AcceptClaim(*seat))
                } else {
                    Some(Action::RejectClaim(*seat))
                };
            }
            GameEvent::AcceptClaim { .. } => {
                return None;
            }
            _ => {}
        }

        self.algorithm
            .on_event(&self.bot_state, &self.game_state, &event);

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
    AcceptClaim(Seat),
    RejectClaim(Seat),
}

#[allow(unused_variables)]
trait Algorithm {
    fn pass(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards;
    fn charge(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards;
    fn play(&mut self, bot_state: &BotState, game_state: &GameState) -> Card;
    fn on_event(&mut self, bot_state: &BotState, game_state: &GameState, event: &GameEvent);
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

fn can_claim(hand: Cards, state: &GameState) -> bool {
    if !state.current_trick.is_empty() {
        return false;
    }
    let heart_losers = losers(Suit::Hearts, hand, &state);
    let other_losers = losers(Suit::Spades, hand, &state)
        + losers(Suit::Diamonds, hand, &state)
        + losers(Suit::Clubs, hand, &state);
    return if state.played.contains_any(Cards::HEARTS) {
        heart_losers + other_losers <= 0
    } else {
        other_losers <= 0 && heart_losers + other_losers <= 0
    };
}

fn losers(suit: Suit, hand: Cards, state: &GameState) -> i8 {
    let mut hand = hand & suit.cards();
    let mut remaining = suit.cards() - hand - state.played;
    let nine = suit.with_rank(Rank::Nine);
    let mut legal_plays = if hand.len() == 1 || state.led_suits.contains(suit) {
        hand
    } else {
        hand - state.charges.all_charges()
    };
    let mut had_winner = false;

    loop {
        if hand.is_empty() {
            return 0;
        }
        if remaining.is_empty() {
            return if hand.contains(nine) { -1 } else { 0 };
        }
        let top = if remaining == nine.into() {
            nine
        } else {
            (remaining - nine).max()
        };
        if top > legal_plays.max() {
            if had_winner && hand.contains(nine) {
                hand -= nine;
                remaining -= remaining.min();
                continue;
            }
            if top == nine && hand.len() == 2 && hand.max() > top {
                return 0;
            }
            return if top > hand.max() {
                hand.len()
            } else {
                legal_plays.len()
            } as i8;
        }
        let winners: Cards = legal_plays.into_iter().filter(|c| *c > top).collect();
        if winners == nine.into() {
            hand -= nine;
            remaining -= remaining.min();
            if hand.is_empty() {
                return -1;
            }
            hand -= hand.min();
            if !remaining.is_empty() {
                remaining -= remaining.min();
            }
        } else {
            hand -= (winners - nine).min();
            remaining -= remaining.min();
        }
        legal_plays = hand;
        had_winner = true;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_losers() {
        fn case(hand: &str, remaining: &str, can_lead_charged: bool) -> i8 {
            let hand = format!("{}S", hand).parse().unwrap();
            let remaining: Cards = format!("{}S", remaining).parse().unwrap();
            let mut state = GameState::new();
            state.played = Cards::SPADES - hand - remaining;
            if !can_lead_charged {
                state.charges.charge(Seat::North, Cards::QUEEN_SPADES);
            }
            losers(Suit::Spades, hand, &state)
        }
        assert_eq!(0, case("AKQ", "23", true));
        assert_eq!(0, case("AK", "234", true));
        assert_eq!(-1, case("AK9", "23", true));
        assert_eq!(-1, case("A9", "23", true));
        assert_eq!(-1, case("A9", "234", true));
        assert_eq!(0, case("A92", "34", true));
        assert_eq!(-1, case("A92", "3", true));
        assert_eq!(0, case("A", "9", true));
        assert_eq!(-1, case("A9", "J", true));
        assert_eq!(0, case("A9", "QJ", true));
        assert_eq!(0, case("A9", "KQJ", true));
        assert_eq!(1, case("A93", "KQJ", true));
        assert_eq!(2, case("A83", "KQJ", true));
        assert_eq!(0, case("A93", "QJ", true));
        assert_eq!(0, case("J8", "954", true));
        assert_eq!(1, case("J8", "T954", true));
        assert_eq!(3, case("K93", "AQJ", true));
        assert_eq!(2, case("K3", "AQJ", true));
        assert_eq!(1, case("8", "9", true));

        assert_eq!(1, case("Q8", "T", false));
        assert_eq!(0, case("Q8", "9", false));
        assert_eq!(0, case("Q8", "4", false));
        assert_eq!(0, case("Q9", "4", false));
        assert_eq!(0, case("Q93", "4", false));
        assert_eq!(-1, case("Q95", "4", false));
        assert_eq!(0, case("Q", "T", false));
        assert_eq!(1, case("Q", "K", false));
    }
}
