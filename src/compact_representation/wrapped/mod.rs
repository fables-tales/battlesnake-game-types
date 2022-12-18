//! A compact board representation that is efficient for simulation
use crate::impl_common_board_traits;
use crate::types::*;

/// you almost certainly want to use the `convert_from_game` method to
/// cast from a json represention to a `CellBoard`
use crate::types::{NeighborDeterminableGame, SnakeBodyGettableGame};
use crate::wire_representation::Game;
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::Rng;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;

use crate::{
    types::{Action, Move, SimulableGame, SimulatorInstruments},
    wire_representation::Position,
};

use super::core::{simulate_with_moves, EvaluateMode};
use super::core::{CellBoard as CCB, CellIndex};
use super::dimensions::{ArcadeMaze, Custom, Dimensions, Fixed, Square};
use super::CellNum as CN;

/// A compact board representation that is significantly faster for simulation than
/// `battlesnake_game_types::wire_representation::Game`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CellBoard<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize> {
    embedded: CCB<T, D, BOARD_SIZE, MAX_SNAKES>,
}

impl_common_board_traits!(CellBoard);

impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
    CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    /// Asserts that the board is consistent (e.g. no snake holes)
    pub fn assert_consistency(&self) -> bool {
        self.embedded.assert_consistency()
    }

    /// creates a wrapped board from a Wire Representation game
    pub fn convert_from_game(game: Game, snake_ids: &SnakeIDMap) -> Result<Self, Box<dyn Error>> {
        if game.game.ruleset.name != "wrapped" {
            return Err("only wrapped games are supported".into());
        }
        let embedded = CCB::convert_from_game(game, snake_ids)?;
        Ok(CellBoard { embedded })
    }

    /// for debugging, packs this board into a custom json representation
    pub fn pack_as_hash(&self) -> HashMap<String, Vec<u32>> {
        self.embedded.pack_as_hash()
    }

    /// for debugging, unloads a board from a custom json representation
    pub fn from_packed_hash(hash: &HashMap<String, Vec<u32>>) -> Self {
        Self {
            embedded: CCB::from_packed_hash(hash),
        }
    }
}

/// 7x7 board with 4 snakes
pub type CellBoard4SnakesSquare7x7 = CellBoard<u8, Square, { 7 * 7 }, 4>;

/// Used to represent the standard 11x11 game with up to 4 snakes.
pub type CellBoard4SnakesSquare11x11 = CellBoard<u8, Square, { 11 * 11 }, 4>;

/// Used to represent the a 15x15 board with up to 4 snakes. This is the biggest board size that
/// can still use u8s
pub type CellBoard8SnakesSquare15x15 = CellBoard<u8, Square, { 15 * 15 }, 8>;

/// Used to represent the largest UI Selectable board with 8 snakes.
pub type CellBoard8SnakesSquare25x25 = CellBoard<u16, Custom, { 25 * 25 }, 8>;

/// Used to represent an absolutely silly game board
pub type CellBoard16SnakesSquare50x50 = CellBoard<u16, Custom, { 50 * 50 }, 16>;

/// Enum that holds a Cell Board sized right for the given game
#[derive(Debug)]
pub enum BestCellBoard {
    /// A game that can have a max height and width of 7x7 and 4 snakes
    Tiny(Box<CellBoard4SnakesSquare7x7>),
    /// A exactly 7x7 board with 4 snakes
    SmallExact(Box<CellBoard<u8, Fixed<7, 7>, { 7 * 7 }, 4>>),
    /// A game that can have a max height and width of 11x11 and 4 snakes
    Standard(Box<CellBoard4SnakesSquare11x11>),
    /// A exactly 11x11 board with 4 snakes
    MediumExact(Box<CellBoard<u8, Fixed<11, 11>, { 11 * 11 }, 4>>),
    /// A game that can have a max height and width of 15x15 and 4 snakes
    LargestU8(Box<CellBoard8SnakesSquare15x15>),
    /// A exactly 19x19 board with 4 snakes
    LargeExact(Box<CellBoard<u16, Fixed<19, 19>, { 19 * 19 }, 4>>),
    /// A board that fits the Arcade Maze map
    ArcadeMaze(Box<CellBoard<u16, ArcadeMaze, { 19 * 21 }, 4>>),
    /// A board that fits the Arcade Maze map
    ArcadeMaze8Snake(Box<CellBoard<u16, ArcadeMaze, { 19 * 21 }, 8>>),
    /// A game that can have a max height and width of 25x25 and 8 snakes
    Large(Box<CellBoard8SnakesSquare25x25>),
    /// A game that can have a max height and width of 50x50 and 16 snakes
    Silly(Box<CellBoard16SnakesSquare50x50>),
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
        let width = self.board.width;
        let height = self.board.height;
        let num_snakes = self.board.snakes.len();
        let id_map = build_snake_id_map(&self);

