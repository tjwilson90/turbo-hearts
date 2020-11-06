mod brute_force;
mod duck;
mod gottatry;
mod heuristic;
mod random;
mod simulate;
mod transposition_table;
mod void;

pub use brute_force::*;
pub use duck::*;
pub use gottatry::*;
pub use heuristic::*;
pub use random::*;
pub use simulate::*;
pub use transposition_table::*;
pub use void::*;

pub enum Bot {
    Duck(DuckBot),
    GottaTry(GottaTryBot),
    Heuristic(HeuristicBot),
    Random(RandomBot),
    Simulate(SimulateBot),
}
