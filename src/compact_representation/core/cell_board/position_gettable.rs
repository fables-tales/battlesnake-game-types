use crate::{types::PositionGettableGame, compact_representation::{core::CellIndex, CellNum}, wire_representation::Position};

use super::CellBoard;

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> PositionGettableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    type NativePositionType = CellIndex<T>;

    fn position_is_snake_body(&self, pos: Self::NativePositionType) -> bool {
        let cell = self.get_cell(pos);

        cell.is_body_segment()
    }

    fn position_from_native(&self, pos: Self::NativePositionType) -> Position {
        let width = self.actual_width;

        pos.into_position(width)
    }

    fn native_from_position(&self, pos: Position) -> Self::NativePositionType {
        Self::NativePositionType::new(pos, self.actual_width)
    }

    fn off_board(&self, pos: Position) -> bool {
        pos.x < 0 || pos.x >= self.get_actual_width() as i32 || pos.y < 0 || pos.y >= self.get_actual_height() as i32
    }
}
