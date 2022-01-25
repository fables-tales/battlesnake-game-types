//! A compact board representation that is efficient for simulation
pub use crate::compact_representation::eval::SinglePlayerMoveResult;
use crate::types::{
    build_snake_id_map, FoodGettableGame, HazardQueryableGame, HazardSettableGame,
    HeadGettableGame, HealthGettableGame, LengthGettableGame, PositionGettableGame,
    RandomReasonableMovesGame, SizeDeterminableGame, SnakeIDGettableGame, SnakeIDMap, SnakeId,
    VictorDeterminableGame, YouDeterminableGame,
};
/// you almost certainly want to use the `convert_from_game` method to
/// cast from a json represention to a `CellBoard`
use crate::types::{NeighborDeterminableGame, SnakeBodyGettableGame};
use crate::wire_representation::Game;
use itertools::Itertools;
use rand::prelude::IteratorRandom;
use rand::thread_rng;
use std::error::Error;
use std::fmt::Display;
use std::time::Instant;

use crate::{
    types::{Move, SimulableGame, SimulatorInstruments},
    wire_representation::Position,
};

use self::eval::MoveEvaluatableWithStateGame;

mod eval;

/// Wrapper type for numbers to allow for shrinking board sizes
pub trait CellNum:
    std::fmt::Debug + Copy + Clone + PartialEq + Eq + std::hash::Hash + Ord + Display
{
    /// converts this cellnum to a usize
    fn as_usize(&self) -> usize;
    /// makes a cellnum from an i32
    fn from_i32(i: i32) -> Self;
    /// makes a cellnum from an usize
    fn from_usize(i: usize) -> Self;
}

impl CellNum for u8 {
    fn as_usize(&self) -> usize {
        *self as usize
    }

    fn from_i32(i: i32) -> Self {
        i as u8
    }

    fn from_usize(i: usize) -> Self {
        i as u8
    }
}
impl CellNum for u16 {
    fn as_usize(&self) -> usize {
        *self as usize
    }

    fn from_i32(i: i32) -> Self {
        i as u16
    }

    fn from_usize(i: usize) -> Self {
        i as u16
    }
}

/// wrapper type for an index in to the board
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct CellIndex<T: CellNum>(pub T);

impl<T: CellNum> CellIndex<T> {
    /// makes a new cell index from a position, needs to know the width of the board
    pub fn new(pos: Position, width: u8) -> Self {
        Self(T::from_i32(pos.y * width as i32 + pos.x))
    }

    /// makes a cellindex from an i32
    pub fn from_i32(i: i32) -> Self {
        Self(T::from_i32(i))
    }

    /// converts a cellindex to a position
    pub fn into_position(self, width: u8) -> Position {
        let y = (self.0.as_usize() as i32 / width as i32) as i32;
        let x = (self.0.as_usize() as i32 % width as i32) as i32;
        Position { x, y }
    }

