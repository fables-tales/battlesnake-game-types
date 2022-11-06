#![allow(missing_docs)]
//! types to match the battlesnake wire representation

use crate::compact_representation;
use crate::compact_representation::dimensions::Dimensions;
use crate::compact_representation::CellNum;
use crate::compact_representation::StandardCellBoard;
use crate::types::*;
use rand::prelude::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::convert::TryInto;
use std::error::Error;
use std::fmt::{self, Display};

/// Struct that matches the `battlesnake` object from the wire representation
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BattleSnake {
    pub id: String,
    pub name: String,
    pub head: Position,
    pub body: VecDeque<Position>,
    pub health: i32,
    pub shout: Option<String>,
    #[serde(skip)]
    pub actual_length: Option<i32>,
}

/// Struct that matches the `position` object from the wire representation
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn manhattan_length(&self) -> u32 {
        (self.x.abs() + self.y.abs()) as u32
    }

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
    pub ruleset: Ruleset,
    pub timeout: i64,
    #[serde(default, deserialize_with = "non_empty_str")]
    pub map: Option<String>,
    #[serde(default, deserialize_with = "non_empty_str")]
    pub source: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Ruleset {
    pub name: String,
    pub version: String,
    pub settings: Option<Settings>,
}

fn non_empty_str<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    let o: Option<String> = Option::deserialize(d)?;
    Ok(o.filter(|s| !s.is_empty()))
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    #[serde(rename = "foodSpawnChance")]
    pub food_spawn_chance: i32,
    #[serde(rename = "minimumFood")]
    pub minimum_food: i32,
    #[serde(rename = "hazardDamagePerTurn")]
    pub hazard_damage_per_turn: i32,
    #[serde(default, rename = "hazardMap", deserialize_with = "non_empty_str")]
    pub hazard_map: Option<String>,
    #[serde(
        default,
        rename = "hazardMapAuthor",
        deserialize_with = "non_empty_str"
    )]
    pub hazard_map_author: Option<String>,
    pub royale: Option<RoyaleSettings>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoyaleSettings {
    #[serde(rename = "shrinkEveryNTurns")]
    pub shrink_every_n_turns: i32,
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
    /// optional, so as to not break backwards compatibility
    pub board: Board,
    pub turn: i32,
    pub game: NestedGame,
}

impl Game {
    pub fn you_are_winner(&self) -> bool {
        if self.you.health == 0 {
            false
        } else {
            self.board.snakes.len() == 1 && self.board.snakes[0].id == self.you.id
        }
    }

    pub fn as_cell_board<
        T: CellNum,
        D: Dimensions,
        const BOARD_SIZE: usize,
        const MAX_SNAKES: usize,
    >(
        &self,
        snake_ids: &SnakeIDMap,
    ) -> Result<StandardCellBoard<T, D, BOARD_SIZE, MAX_SNAKES>, Box<dyn Error>> {
        StandardCellBoard::convert_from_game(self.clone(), snake_ids)
    }

