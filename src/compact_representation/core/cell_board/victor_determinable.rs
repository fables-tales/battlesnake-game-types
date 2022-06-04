use itertools::Itertools;

use crate::{
    compact_representation::{core::dimensions::Dimensions, CellNum},
    types::{SnakeId, VictorDeterminableGame},
};

use super::CellBoard;

impl<T: CellNum, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
    VictorDeterminableGame for CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    fn is_over(&self) -> bool {
        self.healths[0] == 0 || self.healths.iter().filter(|h| **h != 0).count() <= 1
    }

    fn get_winner(&self) -> Option<Self::SnakeIDType> {
        if self.is_over() {
            let winning_ids = self
                .healths
                .iter()
                .enumerate()
                .filter_map(|(id, health)| {
                    if *health != 0 {
                        Some(SnakeId(id as u8))
                    } else {
                        None
                    }
                })
                .collect_vec();
            if winning_ids.is_empty() {
                return None;
            } else {
                return Some(winning_ids[0]);
            }
        }
        None
    }

    fn alive_snake_count(&self) -> usize {
        self.healths.iter().filter(|h| **h != 0).count()
    }
}