    /// Returns the CellIndex from moving in the direction of Move
    pub fn in_direction(&self, m: &Move, width: u8) -> Self {
        Self::new(self.into_position(width).add_vec(m.to_vector()), width)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Cell<T: CellNum> {
    flags: u8,
    id: SnakeId,
    idx: CellIndex<T>,
}

const SNAKE_HEAD: u8 = 0x06;
const SNAKE_BODY_PIECE: u8 = 0x01;
const DOUBLE_STACKED_PIECE: u8 = 0x02;
const TRIPLE_STACKED_PIECE: u8 = 0x03;
const FOOD: u8 = 0x04;
const EMPTY: u8 = 0x05;
const KIND_MASK: u8 = 0x07;

const IS_HAZARD: u8 = 0x10;

const TRIPLE_STACK: usize = 3;
const DOUBLE_STACK: usize = 2;

impl<T: CellNum> Cell<T> {
    pub fn get_tail_position(&self, ci: CellIndex<T>) -> Option<CellIndex<T>> {
        if self.is_head() {
            if self.is_triple_stacked_piece() {
                Some(ci)
            } else {
                Some(self.idx)
            }
        } else {
            None
        }
    }

    fn is_empty(&self) -> bool {
        self.flags & KIND_MASK == EMPTY
    }

    fn get_next_index(&self) -> Option<CellIndex<T>> {
        if self.is_snake_body_piece() || self.is_double_stacked_piece() {
            Some(self.idx)
        } else {
            None
        }
    }

    fn is_food(&self) -> bool {
        self.flags & KIND_MASK == FOOD
    }

    fn set_hazard(&mut self) {
        self.flags |= IS_HAZARD
    }

    fn clear_hazard(&mut self) {
        self.flags &= !IS_HAZARD
    }

    fn is_hazard(&self) -> bool {
        self.flags & IS_HAZARD != 0
    }

    fn is_body_segment(&self) -> bool {
        self.is_snake_body_piece()
            || self.is_double_stacked_piece()
            || self.is_triple_stacked_piece()
    }

    fn is_head(&self) -> bool {
        self.flags & KIND_MASK == SNAKE_HEAD || self.is_triple_stacked_piece()
    }

    fn remove_snake(&mut self) {
        if self.is_head() || self.is_body_segment() {
            self.remove();
        }
    }

    /// resets a cell to empty preserving the cell's hazard status
    fn remove(&mut self) {
        let reset_to_empty = (self.flags & !KIND_MASK) | EMPTY;
        self.flags = reset_to_empty;
        self.id = SnakeId(0);
        self.idx = CellIndex(T::from_i32(0));
    }

    fn is_stacked(&self) -> bool {
        self.is_double_stacked_piece() || self.is_triple_stacked_piece()
    }

    fn empty() -> Self {
        Cell {
            flags: EMPTY,
            id: SnakeId(0),
            idx: CellIndex(T::from_i32(0)),
        }
    }

    fn make_snake_head(sid: SnakeId, tail_index: CellIndex<T>) -> Self {
        Cell {
            flags: SNAKE_HEAD,
            id: sid,
            idx: tail_index,
        }
    }

    fn make_body_piece(sid: SnakeId, next_index: CellIndex<T>) -> Self {
        Cell {
            flags: SNAKE_BODY_PIECE,
            id: sid,
            idx: next_index,
        }
    }

    fn make_double_stacked_piece(sid: SnakeId, next_index: CellIndex<T>) -> Self {
        Cell {
            flags: DOUBLE_STACKED_PIECE,
            id: sid,
            idx: next_index,
        }
    }

    fn make_triple_stacked_piece(sid: SnakeId) -> Self {
        Cell {
            flags: TRIPLE_STACKED_PIECE,
            id: sid,
            idx: CellIndex(T::from_i32(0)),
        }
    }

    fn is_snake_body_piece(&self) -> bool {
        self.flags & KIND_MASK == SNAKE_BODY_PIECE
    }

    fn is_double_stacked_piece(&self) -> bool {
        self.flags & KIND_MASK == DOUBLE_STACKED_PIECE
    }

    fn is_triple_stacked_piece(&self) -> bool {
        self.flags & KIND_MASK == TRIPLE_STACKED_PIECE
    }

    fn is_body(&self) -> bool {
        self.flags & KIND_MASK == SNAKE_BODY_PIECE || self.flags & KIND_MASK == DOUBLE_STACKED_PIECE
    }

    fn set_food(&mut self) {
        self.flags = (self.flags & !KIND_MASK) | FOOD;
    }

    fn set_head(&mut self, sid: SnakeId, tail_index: CellIndex<T>) {
        self.flags = (self.flags & !KIND_MASK) | SNAKE_HEAD;
        self.id = sid;
        self.idx = tail_index;
    }

    fn set_body_piece(&mut self, sid: SnakeId, next_pos: CellIndex<T>) {
        self.flags = (self.flags & !KIND_MASK) | SNAKE_BODY_PIECE;
        self.id = sid;
        self.idx = next_pos;
    }

    fn set_double_stacked(&mut self, sid: SnakeId, next_pos: CellIndex<T>) {
        self.flags = (self.flags & !KIND_MASK) | DOUBLE_STACKED_PIECE;
        self.id = sid;
        self.idx = next_pos;
    }

    fn get_snake_id(&self) -> Option<SnakeId> {
        if self.is_body_segment() || self.is_head() {
            Some(self.id)
        } else {
            None
        }
    }
}

/// A compact board representation that is significantly faster for simulation than
/// `battlesnake_game_types::wire_representation::Game`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CellBoard<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> {
    hazard_damage: u8,
    cells: [Cell<T>; BOARD_SIZE],
    healths: [u8; MAX_SNAKES],
    heads: [CellIndex<T>; MAX_SNAKES],
    lengths: [u16; MAX_SNAKES],
    actual_width: u8,
}

/// 7x7 board with 4 snakes
pub type CellBoard4Snakes7x7 = CellBoard<u8, { 7 * 7 }, 4>;

/// Used to represent the standard 11x11 game with up to 4 snakes.
pub type CellBoard4Snakes11x11 = CellBoard<u8, { 11 * 11 }, 4>;

/// Used to represent the a 15x15 board with up to 4 snakes. This is the biggest board size that
/// can still use u8s
pub type CellBoard8Snakes15x15 = CellBoard<u8, { 15 * 15 }, 8>;

/// Used to represent the largest UI Selectable board with 8 snakes.
pub type CellBoard8Snakes25x25 = CellBoard<u16, { 25 * 25 }, 8>;

/// Used to represent an absolutely silly game board
pub type CellBoard16Snakes50x50 = CellBoard<u16, { 50 * 50 }, 16>;

/// Enum that holds a Cell Board sized right for the given game
#[derive(Debug)]
pub enum BestCellBoard {
    #[allow(missing_docs)]
    Tiny(Box<CellBoard4Snakes7x7>),
    #[allow(missing_docs)]
    Standard(Box<CellBoard4Snakes11x11>),
    #[allow(missing_docs)]
    LargestU8(Box<CellBoard8Snakes15x15>),
    #[allow(missing_docs)]
    Large(Box<CellBoard8Snakes25x25>),
    #[allow(missing_docs)]
    Silly(Box<CellBoard16Snakes50x50>),
}

/// Trait to get the best sized cellboard for the given game. It returns the smallest Compact board
/// that has enough room to fit the given Wire game. If the game can't fit in any of our Compact
/// boards we panic. However the largest board available is MUCH larger than the biggest selectable
/// board in the Battlesnake UI
pub trait ToBestCellBoard {
    #[allow(missing_docs)]
    fn to_best_cell_board(self) -> Result<BestCellBoard, Box<dyn Error>>;
}

impl ToBestCellBoard for Game {
    fn to_best_cell_board(self) -> Result<BestCellBoard, Box<dyn Error>> {
        let dimension = self.board.width;
        let num_snakes = self.board.snakes.len();
        let id_map = build_snake_id_map(&self);

        let best_board = if dimension <= 7 && num_snakes <= 4 {
            BestCellBoard::Tiny(Box::new(CellBoard4Snakes7x7::convert_from_game(
                self, &id_map,
            )?))
        } else if dimension <= 11 && num_snakes <= 4 {
            BestCellBoard::Standard(Box::new(CellBoard4Snakes11x11::convert_from_game(
                self, &id_map,
            )?))
        } else if dimension <= 15 && num_snakes <= 8 {
            BestCellBoard::LargestU8(Box::new(CellBoard8Snakes15x15::convert_from_game(
                self, &id_map,
            )?))
        } else if dimension <= 25 && num_snakes <= 8 {
            BestCellBoard::Large(Box::new(CellBoard8Snakes25x25::convert_from_game(
                self, &id_map,
            )?))
        } else if dimension <= 50 && num_snakes <= 16 {
            BestCellBoard::Silly(Box::new(CellBoard16Snakes50x50::convert_from_game(
                self, &id_map,
            )?))
        } else {
            panic!("No board was big enough")
        };

        Ok(best_board)
    }
}

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> Display
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = self.actual_width;
        let height = self.actual_height();
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
                    write!(f, "H")?;
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
        Ok(())
    }
}

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

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
    CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{

    fn actual_height(&self) -> u8 {
        self.actual_width
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
        for y in 0..game.board.height {
            for x in 0..game.board.width {
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

        Ok(CellBoard {
            cells,
            heads,
            healths,
            lengths,
            actual_width: game.board.width as u8,
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
            || position.x >= self.actual_width as i32
            || position.y < 0
            || position.y >= self.actual_height() as i32
    }

    /// Get the health for a given snake
    pub fn get_health(&self, snake_id: SnakeId) -> u8 {
        self.healths[snake_id.0 as usize]
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

    /// Mutibaly call remove_snake on the specified cell
    pub fn cell_remove_snake(&mut self, cell_index: CellIndex<T>) {
        let mut old_cell = self.get_cell(cell_index);
        old_cell.remove_snake();
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
    pub fn set_cell_head(&mut self, cell_index: CellIndex<T>, sid: SnakeId, next_id: CellIndex<T>) {
        let mut old_cell = self.get_cell(cell_index);
        old_cell.set_head(sid, next_id);
        self.cells[cell_index.0.as_usize()] = old_cell;
    }

    /// gets the snake ID at a given index, returns None if the provided index is not a snake cell
    pub fn get_snake_id_at(&self, index: CellIndex<T>) -> Option<SnakeId> {
        self.get_cell(index).get_snake_id()
    }

    /// Determines if this cell contains exactly a snake's body piece, ignoring heads, double stacks and triple stacks
    pub fn cell_is_snake_body_piece(&self, current_index: CellIndex<T>) -> bool {
        self.get_cell(current_index).is_snake_body_piece()
    }

    /// determines if this cell is double stacked (e.g. a tail that has hit a food)
    pub fn cell_is_double_stacked_piece(&self, current_index: CellIndex<T>) -> bool {
        self.get_cell(current_index).is_double_stacked_piece()
    }

    /// determines if this cell is triple stacked (the snake at the start of the game)
    pub fn cell_is_triple_stacked_piece(&self, current_index: CellIndex<T>) -> bool {
        self.get_cell(current_index).is_triple_stacked_piece()
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

    /// determin the width of the CellBoard
    pub fn width() -> u8 {
        (BOARD_SIZE as f32).sqrt() as u8
    }

    /// Get all the hazards for this board
    pub fn get_all_hazards_as_positions(&self) -> Vec<crate::wire_representation::Position> {
        self.cells
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_hazard())
            .map(|(i, _)| CellIndex(T::from_usize(i)).into_position(Self::width()))
            .collect()
    }
}

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

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> PositionGettableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    type NativePositionType = CellIndex<T>;

    fn position_is_snake_body(&self, pos: Self::NativePositionType) -> bool {
        let cell = self.get_cell(pos);

        cell.is_body_segment()
    }

    fn position_from_native(&self, pos: Self::NativePositionType) -> Position {
        let width = self.actual_width;

        pos.into_position(width)
    }

    fn native_from_position(&self, pos: Position) -> Self::NativePositionType {
        Self::NativePositionType::new(pos, self.actual_width)
    }
}

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> HazardQueryableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn is_hazard(&self, pos: &Self::NativePositionType) -> bool {
        self.cell_is_hazard(*pos)
    }
}

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> HazardSettableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn set_hazard(&mut self, pos: Self::NativePositionType) {
        self.cells[pos.0.as_usize()].set_hazard();
    }

    fn clear_hazard(&mut self, pos: Self::NativePositionType) {
        self.cells[pos.0.as_usize()].clear_hazard();
    }
}

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> HeadGettableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn get_head_as_position(
        &self,
        snake_id: &Self::SnakeIDType,
    ) -> crate::wire_representation::Position {
        let idx = self.heads[snake_id.0.as_usize()];
        let width = self.actual_width;
        idx.into_position(width)
    }

    fn get_head_as_native_position(
        &self,
        snake_id: &Self::SnakeIDType,
    ) -> Self::NativePositionType {
        self.heads[snake_id.0.as_usize()]
    }
}

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> FoodGettableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn get_all_food_as_positions(&self) -> Vec<crate::wire_representation::Position> {
        self.cells
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_food())
            .map(|(i, _)| CellIndex(T::from_usize(i)).into_position(self.actual_width))
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

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> YouDeterminableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn is_you(&self, snake_id: &Self::SnakeIDType) -> bool {
        snake_id.0 == 0
    }

    fn you_id(&self) -> &Self::SnakeIDType {
        &SnakeId(0)
    }
}

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> LengthGettableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    type LengthType = u16;

    fn get_length(&self, snake_id: &Self::SnakeIDType) -> Self::LengthType {
        self.lengths[snake_id.0.as_usize()]
    }

    fn get_length_i64(&self, snake_id: &Self::SnakeIDType) -> i64 {
        self.get_length(*snake_id) as i64
    }
}

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> HealthGettableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    type HealthType = u8;
    const ZERO: Self::HealthType = 0;

    fn get_health(&self, snake_id: &Self::SnakeIDType) -> Self::HealthType {
        self.healths[snake_id.0.as_usize()]
    }

    fn get_health_i64(&self, snake_id: &Self::SnakeIDType) -> i64 {
        self.get_health(*snake_id) as i64
    }
}

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> VictorDeterminableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
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

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> RandomReasonableMovesGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn random_reasonable_move_for_each_snake(&self) -> Vec<(Self::SnakeIDType, Move)> {
        let width = self.actual_width;
        self.healths
            .iter()
            .enumerate()
            .filter(|(_, health)| **health > 0)
            .map(|(idx, _)| {
                let head = self.heads[idx];
                let head_pos = head.into_position(width);

                let mv = Move::all()
                    .into_iter()
                    .filter(|mv| {
                        let new_head = head_pos.add_vec(mv.to_vector());
                        let ci = CellIndex::new(head_pos.add_vec(mv.to_vector()), width);

                        !self.off_board(new_head)
                            && !self.get_cell(ci).is_body_segment()
                            && !self.get_cell(ci).is_head()
                    })
                    .choose(&mut thread_rng())
                    .unwrap_or(Move::Up);
                (SnakeId(idx as u8), mv)
            })
            .collect_vec()
    }
}

