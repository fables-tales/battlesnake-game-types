//! various types that are useful for working with battlesnake
use crate::wire_representation::{Game, Position};
use rand::Rng;
use serde::{Deserialize, Serialize, Serializer};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::hash::Hash;
use std::time::Duration;

/// Represents the snake IDs for a given game. This should be established once on the `/start` request and then
/// stored, so that `SnakeIds` are stable throughout the game.
pub type SnakeIDMap = HashMap<String, SnakeId>;

/// A vector with which to do positional math
#[derive(Debug, Clone, Copy)]
pub struct Vector {
    /// x position
    pub x: i64,
    /// y position
    pub y: i64,
}

/// there are 4 moves
pub const N_MOVES: usize = 4;

/// Represents a move
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Move {
    #[allow(missing_docs)]
    Left,
    #[allow(missing_docs)]
    Down,
    #[allow(missing_docs)]
    Up,
    #[allow(missing_docs)]
    Right,
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Move::Left => write!(f, "left"),
            Move::Right => write!(f, "right"),
            Move::Up => write!(f, "up"),
            Move::Down => write!(f, "down"),
        }
    }
}

impl Move {
    /// convert this move to a vector
    pub fn to_vector(self) -> Vector {
        match self {
            Move::Left => Vector { x: -1, y: 0 },
            Move::Right => Vector { x: 1, y: 0 },
            Move::Up => Vector { x: 0, y: 1 },
            Move::Down => Vector { x: 0, y: -1 },
        }
    }

    /// create a Move from the given vector
    pub fn from_vector(vector: Vector) -> Self {
        match vector {
            Vector { x: -1, y: 0 } => Self::Left,
            Vector { x: 1, y: 0 } => Self::Right,
            Vector { x: 0, y: 1 } => Self::Up,
            Vector { x: 0, y: -1 } => Self::Down,
            _ => panic!(),
        }
    }

    /// returns a vec of all possible moves
    pub const fn all() -> [Self; N_MOVES] {
        [Move::Up, Move::Down, Move::Left, Move::Right]
    }

    /// returns an Iterator of all possible moves
    pub fn all_iter() -> MoveIter {
        MoveIter(0)
    }

    /// converts this move to a usize index. indices are the same order as the `Move::all()` method
    pub fn as_index(&self) -> usize {
        match self {
            Move::Up => 0,
            Move::Down => 1,
            Move::Left => 2,
            Move::Right => 3,
        }
    }

    /// converts a usize index to a move
    pub fn from_index(index: usize) -> Move {
        match index {
            0 => Move::Up,
            1 => Move::Down,
            2 => Move::Left,
            3 => Move::Right,
            _ => panic!("invalid index"),
        }
    }

    #[allow(dead_code)]
    /// checks if a given move is not opposibe this move. e.g. Up is not opposite to Left, but is opposite to Down
    pub fn is_not_opposite(&self, other: &Move) -> bool {
        !matches!(
            (self, other),
            (Move::Up, Move::Down)
                | (Move::Down, Move::Up)
                | (Move::Left, Move::Right)
                | (Move::Right, Move::Left)
        )
    }
}

#[derive(Copy, Clone, Debug)]
/// Iterator over all moves. Returned by `Move::all_iter()`
///
/// The iterator yields elements in the same order as `Move::all()`
pub struct MoveIter(usize);

impl Iterator for MoveIter {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 < N_MOVES {
            let m = Move::from_index(self.0);
            self.0 += 1;
            Some(m)
        } else {
            None
        }
    }
}

/// token to represent a snake id
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize)]
#[repr(transparent)]
pub struct SnakeId(pub u8);

impl SnakeId {
    /// convert this snake ID to a usize
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl Serialize for SnakeId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.0)
    }
}

