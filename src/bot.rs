use crate::{
    bot::{duck::Duck, gottatry::GottaTry, heuristic::Heuristic, random::Random},
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
                    let _ = games.pass_cards(self.game_id, self.user_id, cards).await;
                }
                Some(Action::Charge(cards)) => {
                    let _ = games.charge_cards(self.game_id, self.user_id, cards).await;
                }
                Some(Action::Play(card)) => {
                    match games.play_card(self.game_id, self.user_id, card).await {
                        Ok(true) => return Ok(()),
                        _ => {}
                    }
                }
                Some(Action::Claim) => {
                    let _ = games.claim(self.game_id, self.user_id).await;
                }
                Some(Action::AcceptClaim(seat)) => {
                    match games.accept_claim(self.game_id, self.user_id, seat).await {
                        Ok(true) => return Ok(()),
                        _ => {}
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
        let phase = self.state.game.phase;
        self.state.game.apply(&event);
        if phase.is_playing() && !self.state.game.phase.is_playing() {
            self.state.pre_pass_hand = Cards::NONE;
            self.state.post_pass_hand = Cards::NONE;
        }
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
            GameEvent::Claim { seat, hand } => {
                return if *seat == self.state.seat {
                    None
                } else if can_claim(*hand, &self.state.game) {
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

        self.algorithm.on_event(&self.state, &event);

        if self.state.game.phase.is_charging() {
            if !self.state.pre_pass_hand.is_empty()
                && self.state.game.can_charge(self.state.seat)
                && !self.state.game.done.charged(self.state.seat)
            {
                Some(Action::Charge(self.algorithm.charge(&self.state)))
            } else {
                None
            }
        } else if self.state.game.phase.is_passing() {
            if !self.state.pre_pass_hand.is_empty()
                && !self.state.game.done.sent_pass(self.state.seat)
            {
                Some(Action::Pass(self.algorithm.pass(&self.state)))
            } else {
                None
            }
        } else if self.state.game.phase.is_playing() {
            if (self.state.post_pass_hand - self.state.game.played).contains(Card::TwoClubs)
                || Some(self.state.seat) == self.state.game.next_actor
            {
                Some(
                    if self.state.game.current_trick.is_empty()
                        && !self.state.game.claims.is_claiming(self.state.seat)
                        && self.state.game.played.len() < 48
                        && must_claim(
                            self.state.post_pass_hand - self.state.game.played,
                            self.state.game.played,
                        )
                    {
                        Action::Claim
                    } else {
                        Action::Play(self.algorithm.play(&self.state))
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
    Pass(Cards),
    Charge(Cards),
    Play(Card),
    Claim,
    AcceptClaim(Seat),
    RejectClaim(Seat),
}

#[allow(unused_variables)]
trait Algorithm {
    fn pass(&mut self, state: &BotState) -> Cards;
    fn charge(&mut self, state: &BotState) -> Cards;
    fn play(&mut self, state: &BotState) -> Card;

    fn on_event(&mut self, state: &BotState, event: &GameEvent);
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