impl<T: SimulatorInstruments, N: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
    SimulableGame<T> for CellBoard<N, BOARD_SIZE, MAX_SNAKES>
{
    #[allow(clippy::type_complexity)]
    fn simulate_with_moves(
        &self,
        instruments: &T,
        snake_ids_and_moves: Vec<(Self::SnakeIDType, Vec<crate::types::Move>)>,
    ) -> Vec<(Vec<(Self::SnakeIDType, crate::types::Move)>, Self)> {
        let start = Instant::now();
        let eval_state = self.generate_state(snake_ids_and_moves.clone());
        let ids_and_moves = snake_ids_and_moves
            .into_iter()
            .filter(|(_, moves)| !moves.is_empty())
            .map(|(sid, moves)| {
                let first_move = moves[0];
                let mut valid_moves = moves
                    .into_iter()
                    // Here we ignore any moves that results in instant death. UNLESS its the only move
                    .filter(|mv| {
                        matches!(
                            eval_state[sid.as_usize()][mv.as_index()],
                            SinglePlayerMoveResult::Alive { .. }
                        )
                    })
                    .collect_vec();

                // If we filtered out all the moves we pick a random move to keep
                if valid_moves.is_empty() {
                    valid_moves.push(first_move);
                }

                std::iter::repeat(sid).zip(valid_moves)
            });
        let possible_new_games = ids_and_moves.multi_cartesian_product();
        let res = possible_new_games
            .into_iter()
            .map(|new_snakes| {
                let output = self.evaluate_moves_with_state(&new_snakes, &eval_state);

                (new_snakes, output)
            })
            .collect();
        instruments.observe_simulation(start.elapsed());

        res
    }
}

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> NeighborDeterminableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn possible_moves(
        &self,
        pos: &Self::NativePositionType,
    ) -> Vec<(Move, Self::NativePositionType)> {
        let width = self.actual_width;

        Move::all()
            .into_iter()
            .map(|mv| {
                let head_pos = pos.into_position(width);
                let new_head = head_pos.add_vec(mv.to_vector());
                let ci = CellIndex::new(new_head, width);

                (mv, new_head, ci)
            })
            .filter(|(_mv, new_head, _)| !self.off_board(*new_head))
            .map(|(mv, _, ci)| (mv, ci))
            .collect()
    }

    fn neighbors(&self, pos: &Self::NativePositionType) -> std::vec::Vec<Self::NativePositionType> {
        let width = self.actual_width;

        Move::all()
            .into_iter()
            .map(|mv| {
                let head_pos = pos.into_position(width);
                let new_head = head_pos.add_vec(mv.to_vector());
                let ci = CellIndex::new(new_head, width);

                (new_head, ci)
            })
            .filter(|(new_head, _)| !self.off_board(*new_head))
            .map(|(_, ci)| ci)
            .collect()
    }
}

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

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> SizeDeterminableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn get_width(&self) -> u32 {
        (BOARD_SIZE as f32).sqrt() as u32
    }

    fn get_height(&self) -> u32 {
        (BOARD_SIZE as f32).sqrt() as u32
    }
}

