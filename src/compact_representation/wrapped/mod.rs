//! A compact board representation that is efficient for simulation
use crate::compact_representation::core::DOUBLE_STACK;
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
use rand::Rng;
use rand::prelude::IteratorRandom;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;

use crate::{
    types::{Move, SimulableGame, SimulatorInstruments, Action},
    wire_representation::Position,
};

use super::core::{Cell, simulate_with_moves};
use super::core::{CellIndex, TRIPLE_STACK, CellBoard as CCB};
use super::CellNum as CN;
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

/// A compact board representation that is significantly faster for simulation than
/// `battlesnake_game_types::wire_representation::Game`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CellBoard<T: CN, const BOARD_SIZE: usize, const MAX_SNAKES: usize> {
    embedded: CCB<T, BOARD_SIZE, MAX_SNAKES>,
}

impl<T: CN, const BOARD_SIZE: usize, const MAX_SNAKES: usize> CellBoard<T, BOARD_SIZE, MAX_SNAKES> {
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
        let embedded = CCB::new(
           game
               .game
               .ruleset
               .settings
               .as_ref()
               .map(|s| s.hazard_damage_per_turn)
               .unwrap_or(15) as u8,
           cells,
           healths,
           heads,
           lengths,
         game.board.width as u8,
        );

        Ok(CellBoard {
            embedded,
        })
    }

    pub fn pack_as_hash(&self) -> HashMap<String, Vec<u32>> {
        self.embedded.pack_as_hash()
    }

    pub fn from_packed_hash(hash: &HashMap<String, Vec<u32>>) -> Self {
        Self {
            embedded: CCB::from_packed_hash(hash),
        }
    }
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
impl<T: CN, const BOARD_SIZE: usize, const MAX_SNAKES: usize> RandomReasonableMovesGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn random_reasonable_move_for_each_snake<'a>(
        &'a self, rng: &'a mut impl Rng,
    ) -> Box<dyn std::iter::Iterator<Item = (SnakeId, Move)> + 'a> {
        let width = self.embedded.get_actual_width();
        Box::new(
            self.embedded.iter_healths()
                .enumerate()
                .filter(|(_, health)| **health > 0)
                .map(move |(idx, _)| {
                    let head = self.get_head_as_native_position(&SnakeId(idx as u8));
                    let head_pos = head.into_position(width);

                    let mv = Move::all()
                        .iter()
                        .filter(|mv| {
                            let new_head = head_pos.add_vec(mv.to_vector());
                            let ci = self.embedded.as_wrapped_cell_index(new_head);

                            !self.embedded.cell_is_body(ci) && !self.embedded.cell_is_snake_head(ci)
                        })
                        .choose(rng)
                        .copied()
                        .unwrap_or(Move::Up);
                    (SnakeId(idx as u8), mv)
                }),
        )
    }
}

impl<T: SimulatorInstruments, N: CN, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
    SimulableGame<T, MAX_SNAKES> for CellBoard<N, BOARD_SIZE, MAX_SNAKES>
{
    #[allow(clippy::type_complexity)]
    fn simulate_with_moves<S>(
        &self,
        instruments: &T,
        snake_ids_and_moves: impl IntoIterator<Item = (Self::SnakeIDType, S)>,
    ) -> Box<dyn Iterator<Item = (Action<MAX_SNAKES>, Self)> + '_>
    where
        S: Borrow<[Move]>,
    {
        Box::new(simulate_with_moves(&self.embedded, instruments, snake_ids_and_moves).map(|v| {
            let (action, board) = v;
            (action, Self { embedded: board})
        }))
    }
}

impl<T: CN, const BOARD_SIZE: usize, const MAX_SNAKES: usize> NeighborDeterminableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn possible_moves(
        &self,
        pos: &Self::NativePositionType,
    ) -> Vec<(Move, Self::NativePositionType)> {
        let width = self.embedded.get_actual_width();

        Move::all()
            .iter()
            .map(|mv| {
                let head_pos = pos.into_position(width);
                let new_head = head_pos.add_vec(mv.to_vector());
                let ci = self.embedded.as_wrapped_cell_index(new_head);

                debug_assert!(!self.embedded.off_board(ci.into_position(width)));

                (mv, new_head, ci)
            })
            .map(|(mv, _, ci)| (*mv, ci))
            .collect()
    }

    fn neighbors(&self, pos: &Self::NativePositionType) -> std::vec::Vec<Self::NativePositionType> {
        self.possible_moves(pos)
            .into_iter()
            .map(|(_, ci)| ci)
            .collect()
    }
}

impl<T: CN, const BOARD_SIZE: usize, const MAX_SNAKES: usize> SnakeBodyGettableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn get_snake_body_vec(&self, snake_id: &Self::SnakeIDType) -> Vec<Self::NativePositionType> {
        self.embedded.get_snake_body_vec(snake_id)
    }
}

impl<T: CN, const BOARD_SIZE: usize, const MAX_SNAKES: usize> SizeDeterminableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn get_width(&self) -> u32 {
        self.embedded.get_width()
    }

    fn get_height(&self) -> u32 {
        self.embedded.get_height()
        
    }
}

