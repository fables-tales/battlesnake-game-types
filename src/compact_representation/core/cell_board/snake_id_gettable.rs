use itertools::Itertools;

use crate::{
    compact_representation::{core::dimensions::Dimensions, CellNum},
    types::{SnakeIDGettableGame, SnakeId},
};

use super::CellBoard;

impl<T: CellNum, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
    SnakeIDGettableGame for CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    type SnakeIDType = SnakeId;

    fn get_snake_ids(&self) -> Vec<Self::SnakeIDType> {
        // use the indices of the snakes with more than 0 health as the snake ids
        self.healths
            .iter()
            .enumerate()
            .filter(|(_, health)| **health > 0)
            .map(|(id, _)| SnakeId(id as u8))
            .collect_vec()
    }
}
