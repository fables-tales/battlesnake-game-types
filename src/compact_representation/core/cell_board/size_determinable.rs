use crate::{
    compact_representation::{core::dimensions::Dimensions, CellNum},
    types::SizeDeterminableGame,
};

use super::CellBoard;

impl<T: CellNum, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
    SizeDeterminableGame for CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    fn get_width(&self) -> u32 {
        self.get_actual_width() as u32
    }

    fn get_height(&self) -> u32 {
        self.get_actual_height() as u32
    }
}
