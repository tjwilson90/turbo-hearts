mod brute_force;
mod duck;
pub mod encoder;
mod gottatry;
mod hand_maker;
mod heuristic;
mod neural_network;
mod random;
mod simulate;
mod transposition_table;
mod void;

pub use brute_force::*;
pub use duck::*;
pub use gottatry::*;
pub use hand_maker::*;
pub use heuristic::*;
pub use neural_network::*;
pub use random::*;
pub use simulate::*;
pub use transposition_table::*;
use turbo_hearts_api::{BotState, Card, Cards, GameEvent, GameState};
pub use void::*;

pub enum Bot {
    Duck(DuckBot),
    GottaTry(GottaTryBot),
    Heuristic(HeuristicBot),
    NeuralNetwork(NeuralNetworkBot),
    Random(RandomBot),
    Simulate(SimulateBot),
}

pub trait Algorithm {
    fn pass(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards;
    fn charge(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards;
    fn play(&mut self, bot_state: &BotState, game_state: &GameState) -> Card;
    fn on_event(&mut self, bot_state: &BotState, game_state: &GameState, event: &GameEvent);
}

impl Algorithm for Bot {
    fn pass(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards {
        match self {
            Bot::Duck(bot) => bot.pass(bot_state, game_state),
            Bot::GottaTry(bot) => bot.pass(bot_state, game_state),
            Bot::Heuristic(bot) => bot.pass(bot_state, game_state),
            Bot::NeuralNetwork(bot) => bot.pass(bot_state, game_state),
            Bot::Random(bot) => bot.pass(bot_state, game_state),
            Bot::Simulate(bot) => bot.pass(bot_state, game_state),
        }
    }

    fn charge(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards {
        match self {
            Bot::Duck(bot) => bot.charge(bot_state, game_state),
            Bot::GottaTry(bot) => bot.charge(bot_state, game_state),
            Bot::Heuristic(bot) => bot.charge(bot_state, game_state),
            Bot::NeuralNetwork(bot) => bot.charge(bot_state, game_state),
            Bot::Random(bot) => bot.charge(bot_state, game_state),
            Bot::Simulate(bot) => bot.charge(bot_state, game_state),
        }
    }

    fn play(&mut self, bot_state: &BotState, game_state: &GameState) -> Card {
        match self {
            Bot::Duck(bot) => bot.play(bot_state, game_state),
            Bot::GottaTry(bot) => bot.play(bot_state, game_state),
            Bot::Heuristic(bot) => bot.play(bot_state, game_state),
            Bot::NeuralNetwork(bot) => bot.play(bot_state, game_state),
            Bot::Random(bot) => bot.play(bot_state, game_state),
            Bot::Simulate(bot) => bot.play(bot_state, game_state),
        }
    }

    fn on_event(&mut self, bot_state: &BotState, game_state: &GameState, event: &GameEvent) {
        match self {
            Bot::Duck(bot) => bot.on_event(bot_state, game_state, event),
            Bot::GottaTry(bot) => bot.on_event(bot_state, game_state, event),
            Bot::Heuristic(bot) => bot.on_event(bot_state, game_state, event),
            Bot::NeuralNetwork(bot) => bot.on_event(bot_state, game_state, event),
            Bot::Random(bot) => bot.on_event(bot_state, game_state, event),
            Bot::Simulate(bot) => bot.on_event(bot_state, game_state, event),
        }
    }
}