/// builds a snake ID map for a given game, mapping snakes to
/// integers. The snake in "you" is always ID 0. Instead of
/// calling this on every game you are given, you should call
/// this function once per game at the start, and store the result
/// that way you can stabally have integer IDs for a given snake
/// throughout a game
pub fn build_snake_id_map(g: &Game) -> SnakeIDMap {
    let mut hm = HashMap::new();
    hm.insert(g.you.id.clone(), SnakeId(0));
    let mut i = 1;
    for snake in g.board.snakes.iter() {
        if snake.id != g.you.id {
            hm.insert(snake.id.clone(), SnakeId(i));
            i += 1;
        }
    }

    hm
}

/// A game for which one can get the snake ids
pub trait SnakeIDGettableGame {
    #[allow(missing_docs)]
    type SnakeIDType: PartialEq + Debug + Serialize + Eq + Hash + Clone + Send;

    #[allow(missing_docs)]
    fn get_snake_ids(&self) -> Vec<Self::SnakeIDType>;
}

/// Instruments to be used with simulation
pub trait SimulatorInstruments: std::fmt::Debug {
    #[allow(missing_docs)]
    fn observe_simulation(&self, duration: Duration);
}

/// A game for which "you" is determinable
pub trait YouDeterminableGame: std::fmt::Debug + SnakeIDGettableGame {
    /// determines for a given game if a given snake id is you.
    fn is_you(&self, snake_id: &Self::SnakeIDType) -> bool;

    /// get the id for you for a given game
    fn you_id(&self) -> &Self::SnakeIDType;
}

/// A game which can have it's winner determined
pub trait VictorDeterminableGame: std::fmt::Debug + SnakeIDGettableGame {
    #[allow(missing_docs)]
    fn is_over(&self) -> bool;

    /// get the winner for a given game, will return None in the case of a draw, or if the game is not over
    fn get_winner(&self) -> Option<Self::SnakeIDType>;

    /// How many snakes are alive
    fn alive_snake_count(&self) -> usize;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
/// Represents moves taken for a given simulation
pub struct Action<const N_SNAKES: usize> {
    moves: [Option<Move>; N_SNAKES],
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Represents only the moves taken by other snakes in a game
pub struct OtherAction<const N_SNAKES: usize> {
    moves: [Option<Move>; N_SNAKES],
}

impl<const N_SNAKES: usize> Action<N_SNAKES> {
    /// create a new action from a given array of moves
    pub fn new(moves: [Option<Move>; N_SNAKES]) -> Self {
        Self { moves }
    }

    /// collects an action from an iterator of moves
    pub fn collect_from<'a, T: Iterator<Item = &'a (SnakeId, Move)>>(ids_and_moves: T) -> Self {
        let mut moves = [None; N_SNAKES];
        for (id, mv) in ids_and_moves {
            moves[id.as_usize()] = Some(*mv);
        }
        Self { moves }
    }

    /// gets your move
    pub fn own_move(&self) -> Move {
        self.moves[0].unwrap()
    }
    /// construct an OtherAction of the other sankes moves
    pub fn other_moves(&self) -> OtherAction<N_SNAKES> {
        let mut new_moves = self.moves;
        new_moves[0] = None;
        OtherAction { moves: new_moves }
    }