    pub fn as_wrapped_cell_board<
        T: compact_representation::CellNum,
        D: Dimensions,
        const BOARD_SIZE: usize,
        const MAX_SNAKES: usize,
    >(
        &self,
        snake_ids: &SnakeIDMap,
    ) -> Result<
        compact_representation::wrapped::CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>,
        Box<dyn Error>,
    > {
        if self.is_wrapped() {
            compact_representation::wrapped::CellBoard::convert_from_game(self.clone(), snake_ids)
        } else {
            panic!("Cannot convert a non-wrapped game to a wrapped game")
        }
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

    /// Returns a boolean indicating whether this game is using the wrapped ruleset
    pub fn is_wrapped(&self) -> bool {
        self.game.ruleset.name == "wrapped"
    }

    pub fn is_arcade_maze_map(&self) -> bool {
        self.game.map == Some("arcade_maze".to_owned())
    }
}

impl RandomReasonableMovesGame for Game {
    fn random_reasonable_move_for_each_snake<'a>(
        &'a self,
        rng: &'a mut impl rand::Rng,
    ) -> Box<dyn Iterator<Item = (Self::SnakeIDType, Move)> + 'a> {
        Box::new(self.board.snakes.iter().map(move |s| {
            let all_moves = Move::all();
            let moves = all_moves.iter().filter(|mv| {
                let mut new_head = s.head.add_vec(mv.to_vector());

                if self.is_wrapped() {
                    let wrapped_x = new_head.x.rem_euclid(self.get_width() as i32);
                    let wrapped_y = new_head.y.rem_euclid(self.get_height() as i32);

                    new_head = Position {
                        x: wrapped_x,
                        y: wrapped_y,
                    };
                }

                let hazard_damage: i32 = self.get_hazard_damage().into();

                let unreasonable = self.off_board(new_head)
                    || self.board.snakes.iter().any(|s| s.body.contains(&new_head))
                    || (self.board.hazards.contains(&new_head) && hazard_damage >= s.health);

                !unreasonable
            });
            (
                s.id.clone(),
                moves.choose(rng).copied().unwrap_or_else(|| {
                    Move::all()
                        .iter()
                        .filter(|mv| {
                            let new_head = s.head.add_vec(mv.to_vector());
                            new_head != s.body[1]
                        })
                        .choose(rng)
                        .copied()
                        .unwrap()
                }),
            )
        }))
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for i in 0..self.board.height {
            let k = self.board.height - i - 1;
            for j in 0..self.board.width {
                let position = Position {
                    x: j as i32,
                    y: k as i32,
                };
                if self.board.food.contains(&position) {
                    write!(f, "f")?;
                } else if self.board.snakes.iter().any(|s| s.head == position) {
                    if position == self.you.head {
                        write!(f, "S")?;
                    } else {
                        write!(f, "H")?;
                    }
                } else if self.board.snakes.iter().any(|s| s.body.contains(&position)) {
                    write!(f, "s")?;
                } else if self.board.hazards.contains(&position) {
                    write!(f, "x")?;
                } else {
                    write!(f, ".")?;
                }
                write!(f, " ")?;
            }
            writeln!(f)?;
        }
        for snake in self.board.snakes.iter() {
            write!(
                f,
                "({} health: {} head: {:?}) ",
                snake.id, snake.health, snake.head
            )?;
        }
        Ok(())
    }
}

impl VictorDeterminableGame for Game {
    fn is_over(&self) -> bool {
        self.you.health == 0 || self.board.snakes.len() == 1
    }

    fn get_winner(&self) -> Option<String> {
        if self.is_over() {
            Some(
                self.snake_ids()
                    .iter()
                    .find(|s| s != &self.you_id())
                    .unwrap_or_else(|| self.you_id())
                    .clone(),
            )
        } else {
            None
        }
    }

    fn alive_snake_count(&self) -> usize {
        self.board.snakes.iter().filter(|s| s.health > 0).count()
    }
}

impl YouDeterminableGame for Game {
    /// determines for a given game if a given snake id is you.
    fn is_you(&self, snake_id: &Self::SnakeIDType) -> bool {
        snake_id == &self.you.id
    }

    /// get the id for you for a given game
    fn you_id(&self) -> &Self::SnakeIDType {
        &self.you.id
    }
}

impl LengthGettableGame for Game {
    type LengthType = usize;

    fn get_length(&self, snake_id: &Self::SnakeIDType) -> Self::LengthType {
        self.board
            .snakes
            .iter()
            .find(|s| &s.id == snake_id)
            .unwrap()
            .body
            .len()
    }

    fn get_length_i64(&self, snake_id: &Self::SnakeIDType) -> i64 {
        self.get_length(snake_id) as i64
    }
}

impl PositionGettableGame for Game {
    type NativePositionType = Position;

    fn position_is_snake_body(&self, pos: Self::NativePositionType) -> bool {
        self.board.snakes.iter().any(|s| s.body.contains(&pos))
    }

    fn position_from_native(&self, native: Self::NativePositionType) -> Position {
        native
    }

    fn native_from_position(&self, pos: Position) -> Self::NativePositionType {
        pos
    }

    fn off_board(&self, pos: Position) -> bool {
        self.off_board(pos)
    }
}

impl FoodGettableGame for Game {
    fn get_all_food_as_positions(&self) -> Vec<crate::wire_representation::Position> {
        self.board.food.clone()
    }

    fn get_all_food_as_native_positions(&self) -> Vec<Self::NativePositionType> {
        self.get_all_food_as_positions()
    }
}

impl HealthGettableGame for Game {
    type HealthType = i32;
    const ZERO: Self::HealthType = 0;

    fn get_health(&self, snake_id: &Self::SnakeIDType) -> Self::HealthType {
        self.board
            .snakes
            .iter()
            .find(|s| &s.id == snake_id)
            .map(|s| s.health)
            .unwrap_or(Self::ZERO)
    }