        let best_board = if width == 7 && height == 7 && num_snakes <= 4 {
            BestCellBoard::SmallExact(Box::new(CellBoard::convert_from_game(self, &id_map)?))
        } else if width <= 7 && height <= 7 && num_snakes <= 4 {
            BestCellBoard::Tiny(Box::new(CellBoard::convert_from_game(self, &id_map)?))
        } else if width == 11 && height == 11 && num_snakes <= 4 {
            BestCellBoard::MediumExact(Box::new(CellBoard::convert_from_game(self, &id_map)?))
        } else if width <= 11 && height <= 11 && num_snakes <= 4 {
            BestCellBoard::Standard(Box::new(CellBoard::convert_from_game(self, &id_map)?))
        } else if width <= 15 && height <= 15 && num_snakes <= 8 {
            BestCellBoard::LargestU8(Box::new(CellBoard::convert_from_game(self, &id_map)?))
        } else if width == 19 && height == 19 && num_snakes <= 4 {
            BestCellBoard::LargeExact(Box::new(CellBoard::convert_from_game(self, &id_map)?))
        } else if width == 19 && height == 21 && num_snakes <= 4 {
            BestCellBoard::ArcadeMaze(Box::new(CellBoard::convert_from_game(self, &id_map)?))
        } else if width == 19 && height == 21 && num_snakes <= 8 {
            BestCellBoard::ArcadeMaze8Snake(Box::new(CellBoard::convert_from_game(self, &id_map)?))
        } else if width <= 25 && height < 25 && num_snakes <= 8 {
            BestCellBoard::Large(Box::new(CellBoard::convert_from_game(self, &id_map)?))
        } else if width <= 50 && height <= 50 && num_snakes <= 16 {
            BestCellBoard::Silly(Box::new(CellBoard::convert_from_game(self, &id_map)?))
        } else {
            panic!("No board was big enough")
        };

        Ok(best_board)
    }
}

impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
    RandomReasonableMovesGame for CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    fn random_reasonable_move_for_each_snake<'a>(
        &'a self,
        rng: &'a mut impl Rng,
    ) -> Box<dyn std::iter::Iterator<Item = (SnakeId, Move)> + 'a> {
        Box::new(
            self.reasonable_moves_for_each_snake()
                .map(move |(sid, mvs)| (sid, *mvs.choose(rng).unwrap())),
        )
    }
}

impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize> ReasonableMovesGame
    for CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    fn reasonable_moves_for_each_snake(
        &self,
    ) -> Box<dyn std::iter::Iterator<Item = (SnakeId, Vec<Move>)> + '_> {
        let width = self.embedded.get_actual_width();
        Box::new(
            self.embedded
                .iter_healths()
                .enumerate()
                .filter(|(_, health)| **health > 0)
                .map(move |(idx, _)| {
                    let head_pos = self.get_head_as_position(&SnakeId(idx as u8));

                    let mvs = IntoIterator::into_iter(Move::all())
                        .filter(|mv| {
                            let mut new_head = head_pos.add_vec(mv.to_vector());
                            let wrapped_x = new_head.x.rem_euclid(self.get_width() as i32);
                            let wrapped_y = new_head.y.rem_euclid(self.get_height() as i32);

                            new_head = Position {
                                x: wrapped_x,
                                y: wrapped_y,
                            };

                            let ci = CellIndex::new(new_head, width);

                            if self.off_board(new_head) {
                                return false;
                            };

                            !self.off_board(new_head)
                                && ((!self.embedded.cell_is_body(ci)
                                    && !self.embedded.cell_is_snake_head(ci))
                                    || self.embedded.cell_is_single_tail(ci))
                        })
                        .collect_vec();
                    let mvs = if mvs.is_empty() { vec![Move::Up] } else { mvs };

                    (SnakeId(idx as u8), mvs)
                }),
        )
    }
}

