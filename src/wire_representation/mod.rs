#![allow(missing_docs)]
//! types to match the battlesnake wire representation
mod simulator;


use crate::compact_representation::{CellBoard, CellNum};
use crate::types::{
    Move, RandomReasonableMovesGame, SimulableGame, SimulatorInstruments, SnakeIDGettableGame,
    SnakeIDMap, Vector, VictorDeterminableGame, YouDeterminableGame,
};
use rand::prelude::IteratorRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

/// Struct that matches the `battlesnake` object from the wire representation
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BattleSnake {
    pub id: String,
    pub name: String,
    pub head: Position,
    pub body: VecDeque<Position>,
    pub health: i32,
    #[serde(skip)]
    pub actual_length: Option<i32>,
}

/// Struct that matches the `position` object from the wire representation
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn add_vec(&self, v: Vector) -> Position {
        Position {
            x: (self.x as i64 + v.x) as i32,
            y: (self.y as i64 + v.y) as i32,
        }
    }
    pub fn sub_vec(&self, v: Vector) -> Position {
        Position {
            x: (self.x as i64 - v.x) as i32,
            y: (self.y as i64 - v.y) as i32,
        }
    }

    pub fn to_vector(&self) -> Vector {
        Vector {
            x: self.x as i64,
            y: self.y as i64,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub height: u32,
    pub width: u32,
    pub food: Vec<Position>,
    pub snakes: Vec<BattleSnake>,
    pub hazards: Vec<Position>,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for i in 0..self.height {
            let k = self.height - i - 1;
            for j in 0..self.width {
                let position = Position {
                    x: j as i32,
                    y: k as i32,
                };
                if self.food.contains(&position) {
                    write!(f, "f")?;
                } else if self.snakes.iter().any(|s| s.head == position) {
                    write!(f, "H")?;
                } else if self.snakes.iter().any(|s| s.body.contains(&position)) {
                    write!(f, "s")?;
                } else if self.hazards.contains(&position) {
                    write!(f, "x")?;
                } else {
                    write!(f, ".")?;
                }
                write!(f, " ")?;
            }
            writeln!(f)?;
        }
        for snake in self.snakes.iter() {
            write!(
                f,
                "({} health: {} head: {:?}) ",
                snake.id, snake.health, snake.head
            )?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NestedGame {
    pub id: String,
}

/// Root object from the battlesnake server in start, move, and end requests, you
/// probably want to do:
/// ```
/// # use battlesnake_game_types::wire_representation::Game;
/// # let body = b"{\"game\":{\"id\":\"4e7c8fe2-a462-4015-95af-5eab3487d5ab\",\"ruleset\":{\"name\":\"royale\",\"version\":\"v1.0.17\"},\"timeout\":500},\"turn\":60,\"board\":{\"height\":11,\"width\":11,\"snakes\":[{\"id\":\"gs_PpJMhVwVvgb4wqHdpGdTVrqB\",\"name\":\"Untimely Neglected Wearable\",\"latency\":\"78\",\"health\":100,\"body\":[{\"x\":2,\"y\":0},{\"x\":2,\"y\":1},{\"x\":2,\"y\":2},{\"x\":2,\"y\":3},{\"x\":2,\"y\":4},{\"x\":2,\"y\":5},{\"x\":2,\"y\":5}],\"head\":{\"x\":2,\"y\":0},\"length\":7,\"shout\":\"\"},{\"id\":\"gs_gbBpgGW7cRFJ3PMpBmJ3RtSF\",\"name\":\"Pretzel\",\"latency\":\"101\",\"health\":78,\"body\":[{\"x\":3,\"y\":7},{\"x\":3,\"y\":8},{\"x\":4,\"y\":8},{\"x\":5,\"y\":8},{\"x\":6,\"y\":8},{\"x\":7,\"y\":8},{\"x\":7,\"y\":7}],\"head\":{\"x\":3,\"y\":7},\"length\":7,\"shout\":\"\"},{\"id\":\"gs_H3PCGx3GqkpSBfv9vfxTdMBF\",\"name\":\"Secret Snake\",\"latency\":\"22\",\"health\":65,\"body\":[{\"x\":1,\"y\":9},{\"x\":2,\"y\":9},{\"x\":3,\"y\":9},{\"x\":3,\"y\":10},{\"x\":2,\"y\":10}],\"head\":{\"x\":1,\"y\":9},\"length\":5,\"shout\":\"\"},{\"id\":\"gs_MMxyjByhGFbtGSV8KJv3tqdV\",\"name\":\"does this work lol\",\"latency\":\"100\",\"health\":86,\"body\":[{\"x\":10,\"y\":4},{\"x\":10,\"y\":5},{\"x\":9,\"y\":5},{\"x\":8,\"y\":5},{\"x\":7,\"y\":5},{\"x\":6,\"y\":5},{\"x\":5,\"y\":5},{\"x\":4,\"y\":5},{\"x\":4,\"y\":4},{\"x\":5,\"y\":4}],\"head\":{\"x\":10,\"y\":4},\"length\":10,\"shout\":\"\"}],\"food\":[{\"x\":10,\"y\":3}],\"hazards\":[{\"x\":0,\"y\":0},{\"x\":0,\"y\":1},{\"x\":0,\"y\":2},{\"x\":0,\"y\":3},{\"x\":0,\"y\":4},{\"x\":0,\"y\":5},{\"x\":0,\"y\":6},{\"x\":0,\"y\":7},{\"x\":0,\"y\":8},{\"x\":0,\"y\":9},{\"x\":0,\"y\":10},{\"x\":1,\"y\":0},{\"x\":1,\"y\":1},{\"x\":1,\"y\":2},{\"x\":1,\"y\":3},{\"x\":1,\"y\":4},{\"x\":1,\"y\":5},{\"x\":1,\"y\":6},{\"x\":1,\"y\":7},{\"x\":1,\"y\":8},{\"x\":1,\"y\":9},{\"x\":1,\"y\":10},{\"x\":2,\"y\":0},{\"x\":2,\"y\":1},{\"x\":2,\"y\":2},{\"x\":2,\"y\":3},{\"x\":2,\"y\":4},{\"x\":2,\"y\":5},{\"x\":2,\"y\":6},{\"x\":2,\"y\":7},{\"x\":2,\"y\":8},{\"x\":2,\"y\":9},{\"x\":2,\"y\":10}]},\"you\":{\"id\":\"gs_MMxyjByhGFbtGSV8KJv3tqdV\",\"name\":\"does this work lol\",\"latency\":\"100\",\"health\":86,\"body\":[{\"x\":10,\"y\":4},{\"x\":10,\"y\":5},{\"x\":9,\"y\":5},{\"x\":8,\"y\":5},{\"x\":7,\"y\":5},{\"x\":6,\"y\":5},{\"x\":5,\"y\":5},{\"x\":4,\"y\":5},{\"x\":4,\"y\":4},{\"x\":5,\"y\":4}],\"head\":{\"x\":10,\"y\":4},\"length\":10,\"shout\":\"\"}}";
/// let g: Result<Game, _> = serde_json::from_slice(body);
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub you: BattleSnake,
    pub board: Board,
    pub turn: i32,
    pub game: NestedGame,
}

impl Game {
    pub fn you_are_winner(&self) -> bool {
        if self.you.health == 0 {
            return false;
        } else if self.board.snakes.len() == 1 && self.board.snakes[0].id == self.you.id {
            return true;
        } else {
            return false;
        }
    }

    pub fn as_cell_board<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize>(
        &self,
        snake_ids: &SnakeIDMap,
    ) -> Result<CellBoard<T, BOARD_SIZE, MAX_SNAKES>, Box<dyn Error>> {
        CellBoard::convert_from_game(self.clone(), snake_ids)
    }
    pub fn off_board(&self, position: Position) -> bool {
        position.x < 0
            || position.x >= self.board.width as i32
            || position.y < 0
            || position.y >= self.board.height as i32
    }

    pub fn snake_ids(&self) -> Vec<String> {
        self.board
            .snakes
            .iter()
            .map(|s| s.id.clone())
            .collect::<Vec<_>>()
    }

    pub fn random_reasonable_move_for_each_snake(&self) -> Vec<(String, Move)> {
        self.board
            .snakes
            .iter()
            .map(|s| {
                let moves = Move::all().into_iter().filter(|mv| {
                    let new_head = s.head.add_vec(mv.to_vector());
                    let unreasonable = self.off_board(new_head)
                        || self.board.snakes.iter().any(|s| s.body.contains(&new_head));
                    !unreasonable
                });
                (
                    s.id.clone(),
                    moves.choose(&mut thread_rng()).unwrap_or(
                        Move::all()
                            .into_iter()
                            .filter(|mv| {
                                let new_head = s.head.add_vec(mv.to_vector());
                                new_head != s.body[1]
                            })
                            .choose(&mut thread_rng())
                            .unwrap(),
                    ),
                )
            })
            .collect()
    }
}

impl VictorDeterminableGame for Game {
    type SnakeIDType = String;

    fn is_over(&self) -> bool {
        self.you.health == 0 || self.board.snakes.len() == 1
    }

    fn get_winner(&self) -> Option<String> {
        if self.is_over() {
            Some(
                self.snake_ids()
                    .iter()
                    .filter(|s| s != &self.you_id())
                    .next()
                    .unwrap_or(self.you_id())
                    .clone(),
            )
        } else {
            None
        }
    }
}

impl YouDeterminableGame for Game {
    type SnakeIDType = String;

    /// determines for a given game if a given snake id is you.
    fn is_you(&self, snake_id: &Self::SnakeIDType) -> bool {
        snake_id == &self.you.id
    }

    /// get the id for you for a given game
    fn you_id(&self) -> &Self::SnakeIDType {
        &self.you.id
    }
}

impl SnakeIDGettableGame for Game {
    type SnakeIDType = String;
    fn get_snake_ids(&self) -> Vec<Self::SnakeIDType> {
        self.board.snakes.iter().map(|s| s.id.clone()).collect()
    }
}

impl<T: SimulatorInstruments> SimulableGame<T> for Game {
    type SnakeIDType = String;
    fn simulate_with_moves(
        &self,
        instruments: &T,
        snake_ids_and_moves: Vec<(Self::SnakeIDType, Vec<Move>)>,
    ) -> Vec<(Vec<(Self::SnakeIDType, Move)>, Game)> {
        simulator::Simulator::new(self).simulate_with_moves(instruments, snake_ids_and_moves)
    }
}

impl RandomReasonableMovesGame for Game {
    type SnakeIDType = String;

    fn random_reasonable_move_for_each_snake(&self) -> Vec<(String, Move)> {
        self.board
            .snakes
            .iter()
            .map(|s| {
                let moves = Move::all().into_iter().filter(|mv| {
                    let new_head = s.head.add_vec(mv.to_vector());
                    let unreasonable = self.off_board(new_head)
                        || self.board.snakes.iter().any(|s| s.body.contains(&new_head));
                    !unreasonable
                });
                (
                    s.id.clone(),
                    moves.choose(&mut thread_rng()).unwrap_or(Move::Up),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Game {
        let game_fixture = include_str!("../../fixtures/4_snake_game.json");
        let g: Result<Game, _> = serde_json::from_slice(game_fixture.as_bytes());
        let g = g.expect("the json literal is valid");
        g
    }

    #[test]
    fn test_game_you_determinable() {
        let g = fixture();
        assert_eq!("gs_MMxyjByhGFbtGSV8KJv3tqdV", g.you_id());
        assert!(g.is_you(&"gs_MMxyjByhGFbtGSV8KJv3tqdV".to_string()));
    }

    #[test]
    fn test_snake_id_gettable() {
        let g = fixture();
        assert_eq!(
            vec![
                "gs_PpJMhVwVvgb4wqHdpGdTVrqB",
                "gs_gbBpgGW7cRFJ3PMpBmJ3RtSF",
                "gs_H3PCGx3GqkpSBfv9vfxTdMBF",
                "gs_MMxyjByhGFbtGSV8KJv3tqdV"
            ],
            g.snake_ids()
        );
    }
}
