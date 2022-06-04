use crate::{
    compact_representation::{core::dimensions::Dimensions, CellNum},
    types::{SnakeId, YouDeterminableGame},
};

use super::CellBoard;

impl<T: CellNum, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
    YouDeterminableGame for CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    fn is_you(&self, snake_id: &Self::SnakeIDType) -> bool {
        snake_id.0 == 0
    }

    fn you_id(&self) -> &Self::SnakeIDType {
        &SnakeId(0)
    }
}
