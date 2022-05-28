use crate::{
    compact_representation::{core::dimensions::Dimensions, CellNum},
    types::HeadGettableGame,
};

use super::CellBoard;

impl<T: CellNum, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize> HeadGettableGame
    for CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    fn get_head_as_position(
        &self,
        snake_id: &Self::SnakeIDType,
    ) -> crate::wire_representation::Position {
        let idx = self.heads[snake_id.0.as_usize()];
        let width = self.get_actual_width();
        idx.into_position(width)
    }

    fn get_head_as_native_position(
        &self,
        snake_id: &Self::SnakeIDType,
    ) -> Self::NativePositionType {
        self.heads[snake_id.0.as_usize()]
    }
}