impl<T: CN, const BOARD_SIZE: usize, const MAX_SNAKES: usize> Display
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.embedded.fmt(f)
    }
}
impl<T: CN, const BOARD_SIZE: usize, const MAX_SNAKES: usize> SnakeIDGettableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    type SnakeIDType = SnakeId;

    fn get_snake_ids(&self) -> Vec<Self::SnakeIDType> {
        self.embedded.get_snake_ids()
    }
}

impl<T: CN, const BOARD_SIZE: usize, const MAX_SNAKES: usize> PositionGettableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    type NativePositionType = CellIndex<T>;

    fn position_is_snake_body(&self, pos: Self::NativePositionType) -> bool {
        self.embedded.position_is_snake_body(pos)
    }

    fn position_from_native(&self, pos: Self::NativePositionType) -> Position {
        self.embedded.position_from_native(pos)
    }

    fn native_from_position(&self, pos: Position) -> Self::NativePositionType {
        self.embedded.native_from_position(pos)
    }
}

impl<T: CN, const BOARD_SIZE: usize, const MAX_SNAKES: usize> HazardQueryableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn is_hazard(&self, pos: &Self::NativePositionType) -> bool {
        self.embedded.is_hazard(pos)
    }

    fn get_hazard_damage(&self) -> u8 {
        self.embedded.get_hazard_damage()
    }
}

impl<T: CN, const BOARD_SIZE: usize, const MAX_SNAKES: usize> HazardSettableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn set_hazard(&mut self, pos: Self::NativePositionType) {
        self.embedded.set_hazard(pos)
    }

    fn clear_hazard(&mut self, pos: Self::NativePositionType) {
        self.embedded.clear_hazard(pos)
    }
}

impl<T: CN, const BOARD_SIZE: usize, const MAX_SNAKES: usize> HeadGettableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn get_head_as_position(
        &self,
        snake_id: &Self::SnakeIDType,
    ) -> crate::wire_representation::Position {
        self.embedded.get_head_as_position(snake_id)
    }

    fn get_head_as_native_position(
        &self,
        snake_id: &Self::SnakeIDType,
    ) -> Self::NativePositionType {
        self.embedded.get_head_as_native_position(snake_id)
    }
}

impl<T: CN, const BOARD_SIZE: usize, const MAX_SNAKES: usize> FoodGettableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn get_all_food_as_positions(&self) -> Vec<crate::wire_representation::Position> {
        self.embedded.get_all_food_as_positions()
    }

    fn get_all_food_as_native_positions(&self) -> Vec<Self::NativePositionType> {
        self.embedded.get_all_food_as_native_positions()
    }
}

impl<T: CN, const BOARD_SIZE: usize, const MAX_SNAKES: usize> YouDeterminableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn is_you(&self, snake_id: &Self::SnakeIDType) -> bool {
        self.embedded.is_you(snake_id)
    }

    fn you_id(&self) -> &Self::SnakeIDType {
        self.embedded.you_id()
    }
}

impl<T: CN, const BOARD_SIZE: usize, const MAX_SNAKES: usize> LengthGettableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    type LengthType = u16;

    fn get_length(&self, snake_id: &Self::SnakeIDType) -> Self::LengthType {
        self.embedded.get_length(*snake_id)
    }

    fn get_length_i64(&self, snake_id: &Self::SnakeIDType) -> i64 {
        self.embedded.get_length_i64(snake_id)
    }
}

impl<T: CN, const BOARD_SIZE: usize, const MAX_SNAKES: usize> HealthGettableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    type HealthType = u8;
    const ZERO: Self::HealthType = 0;

    fn get_health(&self, snake_id: &Self::SnakeIDType) -> Self::HealthType {
        self.embedded.get_health(snake_id)
    }

    fn get_health_i64(&self, snake_id: &Self::SnakeIDType) -> i64 {
        self.embedded.get_health_i64(snake_id) as i64
    }
}

impl<T: CN, const BOARD_SIZE: usize, const MAX_SNAKES: usize> VictorDeterminableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn is_over(&self) -> bool {
        self.embedded.is_over()
    }

    fn get_winner(&self) -> Option<Self::SnakeIDType> {
        self.embedded.get_winner()
    }

    fn alive_snake_count(&self) -> usize {
        self.embedded.alive_snake_count()
    }
}


