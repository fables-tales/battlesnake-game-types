use crate::{
    compact_representation::{core::dimensions::Dimensions, CellNum},
    types::HazardQueryableGame,
};

use super::CellBoard;

impl<T: CellNum, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
    HazardQueryableGame for CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    fn is_hazard(&self, pos: &Self::NativePositionType) -> bool {
        self.cell_is_hazard(*pos)
    }

    fn get_hazard_damage(&self) -> u8 {
        self.hazard_damage
    }
}