    fn get_health_i64(&self, snake_id: &Self::SnakeIDType) -> i64 {
        self.get_health(snake_id) as i64
    }
}

impl SnakeIDGettableGame for Game {
    type SnakeIDType = String;
    fn get_snake_ids(&self) -> Vec<Self::SnakeIDType> {
        self.board.snakes.iter().map(|s| s.id.clone()).collect()
    }
}

impl HeadGettableGame for Game {
    fn get_head_as_position(
        &self,
        snake_id: &Self::SnakeIDType,
    ) -> crate::wire_representation::Position {
        self.get_head_as_native_position(snake_id)
    }

    fn get_head_as_native_position(
        &self,
        snake_id: &Self::SnakeIDType,
    ) -> Self::NativePositionType {
        self.board
            .snakes
            .iter()
            .find(|s| &s.id == snake_id)
            .unwrap()
            .head
    }
}

impl ShoutGettableGame for Game {
    fn get_shout(&self, snake_id: &Self::SnakeIDType) -> Option<String> {
        self.board
            .snakes
            .iter()
            .find(|s| &s.id == snake_id)
            .unwrap()
            .shout
            .clone()
    }
}

impl SizeDeterminableGame for Game {
    fn get_width(&self) -> u32 {
        self.board.width
    }

    fn get_height(&self) -> u32 {
        self.board.height
    }
}

impl TurnDeterminableGame for Game {
    fn turn(&self) -> u64 {
        self.turn.try_into().unwrap()
    }
}

impl SnakeBodyGettableGame for Game {
    fn get_snake_body_vec(&self, snake_id: &Self::SnakeIDType) -> Vec<Self::NativePositionType> {
        self.get_snake_body_iter(snake_id).collect()
    }

    fn get_snake_body_iter(
        &self,
        snake_id: &Self::SnakeIDType,
    ) -> Box<dyn Iterator<Item = Self::NativePositionType> + '_> {
        Box::new(
            self.board
                .snakes
                .iter()
                .find(|s| &s.id == snake_id)
                .unwrap()
                .body
                .iter()
                .cloned(),
        )
    }
}

impl HazardQueryableGame for Game {
    fn is_hazard(&self, pos: &Self::NativePositionType) -> bool {
        self.board.hazards.contains(pos)
    }

    fn get_hazard_damage(&self) -> u8 {
        self.game
            .ruleset
            .settings
            .as_ref()
            .map(|settings| settings.hazard_damage_per_turn)
            .unwrap_or(15) as u8
    }
}

impl HazardSettableGame for Game {
    fn set_hazard(&mut self, pos: Self::NativePositionType) {
        self.board.hazards.push(pos);
    }

    fn clear_hazard(&mut self, pos: Self::NativePositionType) {
        self.board.hazards.retain(|p| p != &pos);
    }
}