#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use itertools::Itertools;
    use rand::{RngCore, SeedableRng};

    use crate::{
        game_fixture,
        types::{
            build_snake_id_map, HeadGettableGame, Move, RandomReasonableMovesGame, SimulableGame,
            SimulatorInstruments, SnakeId, HealthGettableGame,
        },
        wire_representation::Position,
    };

    use super::{Cell, CellBoard4Snakes11x11, CellIndex};

    #[derive(Debug)]
    struct Instruments {}

    impl SimulatorInstruments for Instruments {
        fn observe_simulation(&self, _: std::time::Duration) {}
    }

    #[test]
    fn test_to_hash_round_trips() {
        let g = game_fixture(include_str!("../../../fixtures/wrapped_fixture.json"));
        eprintln!("{}", g.board);
        let snake_ids = build_snake_id_map(&g);
        let orig_wrapped_cell: CellBoard4Snakes11x11 = g.as_wrapped_cell_board(&snake_ids).unwrap();
        let hash = orig_wrapped_cell.pack_as_hash();
        eprintln!("{}", serde_json::to_string(&hash).unwrap());
        eprintln!(
            "{}",
            serde_json::to_string(&CellBoard4Snakes11x11::from_packed_hash(&hash).pack_as_hash())
                .unwrap()
        );
        assert_eq!(
            CellBoard4Snakes11x11::from_packed_hash(&hash),
            orig_wrapped_cell
        );
    }

    #[test]
    fn test_cell_round_trips() {
        let mut c: Cell<u8> = Cell::empty();
        c.set_body_piece(SnakeId(3), CellIndex::new(Position::new(1, 2), 11));
        let as_u32 = c.pack_as_u32();
        assert_eq!(c, Cell::from_u32(as_u32));
    }

    #[test]
    fn test_wrapping_simulation_works() {
        let g = game_fixture(include_str!("../../../fixtures/wrapped_fixture.json"));
        eprintln!("{}", g.board);
        let snake_ids = build_snake_id_map(&g);
        let orig_wrapped_cell: CellBoard4Snakes11x11 = g.as_wrapped_cell_board(&snake_ids).unwrap();
        let mut rng = rand::thread_rng();
        run_move_test(
            orig_wrapped_cell,
            snake_ids.clone(),
            11 * 2 + (rng.next_u32() % 20) as i32,
            0,
            1,
            Move::Up,
        );

        // the input state isn't safe to move down in, but it is if we move one to the right
        let move_map = snake_ids
            .clone()
            .into_iter()
            .map(|(_, sid)| (sid, [Move::Right].as_slice()))
            .collect_vec();
        let instruments = Instruments {};
        let wrapped_for_down = orig_wrapped_cell
            .clone()
            .simulate_with_moves(&instruments, move_map.into_iter())
            .next()
            .unwrap()
            .1;
        run_move_test(
            wrapped_for_down,
            snake_ids.clone(),
            11 * 2 + (rng.next_u32() % 20) as i32,
            0,
            -1,
            Move::Down,
        );

        run_move_test(
            orig_wrapped_cell,
            snake_ids.clone(),
            11 * 2 + (rng.next_u32() % 20) as i32,
            -1,
            0,
            Move::Left,
        );
        run_move_test(
            orig_wrapped_cell,
            snake_ids,
            11 * 2 + (rng.next_u32() % 20) as i32,
            1,
            0,
            Move::Right,
        );

        let mut wrapped = orig_wrapped_cell;
        let mut rng = rand::rngs::SmallRng::from_entropy();
        for _ in 0..15 {
            let move_map = wrapped
                .random_reasonable_move_for_each_snake(&mut rng)
                .into_iter()
                .map(|(sid, mv)| (sid, [mv]))
                .collect_vec();
            wrapped = wrapped
                .simulate_with_moves(
                    &instruments,
                    move_map.iter().map(|(sid, mv)| (*sid, mv.as_slice())),
                )
                .collect_vec()[0]
                .1;
        }
        assert!(wrapped.get_health(&SnakeId(0)) as i32 > 0);
        assert!(wrapped.get_health(&SnakeId(1)) as i32 > 0);
    }

    fn run_move_test(
        orig_wrapped_cell: super::CellBoard4Snakes11x11,
        snake_ids: HashMap<String, SnakeId>,
        rollout: i32,
        inc_x: i32,
        inc_y: i32,
        mv: Move,
    ) {
        let mut wrapped_cell = orig_wrapped_cell;
        let instruments = Instruments {};
        let start_health = wrapped_cell.get_health(&SnakeId(0));
        let move_map = snake_ids
            .into_iter()
            .map(|(_, sid)| (sid, [mv]))
            .collect_vec();
        let start_y = wrapped_cell.get_head_as_position(&SnakeId(0)).y;
        let start_x = wrapped_cell.get_head_as_position(&SnakeId(0)).x;
        for _ in 0..rollout {
            wrapped_cell = wrapped_cell
                .simulate_with_moves(
                    &instruments,
                    move_map
                        .iter()
                        .map(|(sid, mv)| (*sid, mv.as_slice()))
                        .clone(),
                )
                .collect_vec()[0]
                .1;
        }
        let end_y = wrapped_cell.get_head_as_position(&SnakeId(0)).y;
        let end_x = wrapped_cell.get_head_as_position(&SnakeId(0)).x;
        assert_eq!(
            wrapped_cell.get_health(&SnakeId(0)) as i32,
            start_health as i32 - rollout
        );
        assert_eq!(((start_y + (rollout * inc_y)).rem_euclid(11)) as i32, end_y);
        assert_eq!(((start_x + (rollout * inc_x)).rem_euclid(11)) as i32, end_x);
    }
}
