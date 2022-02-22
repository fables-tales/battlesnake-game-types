use crate::{compact_representation::CellNum, types::HeadGettableGame};

use super::CellBoard;

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> HeadGettableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn get_head_as_position(
        &self,
        snake_id: &Self::SnakeIDType,
    ) -> crate::wire_representation::Position {
        let idx = self.heads[snake_id.0.as_usize()];
        let width = self.actual_width;
        idx.into_position(width)
    }

    fn get_head_as_native_position(
        &self,
        snake_id: &Self::SnakeIDType,
    ) -> Self::NativePositionType {
        self.heads[snake_id.0.as_usize()]
    }
}
