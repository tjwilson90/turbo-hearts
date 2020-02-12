use crate::bot::random::Random;
use crate::cards::{Card, Cards};
use crate::error::CardsError;
use crate::game::{GameFeEvent, Games};
use crate::types::{GameId, Name};
use rand::Rng;
use std::sync::mpsc::TryRecvError;

mod random;

static NAMES: &[&str] = &include!("../names.json");

pub fn name() -> Name {
    let mut rng = rand::thread_rng();
    let initial = ('a' as u8 + rng.gen_range(0, 26)) as char;
    let mut name = initial.to_string();
    name.push_str(NAMES[rng.gen_range(0, NAMES.len())]);
    name.push_str(" (bot)");
    name
}

pub struct Bot {
    name: Name,
    algorithm: Box<dyn Algorithm + Send + Sync>,
}

impl Bot {
    pub fn new(name: Name, algorithm: &str) -> Self {
        let algorithm = match algorithm {
            "random" => Box::new(Random::new(name.clone())),
            _ => panic!("Unknown algorithm"),
        };
        Self { name, algorithm }
    }

    pub async fn run(mut self, games: Games, id: GameId) -> Result<(), CardsError> {
        let mut rx = games.subscribe(id, self.name.clone()).await?;
        loop {
            match rx.try_recv() {
                Ok(event) => self.algorithm.handle(event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return Ok(()),
            }
        }
        loop {
            match self.algorithm.reply() {
                Some(Action::Pass(cards)) => games.pass_cards(id, self.name.clone(), cards).await?,
                Some(Action::Charge(cards)) => {
                    games.charge_cards(id, self.name.clone(), cards).await?
                }
                Some(Action::Play(card)) => {
                    let complete = games.play_card(id, self.name.clone(), card).await?;
                    if complete {
                        return Ok(());
                    }
                }
                None => {}
            }
            match rx.recv().await {
                Some(event) => self.algorithm.handle(event),
                None => break,
            }
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Action {
    Pass(Cards),
    Charge(Cards),
    Play(Card),
}

trait Algorithm {
    fn handle(&mut self, event: GameFeEvent);
    fn reply(&self) -> Option<Action>;
}