    /// Get the inner array back
    pub fn into_inner(self) -> [Option<Move>; N_SNAKES] {
        self.moves
    }
}

/// a game for which future states can be simulated
pub trait SimulableGame<T: SimulatorInstruments, const N_SNAKES: usize>:
    std::fmt::Debug + Sized + SnakeIDGettableGame
{
    /// simulates all possible future games for a given game returning the snake ids, moves that
    /// got to a given state, plus that state
    #[allow(clippy::type_complexity)]
    fn simulate(
        &self,
        instruments: &T,
        snake_ids: Vec<Self::SnakeIDType>,
    ) -> Box<dyn Iterator<Item = (Action<N_SNAKES>, Self)> + '_> {
        let moves_to_simulate = Move::all();
        let build = snake_ids
            .into_iter()
            .map(|s| (s, moves_to_simulate.as_slice()));
        self.simulate_with_moves(instruments, build)
    }
    /// simulates the next possible states for a a game with a given set of snakes and moves, producing a list of the new games,
    /// along with the moves that got to that position
    #[allow(clippy::type_complexity)]
    fn simulate_with_moves<S>(
        &self,
        instruments: &T,
        snake_ids_and_moves: impl IntoIterator<Item = (Self::SnakeIDType, S)>,
    ) -> Box<dyn Iterator<Item = (Action<N_SNAKES>, Self)> + '_>
    where
        S: Borrow<[Move]>;
}

/// A game where positions can be checked for hazards
pub trait HazardQueryableGame: PositionGettableGame {
    /// Is this position a hazard?
    fn is_hazard(&self, pos: &Self::NativePositionType) -> bool;

    /// how much damage do hazards do?
    fn get_hazard_damage(&self) -> u8;
}

/// A game where positions can be checked for food
pub trait FoodQueryableGame: PositionGettableGame {
    /// Is this position a food?
    fn is_food(&self, pos: &Self::NativePositionType) -> bool;
}

/// A game where positions can be checked to see if they are a certain snakes Neck piece
///
/// A neck is defined as the piece that comes immediately after the head of a snake. If the snake
/// is fully triple stacked it has no neck piece.
pub trait NeckQueryableGame: PositionGettableGame + SnakeIDGettableGame {
    /// Is this position a neck for the given snake?
    fn is_neck(&self, sid: &Self::SnakeIDType, pos: &Self::NativePositionType) -> bool;
}

/// A game where positions can have their hazards set and cleared
pub trait HazardSettableGame: PositionGettableGame {
    /// make this position a hazard
    fn set_hazard(&mut self, pos: Self::NativePositionType);

    /// clear this position of being a hazard
    fn clear_hazard(&mut self, pos: Self::NativePositionType);
}

/// A game for which board positions can be identified and returned
pub trait PositionGettableGame {
    /// the native position type for this board
    type NativePositionType: Eq + Hash + Clone + Ord + PartialOrd + Debug;

    /// Check if the given position is a snake body
    fn position_is_snake_body(&self, pos: Self::NativePositionType) -> bool;

    /// Convert a position to the native type
    fn position_from_native(&self, native: Self::NativePositionType) -> Position;

    /// Convert a position to the native type
    fn native_from_position(&self, pos: Position) -> Self::NativePositionType;

    /// checks if a given position is not on this board
    fn off_board(&self, pos: Position) -> bool;
}

/// A game for which the head of the current snake can be got.
pub trait HeadGettableGame: PositionGettableGame + SnakeIDGettableGame {
    /// get the head position for a given snake id, as a position struct (slow for simulation)
    fn get_head_as_position(
        &self,
        snake_id: &Self::SnakeIDType,
    ) -> crate::wire_representation::Position;

    /// get the head position for a given snake as some "native" type for this game
    fn get_head_as_native_position(&self, snake_id: &Self::SnakeIDType)
        -> Self::NativePositionType;
}

/// A game for which the food on the board can be queries
pub trait FoodGettableGame: PositionGettableGame + SnakeIDGettableGame {
    /// get the head position for a given snake id, as a position struct (slow for simulation)
    fn get_all_food_as_positions(&self) -> Vec<crate::wire_representation::Position>;

    /// get the head position for a given snake as some "native" type for this game
    fn get_all_food_as_native_positions(&self) -> Vec<Self::NativePositionType>;
}

/// A game for which the length of the current snake can be got.
pub trait LengthGettableGame: SnakeIDGettableGame {
    /// the length type for this game
    type LengthType: Ord + PartialOrd;

    /// get the length for a given snake
    fn get_length(&self, snake_id: &Self::SnakeIDType) -> Self::LengthType;

