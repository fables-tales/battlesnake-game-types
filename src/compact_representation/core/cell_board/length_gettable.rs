use std::fmt::Display;

use crate::{
    compact_representation::{
        core::{dimensions::Dimensions, CellIndex},
        CellNum,
    },
    types::LengthGettableGame,
    wire_representation::Position,
};

use super::CellBoard;

impl<T: CellNum, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize> LengthGettableGame
    for CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    type LengthType = u16;

    fn get_length(&self, snake_id: &Self::SnakeIDType) -> Self::LengthType {
        self.lengths[snake_id.0.as_usize()]
    }

    fn get_length_i64(&self, snake_id: &Self::SnakeIDType) -> i64 {
        self.get_length(*snake_id) as i64
    }
}
impl<T: CellNum, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize> Display
    for CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = self.get_actual_width();
        let height = self.get_actual_height();
        writeln!(f)?;
        for y in 0..height {
            for x in 0..width {
                let y = height - y - 1;
                let position = Position {
                    x: x as i32,
                    y: y as i32,
                };
                let cell_idx = CellIndex::new(position, width);
                if self.cell_is_snake_head(cell_idx) {
                    let id = self.get_snake_id_at(cell_idx);
                    write!(f, "{}", id.unwrap().as_usize())?;
                } else if self.cell_is_food(cell_idx) {
                    write!(f, "f")?
                } else if self.cell_is_body(cell_idx) {
                    write!(f, "s")?
                } else if self.cell_is_hazard(cell_idx) {
                    write!(f, "x")?
                } else {
                    debug_assert!(self.cells[cell_idx.0.as_usize()].is_empty());
                    write!(f, ".")?
                }
                write!(f, " ")?;
            }
            writeln!(f)?;
        }
        let hash_repr = self.pack_as_hash();
        writeln!(f, "{}", serde_json::to_string(&hash_repr).unwrap())?;
        Ok(())
    }
}
