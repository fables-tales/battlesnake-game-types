use itertools::Itertools;

use crate::{types::{SnakeIDGettableGame, SnakeId}, compact_representation::CellNum};

use super::CellBoard;


impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> SnakeIDGettableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
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