#[cfg(test)]
mod test {

    use std::collections::HashMap;

    use super::*;
    use crate::{
        types::{build_snake_id_map, SnakeIDGettableGame, VictorDeterminableGame},
        wire_representation::Game as DEGame, game_fixture,
    };
    #[derive(Debug)]
    struct Instruments;
    impl SimulatorInstruments for Instruments {
        fn observe_simulation(&self, _: std::time::Duration) {}
    }

    #[test]
    fn test_compact_board_conversion() {
        let start_of_game_fixture = game_fixture(include_str!("../../fixtures/start_of_game.json"));
        let converted = Game::to_best_cell_board(start_of_game_fixture);
        assert!(converted.is_ok());
        let u = converted.unwrap();
        match u {
            BestCellBoard::Standard(_) => {},
            _ => panic!("expected standard board"),
        }

        let tiny_board = game_fixture(include_str!("../../fixtures/7x7board.json"));
        let converted = Game::to_best_cell_board(tiny_board);
        assert!(converted.is_ok());
        let u = converted.unwrap();
        match u {
            BestCellBoard::Tiny(_) => {},
            _ => panic!("expected standard board"),
        }
    }
    /// I think this test shows that the Wire has a bug currently. Need to compare with the Go
    /// version. Maybe I need to break out my Fuzzer stuff again.
    #[test]
    fn test_all_options_dead() {
        let game_fixture =
            include_str!("../../fixtures/all-options-dead-prefer-out-of-bounds.json");
        let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
        let g = g.expect("the json literal is valid");
        test_simulation_equivalents(g);
    }

