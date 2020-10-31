mod duck;
mod gottatry;
mod heuristic;
mod random;
mod simulate;
mod void;

pub use duck::*;
pub use gottatry::*;
pub use heuristic::*;
pub use random::*;
pub use simulate::*;
pub use void::*;

pub enum Bot {
    Duck(DuckBot),
    GottaTry(GottaTryBot),
    Heuristic(HeuristicBot),
    Random(RandomBot),
    Simulate(SimulateBot),
}
