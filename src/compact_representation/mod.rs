//! A compact board representation that is efficient for simulation
mod core;
mod standard;
pub mod wrapped;

pub use self::core::CellNum;

/// A cell board for a standard game (e.g. not wrapped or constrictor)
pub type StandardCellBoard<T, const BOARD_SIZE: usize, const MAX_SNAKES: usize> =
    standard::CellBoard<T, BOARD_SIZE, MAX_SNAKES>;

/// A standard mode board, 11x11 with 4 snakes
pub type StandardCellBoard4Snakes11x11 = StandardCellBoard<u8, { 11 * 11 }, 4>;

/// A cell board for a wrapped game
pub type WrappedCellBoard<T, const BOARD_SIZE: usize, const MAX_SNAKES: usize> =
    wrapped::CellBoard<T, BOARD_SIZE, MAX_SNAKES>;

/// A wrapped mode board, 11x11 with 4 snakes
pub type WrappedCellBoard4Snakes11x11 = WrappedCellBoard<u8, { 11 * 11 }, 4>;