impl<
        T: SimulatorInstruments,
        N: CN,
        D: Dimensions,
        const BOARD_SIZE: usize,
        const MAX_SNAKES: usize,
    > SimulableGame<T, MAX_SNAKES> for CellBoard<N, D, BOARD_SIZE, MAX_SNAKES>
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
        Box::new(
            simulate_with_moves(
                &self.embedded,
                instruments,
                snake_ids_and_moves,
                EvaluateMode::Wrapped,
            )
            .map(|v| {
                let (action, board) = v;
                (action, Self { embedded: board })
            }),
        )
    }
}

impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
    NeighborDeterminableGame for CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    fn possible_moves<'a>(
        &'a self,
        pos: &Self::NativePositionType,
    ) -> Box<(dyn std::iter::Iterator<Item = (Move, CellIndex<T>)> + 'a)> {
        let width = self.embedded.get_actual_width();
        let head_pos = pos.into_position(width);

        Box::new(
            Move::all_iter()
                .map(move |mv| {
                    let new_head = head_pos.add_vec(mv.to_vector());
                    let ci = self.embedded.as_wrapped_cell_index(new_head);

                    debug_assert!(!self.embedded.off_board(ci.into_position(width)));

                    (mv, new_head, ci)
                })
                .map(|(mv, _, ci)| (mv, ci)),
        )
    }

    fn neighbors<'a>(
        &'a self,
        pos: &Self::NativePositionType,
    ) -> Box<(dyn Iterator<Item = CellIndex<T>> + 'a)> {
        Box::new(self.possible_moves(pos).map(|(_, ci)| ci))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use itertools::Itertools;
    use rand::{RngCore, SeedableRng};

    use crate::{
        compact_representation::core::Cell,
        game_fixture,
        types::{
            build_snake_id_map, HeadGettableGame, HealthGettableGame, Move,
            NeighborDeterminableGame, RandomReasonableMovesGame, ReasonableMovesGame,
            SimulableGame, SimulatorInstruments, SnakeId,
        },
        wire_representation::Position,
    };

    use super::{CellBoard4SnakesSquare11x11, CellIndex};

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
        let orig_wrapped_cell: CellBoard4SnakesSquare11x11 =
            g.as_wrapped_cell_board(&snake_ids).unwrap();
        let hash = orig_wrapped_cell.pack_as_hash();
        eprintln!("{}", serde_json::to_string(&hash).unwrap());
        eprintln!(
            "{}",
            serde_json::to_string(
                &CellBoard4SnakesSquare11x11::from_packed_hash(&hash).pack_as_hash()
            )
            .unwrap()
        );
        assert_eq!(
            CellBoard4SnakesSquare11x11::from_packed_hash(&hash),
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
        let orig_wrapped_cell: CellBoard4SnakesSquare11x11 =
            g.as_wrapped_cell_board(&snake_ids).unwrap();
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
        orig_wrapped_cell: super::CellBoard4SnakesSquare11x11,
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

    #[test]
    fn test_wrapped_panic() {
        //        {"lengths":[9,0,19,0],"healths":[61,0,88,0],"hazard_damage":[0],"cells":[655361,5,5,5,5,5,5,5,5,5,589825,1,720897,786433,851969,5,5,5,5,5,1376257,917510,5,5,5,5,5,5,5,5,5,5,5,2818561,2163201,2228737,2294273,2359814,2425345,5,5,5,5,3539457,2949633,3670529,5,5,5,2490881,5,5,5,5,2884097,4260353,3604993,4,5,5,3211777,3932673,3998209,4063745,4129281,4194817,5,5,5,5,5,5,5,5,5,5,5,5,4,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,4,5,5,5,5,5,5,5],"heads":[21,0,37,0],"actual_width":[11]}
        //
        // this panic was because we were simulating a snake with zero health, which is always consistent because
        // we essentially "break" the snake in the cell representation when we kill it.
        let orig_crash_game = game_fixture(include_str!("../../../fixtures/wrapped_panic.json"));
        let snake_ids = build_snake_id_map(&orig_crash_game);
        let compact_ids: Vec<SnakeId> = snake_ids.iter().map(|(_, v)| *v).collect();

        let instruments = Instruments {};
        {
            // this json fixture is the frame at which we crashed, and it comes from a deep forward simulation of orig_crash_game
            let json_hash = include_str!("../../../fixtures/crash_json_hash.json");
            let hm = serde_json::from_str(json_hash).unwrap();
            let game = super::CellBoard4SnakesSquare11x11::from_packed_hash(&hm);
            eprintln!("{}", orig_crash_game.board);
            dbg!(&compact_ids);
            let snakes_and_moves = compact_ids.iter().map(|id| (*id, vec![Move::Up]));
            let mut results = game
                .simulate_with_moves(&instruments, snakes_and_moves)
                .collect_vec();
            assert!(results.len() == 1);
            let (mvs, g) = results.pop().unwrap();
            dbg!(mvs);
            g.assert_consistency();
            g.simulate(&instruments, compact_ids.clone()).for_each(drop);
        }

        {
            let snakes_and_moves = vec![
                (SnakeId(0), [Move::Up].as_slice()),
                (SnakeId(1), [Move::Right].as_slice()),
                (SnakeId(2), [Move::Up].as_slice()),
                (SnakeId(3), [Move::Up].as_slice()),
            ];
            let json_hash = include_str!("../../../fixtures/another_wraped_panic_serialized.json");
            let hm = serde_json::from_str(json_hash).unwrap();
            let game = super::CellBoard4SnakesSquare11x11::from_packed_hash(&hm);
            game.assert_consistency();
            eprintln!(
                "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n!!!!!!!!!!!!!!!!!!!!!!!!!!!!1\n{}",
                game
            );
            let mut results = game
                .simulate_with_moves(&instruments, snakes_and_moves)
                .collect_vec();
            assert!(results.len() == 1);
            let (mvs, g) = results.pop().unwrap();
            dbg!(mvs);
            eprintln!("{}", g);
            g.assert_consistency();
            g.simulate(&instruments, compact_ids.clone()).for_each(drop);
        }
        {
            let snakes_and_moves = vec![
                (SnakeId(0), [Move::Down].as_slice()),
                (SnakeId(1), [Move::Left].as_slice()),
                (SnakeId(2), [Move::Up].as_slice()),
            ];
            let json_hash = include_str!("../../../fixtures/another_wrapped_panic.json");
            let hm = serde_json::from_str(json_hash).unwrap();
            let game = super::CellBoard4SnakesSquare11x11::from_packed_hash(&hm);
            game.assert_consistency();
            eprintln!(
                "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n!!!!!!!!!!!!!!!!!!!!!!!!!!!!1\n{}",
                game
            );
            let mut results = game
                .simulate_with_moves(&instruments, snakes_and_moves)
                .collect_vec();
            assert!(results.len() == 1);
            let (mvs, g) = results.pop().unwrap();
            dbg!(mvs);
            eprintln!("{}", g);
            // head to head collision of 0 and 1 here
            assert_eq!(g.get_health(&SnakeId(0)), 0);
            assert_eq!(g.get_health(&SnakeId(1)), 0);
            g.assert_consistency();
            g.simulate(&instruments, compact_ids).for_each(drop);
        }
    }

    #[test]
    fn test_neighbors_and_possible_moves_cornered() {
        let g = game_fixture(include_str!("../../../fixtures/cornered_wrapped.json"));
        let snake_id_mapping = build_snake_id_map(&g);
        let compact: CellBoard4SnakesSquare11x11 =
            g.as_wrapped_cell_board(&snake_id_mapping).unwrap();

        let head = compact.get_head_as_native_position(&SnakeId(0));
        assert_eq!(head, CellIndex(10 * 11));

        let expected_possible_moves = vec![
            (Move::Up, CellIndex(0)),
            (Move::Down, CellIndex(9 * 11)),
            (Move::Left, CellIndex(10 * 11 + 10)),
            (Move::Right, CellIndex(10 * 11 + 1)),
        ];

        assert_eq!(
            compact.possible_moves(&head).collect::<Vec<_>>(),
            expected_possible_moves
        );

        assert_eq!(
            compact.neighbors(&head).collect::<Vec<_>>(),
            expected_possible_moves
                .into_iter()
                .map(|(_, pos)| pos)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn reasonable_moves_for_each_snake_mojave_12_18_12_34() {
        let g = game_fixture(include_str!("../../../fixtures/mojave_12_18_12_34.json"));
        let snake_id_mapping = build_snake_id_map(&g);
        let compact: CellBoard4SnakesSquare11x11 =
            g.as_wrapped_cell_board(&snake_id_mapping).unwrap();

        let moves = compact.reasonable_moves_for_each_snake().collect_vec();

        assert_eq!(
            moves,
            vec![
                (SnakeId(0), vec![Move::Up, Move::Down]),
                (SnakeId(1), vec![Move::Down, Move::Left, Move::Right]),
            ]
        );
    }
}
