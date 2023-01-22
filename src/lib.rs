#![deny(
    warnings,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs
)]
//! Types for working with [battlesnake](https://docs.battlesnake.com/).
//! The goal is to provide simulation tooling and fast representations that
//! enable development of efficient minmax/MCTS.
//! you will likely be most interested in the CellBoard type which implements
//! all the traits necessary for minmax/MCTS and is much faster than using the
//! wire representation, in our benchmarks, we see that our compact representation
//! is on the order of about 33% faster for simulation than the wire representation.
//! ```plain
//!Gnuplot not found, using plotters backend
//! compact start of game   time:   [2.7520 us 2.7643 us 2.7774 us]
//!                         change: [-9.0752% -8.5713% -8.0468%] (p = 0.00 < 0.05)
//!                         Performance has improved.
//!
//! vec game start of game  time:   [4.1108 us 4.1303 us 4.1498 us]
//!                         change: [-12.869% -9.2803% -5.8488%] (p = 0.00 < 0.05)
//!                         Performance has improved.
//! Found 1 outliers among 100 measurements (1.00%)
//!   1 (1.00%) high mild
//!
//! compact late stage      time:   [14.098 us 14.152 us 14.209 us]
//!
//! vec late stage          time:   [21.124 us 21.337 us 21.592 us]
//! Found 14 outliers among 100 measurements (14.00%)
//! ```

use wire_representation::Game;

pub mod compact_representation;
pub mod hazard_algorithms;
pub mod types;
pub mod wire_representation;

/// Loads a fixture from a given string
pub fn game_fixture(game_fixture: &str) -> Game {
    let g: Result<Game, _> = serde_json::from_str(game_fixture);
    g.expect("the json literal is valid")
}
