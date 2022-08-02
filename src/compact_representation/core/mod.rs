mod cell_board;
mod cell_num;
mod impl_common;
mod simulate;

use crate::{
    types::{Move, SnakeId},
    wire_representation::Position,
};

pub use cell_board::{CellBoard, EvaluateMode};
pub use cell_num::CellNum;
pub use simulate::simulate_with_moves;

/// wrapper type for an index in to the board
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct CellIndex<T: CellNum>(pub T);

impl<T: CellNum> CellIndex<T> {
    /// makes a new cell index from a position, needs to know the width of the board
    pub fn new(pos: Position, width: u8) -> Self {
        Self(T::from_i32(pos.y * width as i32 + pos.x))
    }

    /// build a CellIndex from a usize
    pub fn from_usize(u: usize) -> Self {
        Self(T::from_usize(u))
    }

    /// makes a cellindex from an i32
    pub fn from_i32(i: i32) -> Self {
        Self(T::from_i32(i))
    }

    /// build a CellIndex from a u32
    pub fn from_u32(u: u32) -> Self {
        Self(T::from_usize(u as usize))
    }

    /// get a usize from a CellIndex
    pub fn as_usize(&self) -> usize {
        self.0.as_usize()
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

pub const TRIPLE_STACK: usize = 3;
pub const DOUBLE_STACK: usize = 2;

use super::dimensions;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
enum CellFlag {
    Empty = 0x05,
    SnakeHead = 0x06,
    SnakeBodyPiece = 0x01,
    DoubleStackedPiece = 0x02,
    TripleStackedPiece = 0x03,
    Food = 0x04,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Cell<T: CellNum> {
    flags: CellFlag,
    hazard_count: u8,
    id: SnakeId,
    idx: CellIndex<T>,
}

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

    pub fn pack_as_u32(&self) -> u32 {
        let mut value: u32 = 0;
        // flags is a byte
        value |= self.flags as u32;
        // ids are actually a u8
        value |= ((self.id.as_usize() as u32) & 0xff) << 8;
        // idx is at most a u16
        value |= ((self.idx.0.as_usize() as u32) & 0xffff) << 16;
        value
    }

    pub fn from_u32(value: u32) -> Self {
        let flags = unsafe { std::mem::transmute::<_, CellFlag>((value & 0xff) as u8) };
        let id = SnakeId(((value >> 8) & 0xff) as u8);
        let idx = CellIndex::from_u32((value >> 16) & 0xffff);

        Self {
            flags,
            id,
            idx,
            hazard_count: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.flags == CellFlag::Empty
    }

    pub fn get_next_index(&self) -> Option<CellIndex<T>> {
        if self.is_snake_body_piece() || self.is_double_stacked_piece() {
            Some(self.idx)
        } else {
            None
        }
    }

    pub fn is_food(&self) -> bool {
        self.flags == CellFlag::Food
    }

    pub fn set_hazard(&mut self) {
        self.hazard_count = 1;
    }

    #[allow(dead_code)]
    pub fn set_hazard_count(&mut self, count: u8) {
        self.hazard_count = count;
    }

    pub fn clear_hazard(&mut self) {
        self.hazard_count = 0;
    }

    pub fn is_hazard(&self) -> bool {
        self.hazard_count > 0
    }

    pub fn is_body_segment(&self) -> bool {
        self.is_snake_body_piece()
            || self.is_double_stacked_piece()
            || self.is_triple_stacked_piece()
    }

    pub fn is_head(&self) -> bool {
        self.flags == CellFlag::SnakeHead || self.is_triple_stacked_piece()
    }

    /// resets a cell to empty preserving the cell's hazard status
    pub fn remove(&mut self) {
        self.flags = CellFlag::Empty;
        self.id = SnakeId(0);
        self.idx = CellIndex(T::from_i32(0));
    }

    pub fn is_stacked(&self) -> bool {
        self.is_double_stacked_piece() || self.is_triple_stacked_piece()
    }

    pub fn empty() -> Self {
        Cell {
            flags: CellFlag::Empty,
            hazard_count: 0,
            id: SnakeId(0),
            idx: CellIndex(T::from_i32(0)),
        }
    }

    pub fn make_snake_head(sid: SnakeId, tail_index: CellIndex<T>) -> Self {
        Cell {
            flags: CellFlag::SnakeHead,
            id: sid,
            idx: tail_index,
            hazard_count: 0,
        }
    }

    pub fn make_body_piece(sid: SnakeId, next_index: CellIndex<T>) -> Self {
        Cell {
            flags: CellFlag::SnakeBodyPiece,
            id: sid,
            idx: next_index,
            hazard_count: 0,
        }
    }

    pub fn make_double_stacked_piece(sid: SnakeId, next_index: CellIndex<T>) -> Self {
        Cell {
            flags: CellFlag::DoubleStackedPiece,
            id: sid,
            idx: next_index,
            hazard_count: 0,
        }
    }

    pub fn make_triple_stacked_piece(sid: SnakeId) -> Self {
        Cell {
            flags: CellFlag::TripleStackedPiece,
            id: sid,
            idx: CellIndex(T::from_i32(0)),
            hazard_count: 0,
        }
    }

    pub fn is_snake_body_piece(&self) -> bool {
        self.flags == CellFlag::SnakeBodyPiece
    }

    pub fn is_double_stacked_piece(&self) -> bool {
        self.flags == CellFlag::DoubleStackedPiece
    }

    pub fn is_triple_stacked_piece(&self) -> bool {
        self.flags == CellFlag::TripleStackedPiece
    }

    pub fn is_body(&self) -> bool {
        self.is_snake_body_piece()
            || self.is_double_stacked_piece()
            || self.is_triple_stacked_piece()
    }

    pub fn set_food(&mut self) {
        self.flags = CellFlag::Food;
    }

    pub fn set_head(&mut self, sid: SnakeId, tail_index: CellIndex<T>) {
        self.flags = CellFlag::SnakeHead;
        self.id = sid;
        self.idx = tail_index;
    }

    pub fn set_body_piece(&mut self, sid: SnakeId, next_pos: CellIndex<T>) {
        self.flags = CellFlag::SnakeBodyPiece;
        self.id = sid;
        self.idx = next_pos;
    }

    pub fn set_double_stacked(&mut self, sid: SnakeId, next_pos: CellIndex<T>) {
        self.flags = CellFlag::DoubleStackedPiece;
        self.id = sid;
        self.idx = next_pos;
    }

    pub fn get_snake_id(&self) -> Option<SnakeId> {
        if self.is_body_segment() || self.is_head() {
            Some(self.id)
        } else {
            None
        }
    }

    pub fn get_idx(&self) -> CellIndex<T> {
        self.idx
    }
}
