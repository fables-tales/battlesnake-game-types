use crate::wire_representation::Game;
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

pub type SnakeIDMap = HashMap<String, SnakeId>;

#[derive(Debug, Clone, Copy)]
pub struct Vector {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Move {
    Left,
    Down,
    Up,
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
    pub fn to_vector(self) -> Vector {
        match self {
            Move::Left => Vector { x: -1, y: 0 },
            Move::Right => Vector { x: 1, y: 0 },
            Move::Up => Vector { x: 0, y: 1 },
            Move::Down => Vector { x: 0, y: -1 },
        }
    }

    pub fn all() -> Vec<Move> {
        vec![Move::Up, Move::Down, Move::Left, Move::Right]
    }

    pub fn as_index(&self) -> usize {
        match self {
            Move::Left => 0,
            Move::Right => 1,
            Move::Up => 2,
            Move::Down => 3,
        }
    }

    #[allow(dead_code)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SnakeId(pub u8);

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

pub trait SnakeIDGettableGame {
    type SnakeIDType;
    fn get_snake_ids(&self) -> Vec<Self::SnakeIDType>;
}

/// Instruments to be used with simulation
pub trait SimulatorInstruments: std::fmt::Debug {
    fn observe_simulation(&self, duration: Duration);
}

/// A game for which "you" is determinable
pub trait YouDeterminableGame: std::fmt::Debug {
    type SnakeIDType;

    /// determines for a given game if a given snake id is you.
    fn is_you(&self, snake_id: &Self::SnakeIDType) -> bool;

    /// get the id for you for a given game
    fn you_id(&self) -> &Self::SnakeIDType;
}

/// A game which can have it's winner determined
pub trait VictorDeterminableGame: std::fmt::Debug {
    type SnakeIDType;
    fn is_over(&self) -> bool;
    fn get_winner(&self) -> Option<Self::SnakeIDType>;
}

pub trait SimulableGame<T: SimulatorInstruments>: std::fmt::Debug + Sized {
    type SnakeIDType;
    /// simulates all possible future games for a given game returning the snake ids, moves that
    /// got to a given state, plus that state
    fn simulate(
        &self,
        instruments: &T,
        snake_ids: Vec<Self::SnakeIDType>,
    ) -> Vec<(Vec<(Self::SnakeIDType, Move)>, Self)> {
        let moves_to_simulate = Move::all();
        let build = snake_ids
            .into_iter()
            .map(|s| (s, moves_to_simulate.clone()))
            .collect::<Vec<_>>();
        self.simulate_with_moves(instruments, build)
    }
    /// simulates the next possible states for a a game with a given set of snakes and moves, producing a list of the new games,
    /// along with the moves that got to that position
    fn simulate_with_moves(
        &self,
        instruments: &T,
        snake_ids_and_moves: Vec<(Self::SnakeIDType, Vec<Move>)>,
    ) -> Vec<(Vec<(Self::SnakeIDType, Move)>, Self)>;
}

pub trait RandomReasonableMovesGame {
    type SnakeIDType;
    fn random_reasonable_move_for_each_snake(&self) -> Vec<(Self::SnakeIDType, Move)>;
}