    #[test]
    fn test_compare_simulators() {
        let game_fixture = include_str!("../../fixtures/tree_search_collision.json");
        let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
        let g = g.expect("the json literal is valid");
        test_simulation_equivalents(g);
    }

    #[test]
    fn test_compare_simulators_start() {
        let game_fixture = include_str!("../../fixtures/start_of_game.json");
        let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
        let g = g.expect("the json literal is valid");
        test_simulation_equivalents(g);
    }

    #[test]
    fn test_this_crash() {
        let game_fixture = include_str!("../../fixtures/this_one_crashed.json");
        let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
        let g = g.expect("the json literal is valid");
        test_simulation_equivalents(g);
    }

    #[test]
    fn test_another_crash() {
        let game_fixture = include_str!("../../fixtures/another_crash.json");
        let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
        let g = g.expect("the json literal is valid");
        test_simulation_equivalents(g);
    }

    #[test]
    fn test_head_gettable() {
        let game_fixture = include_str!("../../fixtures/late_stage.json");
        let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
        let g = g.expect("the json literal is valid");
        let snake_id_mapping = build_snake_id_map(&g);
        let compact: CellBoard4Snakes11x11 = g.as_cell_board(&snake_id_mapping).unwrap();
        assert_eq!(
            compact.get_head_as_position(&SnakeId(0)),
            Position { x: 4, y: 6 }
        );
        assert_eq!(
            compact.get_head_as_native_position(&SnakeId(0)),
            CellIndex(6 * 11 + 4)
        );
    }

