use std::fmt::Display;

/// Wrapper type for numbers to allow for shrinking board sizes
pub trait CellNum:
    std::fmt::Debug + Copy + Clone + PartialEq + Eq + std::hash::Hash + Ord + Display + 'static
{
    /// converts this cellnum to a usize
    fn as_usize(&self) -> usize;
    /// makes a cellnum from an i32
    fn from_i32(i: i32) -> Self;
    /// makes a cellnum from an usize
    fn from_usize(i: usize) -> Self;
}

impl CellNum for u8 {
    fn as_usize(&self) -> usize {
        *self as usize
    }

    fn from_i32(i: i32) -> Self {
        i as u8
    }

    fn from_usize(i: usize) -> Self {
        i as u8
    }
}
impl CellNum for u16 {
    fn as_usize(&self) -> usize {
        *self as usize
    }

    fn from_i32(i: i32) -> Self {
        i as u16
    }

    fn from_usize(i: usize) -> Self {
        i as u16
    }
}
