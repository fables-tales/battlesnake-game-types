use crate::{compact_representation::CellNum, types::SizeDeterminableGame};

use super::CellBoard;

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> SizeDeterminableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn get_width(&self) -> u32 {
        self.actual_width as u32
    }

    fn get_height(&self) -> u32 {
        self.actual_height() as u32
    }
}