    #[test]
    fn test_bench_compact_late_stage() {
        let game_fixture = include_str!("../../fixtures/late_stage.json");
        let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
        let g = g.expect("the json literal is valid");
        let snake_id_mapping = build_snake_id_map(&g);
        let compact: CellBoard4Snakes11x11 = g.as_cell_board(&snake_id_mapping).unwrap();
        let snake_ids = compact.get_snake_ids();
        eprintln!("sids: {:?}", snake_ids);
        let instruments = Instruments;
        for _ in 0..100 {
            compact.simulate(&instruments, snake_ids.clone());
        }
        let start_time = Instant::now();
        for _ in 0..100000 {
            compact.simulate(&instruments, snake_ids.clone());
        }
        eprintln!("{:?}", start_time.elapsed());
    }

    #[test]
    fn test_tail_collision() {
        let game_fixture = include_str!("../../fixtures/start_of_game.json");
        let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
        let g = g.expect("the json literal is valid");
        let snake_id_mapping = build_snake_id_map(&g);
        let mut compact: CellBoard4Snakes11x11 = g.as_cell_board(&snake_id_mapping).unwrap();
        let moves = [
            Move::Left,
            Move::Down,
            Move::Right,
            Move::Up,
            Move::Left,
            Move::Down,
        ];
        let instruments = Instruments;
        eprintln!("{}", compact);
        for mv in moves {
            let res = compact.simulate_with_moves(&instruments, vec![(SnakeId(0), vec![mv])]);
            compact = res[0].1.clone();
            eprintln!("{}", compact);
        }
        assert!(compact.healths[0] > 0);
    }