    /// get the length for a given snake
    fn get_length_i64(&self, snake_id: &Self::SnakeIDType) -> i64;
}

/// A game for which the health of the current snake can be got.
pub trait HealthGettableGame: SnakeIDGettableGame {
    /// the health type for this game
    type HealthType: PartialEq;

    /// A constant that defines what zero health means for the given game
    const ZERO: Self::HealthType;

    /// get the health for a given snake
    fn get_health(&self, snake_id: &Self::SnakeIDType) -> Self::HealthType;

    /// get the health for a given snake as an i64
    fn get_health_i64(&self, snake_id: &Self::SnakeIDType) -> i64;

    /// check wheterh a given snake is alive
    fn is_alive(&self, snake_id: &Self::SnakeIDType) -> bool {
        self.get_health(snake_id) != Self::ZERO
    }
}

/// a game for which random reasonable moves for a given snake can be determined. e.g. do not collide with yourself
pub trait RandomReasonableMovesGame: SnakeIDGettableGame {
    #[allow(missing_docs)]
    fn random_reasonable_move_for_each_snake<'a>(
        &'a self,
        rng: &'a mut impl Rng,
    ) -> Box<dyn Iterator<Item = (Self::SnakeIDType, Move)> + 'a>;
}

/// a game for which reasonable moves for a given snake can be determined. e.g. do not collide with yourself
pub trait ReasonableMovesGame: SnakeIDGettableGame {
    #[allow(missing_docs)]
    fn reasonable_moves_for_each_snake(
        &self,
    ) -> Box<dyn Iterator<Item = (Self::SnakeIDType, Vec<Move>)> + '_>;
}

/// a game for which the neighbors of a given Position can be determined
pub trait NeighborDeterminableGame: PositionGettableGame {
    /// returns the neighboring positions
    fn neighbors<'a>(
        &'a self,
        pos: &Self::NativePositionType,
    ) -> Box<dyn Iterator<Item = Self::NativePositionType> + 'a>;

    /// returns the neighboring positions, and the Move required to get to each
    fn possible_moves<'a>(
        &'a self,
        pos: &Self::NativePositionType,
    ) -> Box<dyn Iterator<Item = (Move, Self::NativePositionType)> + 'a>;
}

/// a game for which each snakes shout can be determined
pub trait ShoutGettableGame: SnakeIDGettableGame {
    /// get the shout for a given snake, if they shouted this turn
    fn get_shout(&self, snake_id: &Self::SnakeIDType) -> Option<String>;
}

/// a game for which the size of the game board can be determined
pub trait SizeDeterminableGame {
    #[allow(missing_docs)]
    fn get_width(&self) -> u32;
    #[allow(missing_docs)]
    fn get_height(&self) -> u32;
}

/// a game for which the current turn is determinable
pub trait TurnDeterminableGame {
    #[allow(missing_docs)]
    fn turn(&self) -> u64;
}

/// A game where an entire snake body is gettable
pub trait SnakeBodyGettableGame: PositionGettableGame + SnakeIDGettableGame {
    /// return a Vec of the positions for a given snake body, in order from head to tail
    fn get_snake_body_vec(&self, snake_id: &Self::SnakeIDType) -> Vec<Self::NativePositionType>;

    /// return an iterator over all the snake body positions. Order is NOT guaranteed to be from head to tail
    /// implementations are free to do any order that is efficient for them.
    /// Positions that would be duplicate, due to a snake being double or triple stacked, may be
    /// omitted
    fn get_snake_body_iter(
        &self,
        snake_id: &Self::SnakeIDType,
    ) -> Box<dyn Iterator<Item = Self::NativePositionType> + '_>;
}

/// A marker trait that can be used to specify the number of snakes this board can support
pub trait MaxSnakes<const MAX_SNAKES: usize> {}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_move_all_order_matches_iter() {
        assert_eq!(Move::all().to_vec(), Move::all_iter().collect::<Vec<_>>());
    }
}
