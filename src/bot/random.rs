use crate::bot::{Action, Algorithm};
use crate::cards::{legal_plays, Card, Cards, Trick};
use crate::game::GameFeEvent;
use crate::types::{ChargingRules, Name, PassDirection, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

pub struct Random {
    name: Name,
    seat: Seat,
    rules: ChargingRules,
    pass_direction: PassDirection,
    pre_pass_hand: Cards,
    post_pass_hand: Cards,
    done_charging: bool,
    charged: Cards,
    led_suits: Cards,
    played: Cards,
    trick: Trick,
    next_action: Option<Action>,
}

enum State {}

impl Random {
    pub fn new(name: Name) -> Self {
        Self {
            name,
            seat: Seat::North,
            rules: ChargingRules::Classic,
            pass_direction: PassDirection::Left,
            pre_pass_hand: Cards::NONE,
            post_pass_hand: Cards::NONE,
            done_charging: false,
            charged: Cards::NONE,
            led_suits: Cards::NONE,
            played: Cards::NONE,
            trick: Trick::new(Seat::North),
            next_action: None,
        }
    }

    fn pass(&self) -> Cards {
        let mut hand = self.pre_pass_hand.into_iter().collect::<Vec<_>>();
        hand.shuffle(&mut rand::thread_rng());
        hand.into_iter().take(3).collect()
    }

    fn charge(&self) -> Cards {
        let cards = self.post_pass_hand & Cards::CHARGEABLE;
        cards
            .into_iter()
            .filter(|_| rand::thread_rng().gen())
            .collect()
    }

    fn play(&self) -> Card {
        random(legal_plays(
            self.post_pass_hand,
            &self.trick,
            self.led_suits,
            self.charged,
            self.played,
        ))
    }
}

fn random(cards: Cards) -> Card {
    let index = rand::thread_rng().gen_range(0, cards.len());
    cards.into_iter().nth(index).unwrap()
}

impl Algorithm for Random {
    fn handle(&mut self, event: GameFeEvent) {
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
            } => {
                self.pre_pass_hand = north | east | south | west;
                self.post_pass_hand = self.pre_pass_hand;
                self.done_charging = false;
                self.charged = Cards::NONE;
                self.led_suits = Cards::NONE;
                self.played = Cards::NONE;
                if self.rules.free() {
                    self.next_action = Some(Action::Pass(self.pass()))
                }
            }
            GameFeEvent::SendPass { cards, .. } => self.post_pass_hand -= cards,
            GameFeEvent::RecvPass { cards, .. } => {
                self.post_pass_hand |= cards;
                self.pass_direction = self.pass_direction.next().unwrap_or(PassDirection::Keeper);
            }
            GameFeEvent::BlindCharge { seat, .. } => {
                if seat == self.seat {
                    self.done_charging = true;
                }
            }
            GameFeEvent::Charge { seat, cards } => {
                if seat == self.seat {
                    self.done_charging = true;
                }
                self.charged |= cards;
            }
            GameFeEvent::Play { .. } => {}
        }
    }

    fn reply(&self) -> Option<Action> {
        self.next_action
    }
}