    #[test]
    fn test_set_hazard() {
        let mut c: Cell<u8> = Cell::empty();
        c.set_food();
        assert!(c.is_food());
        c.set_hazard();
        eprintln!("{:#08b}", c.flags);
        assert!(c.is_food());
        assert!(c.is_hazard());
        assert!(!c.is_head());
        assert!(!c.is_body());
    }

    #[test]
    fn test_clear_hazard() {
        let mut c: Cell<u8> = Cell::empty();
        c.set_food();
        assert!(c.is_food());
        c.set_hazard();
        c.clear_hazard();
        assert!(c.is_food());
        assert!(!c.is_hazard());
        assert!(!c.is_head());
        assert!(!c.is_body());
        let mut c: Cell<u8> = Cell::make_double_stacked_piece(SnakeId(0), CellIndex(0));
        c.set_hazard();
        c.clear_hazard();
        assert!(c.is_body());
        assert!(!c.is_hazard());
    }

    #[test]
    fn test_remove() {
        let mut c: Cell<u8> = Cell::make_body_piece(SnakeId(3), CellIndex(17));
        c.remove();
        c.set_hazard();
        eprintln!("{:#08b}", c.flags);
        assert!(c.is_empty());
        assert!(c.is_hazard());
        assert!(c.id == SnakeId(0));
        assert!(c.idx == CellIndex(0));
    }
    #[test]
    fn test_set_food() {
        let mut c: Cell<u8> = Cell::empty();
        c.set_food();
        c.set_hazard();
        eprintln!("{:#08b}", c.flags);
        assert!(c.is_food());
        assert!(c.is_hazard());
        assert!(c.id == SnakeId(0));
        assert!(c.idx == CellIndex(0));
    }

    #[test]
    fn test_set_head() {
        let mut c: Cell<u8> = Cell::empty();
        c.set_head(SnakeId(3), CellIndex(17));
        c.set_hazard();
        eprintln!("{:#08b}", c.flags);
        assert!(c.is_head());
        assert!(c.is_hazard());
        assert!(c.id == SnakeId(3));
        assert!(c.idx == CellIndex(17));
    }

    #[test]
    fn test_playout() {
        for _ in 0..100 {
            let game_fixture = include_str!("../../fixtures/start_of_game.json");
            let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
            let mut g = g.expect("the json literal is valid");
            let snake_id_mapping = build_snake_id_map(&g);
            let instruments = Instruments;
            let mut compact: CellBoard4Snakes11x11 = g.as_cell_board(&snake_id_mapping).unwrap();
            while !g.is_over() {
                let orig = g.clone();
                let moves = g.random_reasonable_move_for_each_snake();
                eprintln!("moves: {:?}", &moves);
                let non_compact_move_map = moves
                    .into_iter()
                    .map(|(id, mv)| (id, vec![mv]))
                    .collect_vec();
                let compact_move_map = non_compact_move_map
                    .iter()
                    .cloned()
                    .map(|(id, mvs)| (*snake_id_mapping.get(&id).unwrap(), mvs))
                    .collect_vec();
                let non_compact_next = g.simulate_with_moves(&instruments, non_compact_move_map);
                let compact_next = compact.simulate_with_moves(&instruments, compact_move_map);
                assert_eq!(non_compact_next.len(), 1);
                assert_eq!(compact_next.len(), 1);
                g = non_compact_next[0].clone().1;
                compact = compact_next[0].1.clone();
                if g.is_over() {
                    eprintln!("orig: {}", orig.board);
                    break;
                }
                let compare_compact: CellBoard4Snakes11x11 =
                    g.as_cell_board(&snake_id_mapping).unwrap();
                if compare_compact != compact {
                    eprintln!("--------\n\norig: {}", orig.board);
                    eprintln!(
                        "compact: {} {:?} g: {}------------------",
                        compact, compact.healths, g.board
                    );
                }
                assert_eq!(compare_compact, compact);
            }
            eprintln!(
                "--------\n\ncompact: {} {:?} g: {}------------------",
                compact, compact.healths, g.board
            );
            assert!(compact.is_over());
        }
    }

