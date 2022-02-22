use crate::{compact_representation::CellNum, types::HazardSettableGame};

use super::CellBoard;

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> HazardSettableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn set_hazard(&mut self, pos: Self::NativePositionType) {
        self.cells[pos.0.as_usize()].set_hazard();
    }

    fn clear_hazard(&mut self, pos: Self::NativePositionType) {
        self.cells[pos.0.as_usize()].clear_hazard();
    }
}