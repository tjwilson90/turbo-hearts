use crate::bot::{Action, Algorithm};
use crate::cards::{legal_plays, Card, Cards, ChargeState, HandState};
use crate::game::GameFeEvent;
use crate::types::{ChargingRules, Name, PassDirection, Seat};
use log::info;
use rand::seq::SliceRandom;
use rand::Rng;

pub struct Random {
    name: Name,
    seat: Seat,
    rules: ChargingRules,
    pre_pass_hand: Cards,
    post_pass_hand: Cards,
    charged: bool,
    charges: ChargeState,
    hand: HandState,
    next_action: Option<Action>,
}

impl Random {
    pub fn new(name: Name) -> Self {
        Self {
            name,
            seat: Seat::North,
            rules: ChargingRules::Classic,
            pre_pass_hand: Cards::NONE,
            post_pass_hand: Cards::NONE,
            charged: false,
            charges: ChargeState::new(ChargingRules::Classic, PassDirection::Left),
            hand: HandState::new(Seat::North),
            next_action: None,
        }
    }

    fn pass(&self) -> Cards {
        let mut hand = self.pre_pass_hand.into_iter().collect::<Vec<_>>();
        hand.shuffle(&mut rand::thread_rng());
        hand.into_iter().take(3).collect()
    }

    fn charge(&self) -> Cards {
        let cards = (self.post_pass_hand & Cards::CHARGEABLE) - self.charges.charged;
        cards
            .into_iter()
            .filter(|_| rand::thread_rng().gen())
            .collect()
    }

    fn play(&self) -> Card {
        random(legal_plays(
            self.post_pass_hand,
            &self.hand,
            self.charges.charged,
        ))
    }
}

fn random(cards: Cards) -> Card {
    let index = rand::thread_rng().gen_range(0, cards.len());
    cards.into_iter().nth(index).unwrap()
}

impl Algorithm for Random {
    fn handle(&mut self, event: GameFeEvent) {
        info!("{} handling event {:?}", self.name, event);
        match event {
            GameFeEvent::Ping => {}
            GameFeEvent::Sit {
                north,
                east,
                south,
                west,
                rules,
            } => {
                self.seat = if &self.name == north.name() {
                    Seat::North
                } else if &self.name == east.name() {
                    Seat::East
                } else if &self.name == south.name() {
                    Seat::South
                } else if &self.name == west.name() {
                    Seat::West
                } else {
                    panic!("{} is not a player in the game", self.name);
                };
                self.rules = rules;
            }
            GameFeEvent::Deal {
                north,
                east,
                south,
                west,
                pass,
            } => {
                self.pre_pass_hand = north | east | south | west;
                self.post_pass_hand = self.pre_pass_hand;
                self.charges = ChargeState::new(self.rules, pass);
            }
            GameFeEvent::SendPass { cards, .. } => self.post_pass_hand -= cards,
            GameFeEvent::RecvPass { cards, .. } => {
                self.post_pass_hand |= cards;
            }
            GameFeEvent::BlindCharge { seat, count } => {
                self.charges.blind_charge(seat, count);
                self.charged |= seat == self.seat;

                if !self.charges.done_charging(self.seat) && self.charges.can_charge(self.seat) {
                    self.next_action = Some(Action::Charge(if self.charged {
                        Cards::NONE
                    } else {
                        self.charge()
                    }));
                }
            }
            GameFeEvent::Charge { seat, cards } => {
                self.charges.charge(seat, cards);
                self.charged |= seat == self.seat;

                if !self.charges.done_charging(self.seat) && self.charges.can_charge(self.seat) {
                    self.next_action = Some(Action::Charge(if self.charged {
                        Cards::NONE
                    } else {
                        self.charge()
                    }));
                }
            }
            GameFeEvent::RevealCharge { cards, .. } => {
                self.charges.charged |= cards;
            }
            GameFeEvent::Play { card, .. } => {
                self.hand.play(card);
                if self.hand.next_player == self.seat && self.hand.played != Cards::ALL {
                    self.next_action = Some(Action::Play(self.play()));
                }
            }
            GameFeEvent::StartPassing => self.next_action = Some(Action::Pass(self.pass())),
            GameFeEvent::StartCharging { seat } => {
                self.charged = false;
                match seat {
                    Some(seat) if seat != self.seat => {}
                    _ => self.next_action = Some(Action::Charge(self.charge())),
                }
            }
            GameFeEvent::StartTrick {
                leader,
                trick_number,
            } => {
                if trick_number == 0 {
                    self.hand.reset(leader);
                }
                if leader == self.seat {
                    self.next_action = Some(Action::Play(self.play()));
                }
            }
        }
    }

    fn reply(&mut self) -> Option<Action> {
        info!("{} reply with action {:?}", self.name, self.next_action);
        self.next_action.take()
    }
}
