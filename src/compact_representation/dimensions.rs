//! We can represent several different configuration of boards
//!
//! Some of these are more compact than others, but they are also more specialized.
//! For example: We have a [Square] struct which only holds onto the width and assumes the height
//! matches. We also have [Fixed] which is a fixed sized board at compile time.

use core::fmt::Debug;
use std::hash::Hash;

/// Trait that all different Dimensions must implement
pub trait Dimensions: Debug + Copy + Hash {
    /// Convert from a width and a height to this dimension
    fn from_dimensions(width: u8, height: u8) -> Self;

    /// Get the width of this dimension
    fn width(&self) -> u8;

    /// Get the height of this dimension
    fn height(&self) -> u8;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// A square board
///
/// We only store the width of the game and assume the height matches
pub struct Square {
    width: u8,
}

impl Dimensions for Square {
    fn width(&self) -> u8 {
        self.width
    }

    fn height(&self) -> u8 {
        self.width
    }

    fn from_dimensions(width: u8, height: u8) -> Self {
        debug_assert!(width == height);

        Self { width }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// A fixed size board
pub struct Fixed<const W: u8, const H: u8>;

impl<const W: u8, const H: u8> Dimensions for Fixed<W, H> {
    fn width(&self) -> u8 {
        W
    }

    fn height(&self) -> u8 {
        H
    }

    fn from_dimensions(width: u8, height: u8) -> Self {
        debug_assert_eq!(width, W);
        debug_assert_eq!(height, H);

        Self
    }
}

/// Alias for a [Fixed] board at the height and width for the ArcadeMaze map
pub type ArcadeMaze = Fixed<19, 21>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// A fully custom dimension
///
/// Stores the height and width seperately
pub struct Custom {
    width: u8,
    height: u8,
}

impl Dimensions for Custom {
    fn width(&self) -> u8 {
        self.width
    }

    fn height(&self) -> u8 {
        self.height
    }

    fn from_dimensions(width: u8, height: u8) -> Self {
        Self { width, height }
    }
}