impl NeighborDeterminableGame for Game {
    fn neighbors<'a>(
        &'a self,
        pos: &Self::NativePositionType,
    ) -> Box<dyn Iterator<Item = Self::NativePositionType> + 'a> {
        Box::new(self.possible_moves(pos).map(|(_m, pos)| pos))
    }

    fn possible_moves<'a>(
        &'a self,
        pos: &Self::NativePositionType,
    ) -> Box<dyn Iterator<Item = (Move, Self::NativePositionType)> + 'a> {
        let clone = *pos;
        Box::new(Move::all_iter().filter_map(move |m| {
            let v = m.to_vector();

            let mut new_pos = clone.add_vec(v);

            if self.is_wrapped() {
                let wrapped_x = new_pos.x.rem_euclid(self.get_width() as i32);
                let wrapped_y = new_pos.y.rem_euclid(self.get_height() as i32);

                new_pos = Position {
                    x: wrapped_x,
                    y: wrapped_y,
                };
            }

            if self.off_board(new_pos) {
                debug_assert!(
                    !self.is_wrapped(),
                    "Wrapped board should not have off-board positions"
                );

                return None;
            }

            Some((m, new_pos))
        }))
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    fn fixture() -> Game {
        let game_fixture = include_str!("../../fixtures/4_snake_game.json");
        let g: Result<Game, _> = serde_json::from_slice(game_fixture.as_bytes());
        g.expect("the json literal is valid")
    }

    #[test]
    fn test_hazard_deserialization() {
        let empty_string_hazard = include_str!("../../fixtures/empty_str_hazard.json");
        let empty_string_hazard: Game =
            serde_json::from_str(empty_string_hazard).expect("the json literal is valid");

        assert_eq!(
            None,
            empty_string_hazard
                .game
                .ruleset
                .settings
                .unwrap()
                .hazard_map
        );

        let with_hazard_settings = include_str!("../../fixtures/hazard_map_settings.json");
        let with_hazard_settings: Game =
            serde_json::from_str(with_hazard_settings).expect("the json literal is valid");

        assert_eq!(
            Some("hz_spiral".to_string()),
            with_hazard_settings
                .game
                .ruleset
                .settings
                .as_ref()
                .unwrap()
                .hazard_map
        );
        assert_eq!(
            Some("altersaddle".to_string()),
            with_hazard_settings
                .game
                .ruleset
                .settings
                .unwrap()
                .hazard_map_author
        );
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

    #[test]
    fn test_center_of_board_neighbors() {
        let g = fixture();

        let pos = Position { x: 5, y: 5 };

        let neighbors = g.neighbors(&pos).collect_vec();

        let expected = vec![
            Position { x: 5, y: 6 },
            Position { x: 5, y: 4 },
            Position { x: 4, y: 5 },
            Position { x: 6, y: 5 },
        ];
        assert_eq!(neighbors, expected);
    }

    #[test]
    fn test_center_of_board_possible_moves() {
        let g = fixture();

        let pos = Position { x: 5, y: 5 };

        let possible_moves = g.possible_moves(&pos).collect_vec();

        let expected = vec![
            (Move::Up, Position { x: 5, y: 6 }),
            (Move::Down, Position { x: 5, y: 4 }),
            (Move::Left, Position { x: 4, y: 5 }),
            (Move::Right, Position { x: 6, y: 5 }),
        ];
        assert_eq!(possible_moves, expected);
    }

    #[test]
    fn test_edge_of_non_wrapped_board_neighbors() {
        let g = fixture();

        let pos = Position { x: 0, y: 0 };

        let neighbors = g.neighbors(&pos).collect_vec();

        let expected = vec![Position { x: 0, y: 1 }, Position { x: 1, y: 0 }];
        assert_eq!(neighbors, expected);
    }

    #[test]
    fn test_edge_of_board_non_wrapped_possible_moves() {
        let g = fixture();

        let pos = Position { x: 0, y: 0 };

        let possible_moves = g.possible_moves(&pos).collect_vec();

        let expected = vec![
            (Move::Up, Position { x: 0, y: 1 }),
            (Move::Right, Position { x: 1, y: 0 }),
        ];
        assert_eq!(possible_moves, expected);
    }

    #[test]
    fn test_edge_of_wrapped_board_neighbors() {
        let game_fixture = include_str!("../../fixtures/wrapped_fixture.json");
        let g: Result<Game, _> = serde_json::from_slice(game_fixture.as_bytes());
        let g = g.expect("the json literal is valid");

        let pos = Position { x: 0, y: 0 };

        let neighbors = g.neighbors(&pos).collect_vec();

        let expected = vec![
            Position { x: 0, y: 1 },
            Position { x: 0, y: 10 },
            Position { x: 10, y: 0 },
            Position { x: 1, y: 0 },
        ];
        assert_eq!(neighbors, expected);
    }

    #[test]
    fn test_edge_of_board_wrapped_possible_moves() {
        let game_fixture = include_str!("../../fixtures/wrapped_fixture.json");
        let g: Result<Game, _> = serde_json::from_slice(game_fixture.as_bytes());
        let g = g.expect("the json literal is valid");

        let pos = Position { x: 0, y: 0 };

        let possible_moves = g.possible_moves(&pos).collect_vec();

        let expected = vec![
            (Move::Up, Position { x: 0, y: 1 }),
            (Move::Down, Position { x: 0, y: 10 }),
            (Move::Left, Position { x: 10, y: 0 }),
            (Move::Right, Position { x: 1, y: 0 }),
        ];
        assert_eq!(possible_moves, expected);
    }

    #[test]
    fn test_map_json() {
        let game_fixture = include_str!("../../fixtures/arcade_maze_map.json");
        let g: Result<Game, _> = serde_json::from_slice(game_fixture.as_bytes());
        let g = g.expect("the json literal is valid");

        assert!(g.is_arcade_maze_map());
    }
}