    fn test_simulation_equivalents(g: DEGame) {
        let snake_id_mapping = build_snake_id_map(&g);
        let non_compact_res = g.simulate(&Instruments, g.get_snake_ids());
        let snake_id_map = build_snake_id_map(&g);
        let compact = CellBoard4Snakes11x11::convert_from_game(g.clone(), &snake_id_map.clone());
        let compact = compact.unwrap();
        if !format!("{}", g.board).starts_with(&format!("{}", compact)) {
            eprintln!("{}", g.board);
            eprintln!("{}", compact);
            eprintln!("{:?}", compact.healths);
            eprintln!(
                "{:?}",
                compact
                    .heads
                    .iter()
                    .map(|h| h.into_position(g.board.width as u8))
                    .collect_vec()
            );
            eprintln!(
                "{:?}",
                compact
                    .heads
                    .iter()
                    .map(|h| compact.get_cell(*h))
                    .collect_vec()
            );
        }
        assert!(
            format!("{}", g.board).starts_with(&format!("{}", compact)) || compact.healths[0] == 0
        );
        compare_simulated_games(
            &snake_id_mapping,
            &g,
            non_compact_res.clone(),
            compact.clone(),
        );
        let non_compact_lookup =
            build_non_compact_lookup(snake_id_mapping.clone(), non_compact_res);

        let compact_results = compact.simulate(
            &Instruments,
            snake_id_mapping.values().map(|s| *s).collect_vec(),
        );
        for (moves, compact_game) in &compact_results {
            if compact_game.healths.iter().filter(|h| **h > 0).count() > 1 {
                eprintln!("{:?}", moves);
                let non_compact_game = non_compact_lookup.get(&to_map_key(&moves)).unwrap();
                let non_compact_res =
                    non_compact_game.simulate(&Instruments, non_compact_game.get_snake_ids());
                compare_simulated_games(
                    &snake_id_mapping,
                    non_compact_game,
                    non_compact_res,
                    compact_game.clone(),
                );
            }
        }
    }

    fn compare_simulated_games(
        snake_id_mapping: &HashMap<String, SnakeId>,
        non_compact_game: &Game,
        non_compact_res: Vec<(Vec<(String, Move)>, DEGame)>,
        compact: CellBoard4Snakes11x11,
    ) {
        let compact_results = compact.simulate(
            &Instruments,
            snake_id_mapping.values().map(|s| *s).collect_vec(),
        );
        assert!(compact_results.len() <= non_compact_res.len()); // TODO: We need to apply the same optimization about walls to the non-compact version to make sure they eliminate the same moves
        let non_compact_lookup =
            build_non_compact_lookup(snake_id_mapping.clone(), non_compact_res);
        for (moves, compact_game) in &compact_results {
            let moves = to_map_key(moves);
            let corresponding_game = non_compact_lookup.get(&moves);
            if corresponding_game.is_none() {
                continue;
            }
            let corresponding_game = corresponding_game.unwrap();
            eprintln!("moves: {:?}", moves);
            eprintln!(
                "-----original: {}, compact: {}\n, actual: {}\n-------",
                non_compact_game.board, compact_game, corresponding_game.board
            );
            eprintln!("compact_game: {:?}", compact_game.healths);
            assert!(
                format!("{}", corresponding_game.board).starts_with(&format!("{}", compact_game))
                    || compact_game.healths[0] == 0
            );
            if !compact_game.healths[0] == 0 {
                assert_eq!(
                    *compact_game,
                    CellBoard4Snakes11x11::convert_from_game(
                        corresponding_game.clone(),
                        snake_id_mapping
                    )
                    .unwrap(),
                )
            }
        }
    }

    fn build_non_compact_lookup(
        snake_id_mapping: HashMap<String, SnakeId>,
        non_compact_res: Vec<(Vec<(String, Move)>, DEGame)>,
    ) -> HashMap<Vec<(SnakeId, Move)>, DEGame> {
        let mut non_compact_lookup = HashMap::new();
        for (move_map, game) in non_compact_res {
            let move_map = move_map
                .into_iter()
                .map(|(id, mv)| (snake_id_mapping.get(&id).unwrap().clone(), mv))
                .collect::<Vec<_>>();
            let move_map = to_map_key(&move_map);
            non_compact_lookup.insert(move_map, game);
        }
        non_compact_lookup
    }

    fn to_map_key(mv_map: &Vec<(SnakeId, Move)>) -> Vec<(SnakeId, Move)> {
        mv_map
            .clone()
            .into_iter()
            .sorted_by_key(|(id, _mv)| id.0)
            .collect::<Vec<_>>()
    }
}
