use crate::{
    compact_representation::{core::dimensions::Dimensions, CellNum},
    types::HealthGettableGame,
};

use super::CellBoard;

impl<T: CellNum, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize> HealthGettableGame
    for CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    type HealthType = u8;
    const ZERO: Self::HealthType = 0;

    fn get_health(&self, snake_id: &Self::SnakeIDType) -> Self::HealthType {
        self.healths[snake_id.0.as_usize()]
    }

    fn get_health_i64(&self, snake_id: &Self::SnakeIDType) -> i64 {
        self.get_health(snake_id) as i64
    }
}
