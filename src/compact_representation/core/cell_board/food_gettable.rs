use crate::{
    compact_representation::{
        core::{dimensions::Dimensions, CellIndex},
        CellNum,
    },
    types::FoodGettableGame,
};

use super::CellBoard;

impl<T: CellNum, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize> FoodGettableGame
    for CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    fn get_all_food_as_positions(&self) -> Vec<crate::wire_representation::Position> {
        self.cells
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_food())
            .map(|(i, _)| CellIndex(T::from_usize(i)).into_position(self.get_actual_width()))
            .collect()
    }

    fn get_all_food_as_native_positions(&self) -> Vec<Self::NativePositionType> {
        self.cells
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_food())
            .map(|(i, _)| CellIndex(T::from_usize(i)))
            .collect()
    }
}
