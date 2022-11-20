use std::collections::HashMap;
use std::error::Error;
use std::slice::Iter;

use itertools::Itertools;
use rand::seq::IteratorRandom;

use crate::types::EmptyCellGettableGame;
use crate::types::SnakeIDMap;
use crate::types::SnakeId;
use crate::types::StandardFoodPlaceableGame;
use crate::wire_representation::Game;
use crate::wire_representation::Position;

use super::dimensions::Dimensions;
use super::Cell;
use super::CellIndex;
use super::CellNum as CN;
use super::{DOUBLE_STACK, TRIPLE_STACK};

mod eval;
mod food_gettable;
mod hazard_queryable;
mod hazard_settable;
mod head_gettable;
mod health_gettable;
mod length_gettable;
mod neck_queryable;
mod position_gettable;
mod size_determinable;
mod snake_body_gettable;
mod snake_id_gettable;
mod victor_determinable;
mod you_determinable;

pub use eval::EvaluateMode;

/// A compact board representation that is significantly faster for simulation than
/// `battlesnake_game_types::wire_representation::Game`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CellBoard<
    T: CN,
    DimensionsType: Dimensions,
    const BOARD_SIZE: usize,
    const MAX_SNAKES: usize,
> {
    hazard_damage: u8,
    cells: [Cell<T>; BOARD_SIZE],
    healths: [u8; MAX_SNAKES],
    heads: [CellIndex<T>; MAX_SNAKES],
    lengths: [u16; MAX_SNAKES],
    dimensions: DimensionsType,
}

#[allow(dead_code)]
fn get_snake_id(
    snake: &crate::wire_representation::BattleSnake,
    snake_ids: &SnakeIDMap,
) -> Option<SnakeId> {
    if snake.health == 0 {
        None
    } else {
        Some(*snake_ids.get(&snake.id).unwrap())
    }
}

impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
    CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    pub fn iter_healths(&self) -> Iter<'_, u8> {
        self.healths.iter()
    }

    /// Asserts that all tails eventually loop back to a head and panics if the board is inconsistent
    pub fn assert_consistency(&self) -> bool {
        for i in 0..MAX_SNAKES {
            let snake_id = SnakeId(i as u8);
            let health = self.healths[i];
            if health > 0 {
                let head_index = self.heads[i];
                let tail_index = self.get_cell(head_index).get_tail_position(head_index);
                if tail_index.is_none() {
                    return false;
                }
                let tail_index = tail_index.unwrap();
                let mut index = tail_index;
                while index != head_index {
                    let cell = self.get_cell(index);
                    if !cell.is_body_segment() {
                        return false;
                    }
                    if cell.get_snake_id() != Some(snake_id) {
                        return false;
                    }
                    let maybe_index = cell.get_next_index();
                    if maybe_index.is_none() {
                        return false;
                    }
                    index = maybe_index.unwrap();
                }
            }
        }
        true
    }

    /// packs this as a hash. Doing this because getting serde to work
    /// with const generics is hard
    pub fn pack_as_hash(&self) -> HashMap<String, Vec<u32>> {
        let mut hash = HashMap::new();
        hash.insert("hazard_damage".to_string(), vec![self.hazard_damage as u32]);
        hash.insert(
            "actual_width".to_string(),
            vec![self.get_actual_width() as u32],
        );
        hash.insert(
            "healths".to_string(),
            self.healths.iter().map(|x| *x as u32).collect(),
        );
        hash.insert(
            "lengths".to_string(),
            self.lengths.iter().map(|x| *x as u32).collect(),
        );
        hash.insert(
            "heads".to_string(),
            self.heads.iter().map(|x| x.as_usize() as u32).collect(),
        );
        hash.insert(
            "cells".to_string(),
            self.cells.iter().map(|x| x.pack_as_u32()).collect(),
        );
        hash
    }

    /// unpacks a packed hash repr back in to a CellBoard
    pub fn from_packed_hash(hash: &HashMap<String, Vec<u32>>) -> Self {
        let hazard_damage = hash.get("hazard_damage").unwrap()[0] as u8;
        let actual_width = hash.get("actual_width").unwrap()[0] as u8;
        let actual_height = hash
            .get("actual_height")
            .map(|h| h[0] as u8)
            .unwrap_or(actual_width);

        let mut healths = [0; MAX_SNAKES];
        let healths_iter = hash.get("healths").unwrap().iter().map(|x| *x as u8);
        for (idx, health) in healths_iter.enumerate() {
            healths[idx] = health;
        }

        let mut lengths = [0; MAX_SNAKES];
        let lengths_iter = hash.get("lengths").unwrap().iter().map(|x| *x as u16);
        for (idx, length) in lengths_iter.enumerate() {
            lengths[idx] = length;
        }

        let mut heads = [CellIndex::<T>::from_usize(0); MAX_SNAKES];
        let heads_iter = hash.get("heads").unwrap().iter().map(|x| *x as usize);
        for (idx, head) in heads_iter.enumerate() {
            heads[idx] = CellIndex::<T>::from_usize(head);
        }

        let mut cells = [Cell::<T>::empty(); BOARD_SIZE];
        let cells_iter = hash.get("cells").unwrap().iter().cloned();
        for (idx, cell) in cells_iter.enumerate() {
            cells[idx] = Cell::<T>::from_u32(cell);
        }

        let dimensions = D::from_dimensions(actual_width, actual_height);

        CellBoard {
            hazard_damage,
            cells,
            healths,
            heads,
            lengths,
            dimensions,
        }
    }

    pub fn as_wrapped_cell_index(&self, mut new_head_position: Position) -> CellIndex<T> {
        if self.off_board(new_head_position) {
            if new_head_position.x < 0 {
                debug_assert!(new_head_position.x == -1);
                debug_assert!(
                    new_head_position.y >= 0
                        && new_head_position.y < self.get_actual_height() as i32
                );
                new_head_position.x = self.get_actual_width() as i32 - 1;
            } else if new_head_position.x >= self.get_actual_width() as i32 {
                debug_assert!(new_head_position.x == self.get_actual_width() as i32);
                debug_assert!(
                    new_head_position.y >= 0
                        && new_head_position.y < self.get_actual_height() as i32
                );
                new_head_position.x = 0;
            } else if new_head_position.y < 0 {
                debug_assert!(new_head_position.y == -1);
                debug_assert!(
                    new_head_position.x >= 0
                        && new_head_position.x < self.get_actual_width() as i32
                );
                new_head_position.y = self.get_actual_height() as i32 - 1;
            } else if new_head_position.y >= self.get_actual_height() as i32 {
                debug_assert!(new_head_position.y == self.get_actual_height() as i32);
                debug_assert!(
                    new_head_position.x >= 0
                        && new_head_position.x < self.get_actual_width() as i32
                );
                new_head_position.y = 0;
            } else {
                panic!("We should never get here");
            }
            CellIndex::<T>::new(new_head_position, Self::width())
        } else {
            CellIndex::<T>::new(new_head_position, Self::width())
        }
    }

    pub fn get_actual_width(&self) -> u8 {
        self.dimensions.width()
    }

    pub fn get_actual_height(&self) -> u8 {
        self.dimensions.height()
    }

    fn kill(&mut self, sid: SnakeId) {
        self.healths[sid.0 as usize] = 0;
        self.heads[sid.0 as usize] = CellIndex::from_i32(0);
        self.lengths[sid.0 as usize] = 0;
    }

    fn kill_and_remove(&mut self, sid: SnakeId) {
        let head = self.heads[sid.as_usize()];
        let mut current_index = self.get_cell(head).get_tail_position(head);

        while let Some(i) = current_index {
            current_index = self.get_cell(i).get_next_index();
            debug_assert!(
                self.get_cell(i).get_snake_id().unwrap_or(sid).as_usize() == sid.as_usize()
            );
            self.cell_remove(i);
        }

        self.kill(sid);
    }

    /// Builds a cellboard from a given game, will return an error if the game doesn't match
    /// the provided BOARD_SIZE or MAX_SNAKES. You are encouraged to use `CellBoard4Snakes11x11`
    /// for the common game layout
    pub fn convert_from_game(game: Game, snake_ids: &SnakeIDMap) -> Result<Self, Box<dyn Error>> {
        if game.board.width * game.board.height > BOARD_SIZE as u32 {
            return Err("game size doesn't fit in the given board size".into());
        }

        if game.board.snakes.len() > MAX_SNAKES {
            return Err("too many snakes".into());
        }

        for snake in &game.board.snakes {
            let counts = &snake.body.iter().counts();
            if counts.values().any(|v| *v == TRIPLE_STACK) && counts.len() != 1 {
                return Err(format!("snake {} has a bad body stack (3 segs on same square and more than one unique position)", snake.id).into());
            }
        }
        let width = game.board.width as u8;
        let height = game.board.height as u8;

        let mut cells = [Cell::empty(); BOARD_SIZE];
        let mut healths: [u8; MAX_SNAKES] = [0; MAX_SNAKES];
        let mut heads: [CellIndex<T>; MAX_SNAKES] = [CellIndex::from_i32(0); MAX_SNAKES];
        let mut lengths: [u16; MAX_SNAKES] = [0; MAX_SNAKES];

        for snake in &game.board.snakes {
            let snake_id = match get_snake_id(snake, snake_ids) {
                Some(value) => value,
                None => continue,
            };

            healths[snake_id.0 as usize] = snake.health as u8;
            if snake.health == 0 {
                continue;
            }
            lengths[snake_id.0 as usize] = snake.body.len() as u16;

            let counts = &snake.body.iter().counts();

            let head_idx = CellIndex::new(snake.head, width);
            let mut next_index = head_idx;
            for (idx, pos) in snake.body.iter().unique().enumerate() {
                let cell_idx = CellIndex::new(*pos, width);
                let count = counts.get(pos).unwrap();
                if idx == 0 {
                    assert!(cell_idx == head_idx);
                    heads[snake_id.0 as usize] = head_idx;
                }
                cells[cell_idx.0.as_usize()] = if *count == TRIPLE_STACK {
                    Cell::make_triple_stacked_piece(snake_id)
                } else if *pos == snake.head {
                    // head can never be doubled, so let's assert it here, the cost of
                    // one comparison is worth the saftey imo
                    assert!(*count != DOUBLE_STACK);
                    let tail_index = CellIndex::new(*snake.body.back().unwrap(), width);
                    Cell::make_snake_head(snake_id, tail_index)
                } else if *count == DOUBLE_STACK {
                    Cell::make_double_stacked_piece(snake_id, next_index)
                } else {
                    Cell::make_body_piece(snake_id, next_index)
                };
                next_index = cell_idx;
            }
        }
        for y in 0..height {
            for x in 0..width {
                let position = Position {
                    x: x as i32,
                    y: y as i32,
                };
                let cell_idx: CellIndex<T> = CellIndex::new(position, width);

                if game.board.hazards.contains(&position) {
                    cells[cell_idx.0.as_usize()].set_hazard();
                }

                if game.board.food.contains(&position) {
                    cells[cell_idx.0.as_usize()].set_food();
                }
            }
        }

        let dimensions = D::from_dimensions(width, height);

        Ok(CellBoard {
            cells,
            heads,
            healths,
            lengths,
            dimensions,
            hazard_damage: game
                .game
                .ruleset
                .settings
                .as_ref()
                .map(|s| s.hazard_damage_per_turn)
                .unwrap_or(15) as u8,
        })
    }
    fn get_cell(&self, cell_index: CellIndex<T>) -> Cell<T> {
        self.cells[cell_index.0.as_usize()]
    }

    /// determines if a given position is not on the board
    pub fn off_board(&self, position: Position) -> bool {
        position.x < 0
            || position.x >= self.get_actual_width() as i32
            || position.y < 0
            || position.y >= self.get_actual_height() as i32
    }

    /// Get the length for a given snake
    pub fn get_length(&self, snake_id: SnakeId) -> u16 {
        self.lengths[snake_id.0 as usize]
    }
    /// Mutibaly call remove on the specified cell
    pub fn cell_remove(&mut self, cell_index: CellIndex<T>) {
        let mut old_cell = self.get_cell(cell_index);
        old_cell.remove();
        self.cells[cell_index.0.as_usize()] = old_cell;
    }
    /// Set the given index to a Snake Body Piece
    pub fn set_cell_body_piece(
        &mut self,
        cell_index: CellIndex<T>,
        sid: SnakeId,
        next_id: CellIndex<T>,
    ) {
        let mut old_cell = self.get_cell(cell_index);
        old_cell.set_body_piece(sid, next_id);
        self.cells[cell_index.0.as_usize()] = old_cell;
    }

    /// Set the given index as a double stacked snake
    pub fn set_cell_double_stacked(
        &mut self,
        cell_index: CellIndex<T>,
        sid: SnakeId,
        next_id: CellIndex<T>,
    ) {
        let mut old_cell = self.get_cell(cell_index);
        old_cell.set_double_stacked(sid, next_id);
        self.cells[cell_index.0.as_usize()] = old_cell;
    }

    /// Set the given index as a snake head
    pub fn set_cell_head(
        &mut self,
        old_head_index: CellIndex<T>,
        sid: SnakeId,
        next_id: CellIndex<T>,
    ) {
        let mut old_cell = self.get_cell(old_head_index);
        old_cell.set_head(sid, next_id);
        self.cells[old_head_index.0.as_usize()] = old_cell;
    }

    /// gets the snake ID at a given index, returns None if the provided index is not a snake cell
    pub fn get_snake_id_at(&self, index: CellIndex<T>) -> Option<SnakeId> {
        self.get_cell(index).get_snake_id()
    }

    /// determines if this cell is a hazard
    pub fn cell_is_hazard(&self, cell_idx: CellIndex<T>) -> bool {
        self.get_cell(cell_idx).is_hazard()
    }

    /// determines if this cell is a snake head (including triple stacked)
    pub fn cell_is_snake_head(&self, cell_idx: CellIndex<T>) -> bool {
        self.get_cell(cell_idx).is_head()
    }

    /// determines if this cell is a food
    pub fn cell_is_food(&self, cell_idx: CellIndex<T>) -> bool {
        self.get_cell(cell_idx).is_food()
    }

    /// determines if this cell is a snake body piece (including double stacked)
    pub fn cell_is_body(&self, cell_idx: CellIndex<T>) -> bool {
        self.get_cell(cell_idx).is_body()
    }

    pub fn cell_is_single_tail(&self, cell_idx: CellIndex<T>) -> bool {
        let cell = self.get_cell(cell_idx);
        if !cell.is_snake_body_piece()
            || cell.is_double_stacked_piece()
            || cell.is_triple_stacked_piece()
        {
            return false;
        }

        if let Some(sid) = cell.get_snake_id() {
            let head = self.heads[sid.0 as usize];

            self.get_cell(head).get_tail_position(head) == Some(cell_idx)
        } else {
            false
        }
    }

    /// determin the width of the CellBoard
    pub fn width() -> u8 {
        (BOARD_SIZE as f32).sqrt() as u8
    }
}

impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize> EmptyCellGettableGame
    for CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    fn get_empty_cells(&self) -> Box<dyn Iterator<Item = Self::NativePositionType> + '_> {
        Box::new(
            self.cells
                .iter()
                .enumerate()
                .filter(|(_, cell)| cell.is_empty())
                .map(|(idx, _)| CellIndex::from_usize(idx)),
        )
    }
}

impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
    StandardFoodPlaceableGame for CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    fn place_food(&mut self, rng: &mut impl rand::Rng) {
        // TODO: Get these constants from the game
        let min_food = 1;
        let food_spawn_chance = 0.15;

        // This is an optimization when min_food is 1. We know we don't need to spawn food if there if any of the board
        // so we can short circuit on the first food we find
        let food_to_add = if !self.cells.iter().any(|c| c.is_food()) {
            min_food
        } else {
            usize::from(rng.gen_bool(food_spawn_chance))
        };

        if food_to_add == 0 {
            return;
        }

        let empty = self.get_empty_cells();
        let random = empty.choose_multiple(rng, food_to_add);
        for pos in random {
            self.cells[pos.0.as_usize()].set_food();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::compact_representation::dimensions::Square;

    use super::CellBoard;
    #[test]
    fn test_assert_consistent() {
        let inconsistent_fixture = include_str!("../../../../fixtures/inconsistent_fixture.json");
        let hm = serde_json::from_str(inconsistent_fixture).unwrap();
        let game = CellBoard::<u8, Square, { 11 * 11 }, 4>::from_packed_hash(&hm);
        assert!(!game.assert_consistency());
    }
}
