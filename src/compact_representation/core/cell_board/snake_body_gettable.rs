use crate::{compact_representation::CellNum, types::{SnakeBodyGettableGame, HeadGettableGame}};

use super::CellBoard;

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> SnakeBodyGettableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn get_snake_body_vec(&self, snake_id: &Self::SnakeIDType) -> Vec<Self::NativePositionType> {
        let mut body = vec![];
        body.reserve(self.get_length(*snake_id).into());
        let head = self.get_head_as_native_position(snake_id);

        let mut cur = Some(self.get_cell(head).get_tail_position(head).unwrap());

        while let Some(c) = cur {
            body.push(c);
            if self.get_cell(c).is_double_stacked_piece() {
                body.push(c);
            }
            if self.get_cell(c).is_triple_stacked_piece() {
                body.push(c);
                body.push(c);
            }
            cur = self.get_cell(c).get_next_index();
        }

        body.reverse();

        body
    }
}
