use crate::{compact_representation::CellNum, types::HazardQueryableGame};

use super::CellBoard;

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> HazardQueryableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn is_hazard(&self, pos: &Self::NativePositionType) -> bool {
        self.cell_is_hazard(*pos)
    }

    fn get_hazard_damage(&self) -> u8 {
        self.hazard_damage
    }
}
