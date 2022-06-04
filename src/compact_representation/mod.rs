//! A compact board representation that is efficient for simulation
mod core;
pub mod standard;
pub mod wrapped;

pub use self::core::CellIndex;
pub use self::core::CellNum;

use self::dimensions::Square;

pub mod dimensions;

/// A cell board for a standard game (e.g. not wrapped or constrictor)
pub type StandardCellBoard<T, D, const BOARD_SIZE: usize, const MAX_SNAKES: usize> =
    standard::CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>;

/// A standard mode board, 11x11 with 4 snakes
pub type StandardCellBoard4Snakes11x11 = StandardCellBoard<u8, Square, { 11 * 11 }, 4>;

/// A cell board for a wrapped game
pub type WrappedCellBoard<T, D, const BOARD_SIZE: usize, const MAX_SNAKES: usize> =
    wrapped::CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>;

/// A wrapped mode board, 11x11 with 4 snakes
pub type WrappedCellBoard4Snakes11x11 = WrappedCellBoard<u8, Square, { 11 * 11 }, 4>;
