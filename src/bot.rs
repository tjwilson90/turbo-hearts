use crate::cards::{Card, Cards};
use crate::error::CardsError;
use crate::game::{GameFeEvent, Games};
use crate::types::{GameId, Name};
use rand::Rng;

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
    algorithm: Box<dyn Algorithm + Send>,
}

impl Bot {
    pub fn new(name: Name, algorithm: &str) -> Self {
        Self {
            name,
            algorithm: match algorithm {
                "random" => Box::new(Random),
                _ => panic!("Unknown algorithm"),
            },
        }
    }

    pub async fn run(mut self, games: Games, id: GameId) -> Result<(), CardsError> {
        let mut rx = games.subscribe(id, self.name.clone()).await?;
        while let Some(event) = rx.recv().await {
            for action in self.algorithm.handle(event) {
                match action {
                    Action::Pass(cards) => games.pass_cards(id, self.name.clone(), cards).await?,
                    Action::Charge(cards) => {
                        games.charge_cards(id, self.name.clone(), cards).await?
                    }
                    Action::Play(card) => {
                        let complete = games.play_card(id, self.name.clone(), card).await?;
                        if complete {
                            return Ok(());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

enum Action {
    Pass(Cards),
    Charge(Cards),
    Play(Card),
}

trait Algorithm {
    fn handle(&mut self, event: GameFeEvent) -> Vec<Action>;
}

struct Random;

impl Algorithm for Random {
    fn handle(&mut self, event: GameFeEvent) -> Vec<Action> {
        unimplemented!()
    }
}
