use crate::{
    compact_representation::{
        core::{dimensions::Dimensions, CellIndex},
        CellNum,
    },
    types::PositionGettableGame,
    wire_representation::Position,
};

use super::CellBoard;

impl<T: CellNum, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
    PositionGettableGame for CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    type NativePositionType = CellIndex<T>;

    fn position_is_snake_body(&self, pos: Self::NativePositionType) -> bool {
        let cell = self.get_cell(pos);

        cell.is_body_segment()
    }

    fn position_from_native(&self, pos: Self::NativePositionType) -> Position {
        let width = self.get_actual_width();

        pos.into_position(width)
    }

    fn native_from_position(&self, pos: Position) -> Self::NativePositionType {
        Self::NativePositionType::new(pos, self.get_actual_width())
    }

    fn off_board(&self, pos: Position) -> bool {
        pos.x < 0
            || pos.x >= self.get_actual_width() as i32
            || pos.y < 0
            || pos.y >= self.get_actual_height() as i32
    }
}
