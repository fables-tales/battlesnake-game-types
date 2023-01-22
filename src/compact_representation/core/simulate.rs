use std::{borrow::Borrow, time::Instant};

use itertools::Itertools;
use tracing::instrument;

use crate::types::{Action, Move, SimulatorInstruments, SnakeId, N_MOVES};

use super::{cell_board::EvaluateMode, dimensions::Dimensions, CellBoard, CellNum};

#[instrument(level = "trace", skip_all)]
pub fn simulate_with_moves<
    'a,
    S,
    I: SimulatorInstruments,
    T: CellNum,
    D: Dimensions,
    const BOARD_SIZE: usize,
    const MAX_SNAKES: usize,
>(
    board: &'a CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>,
    instruments: &I,
    snake_ids_and_moves: impl IntoIterator<Item = (SnakeId, S)>,
    evaluate_mode: EvaluateMode,
) -> Box<dyn Iterator<Item = (Action<MAX_SNAKES>, CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>)> + 'a>
where
    S: Borrow<[Move]>,
{
    let start = Instant::now();
    let snake_ids_and_moves = snake_ids_and_moves.into_iter().collect_vec();

    let mut snake_ids_we_are_simulating = [false; MAX_SNAKES];
    for (snake_id, _) in snake_ids_and_moves.iter() {
        snake_ids_we_are_simulating[snake_id.0.as_usize()] = true;
    }

    // [
    // sid major, move minor
    // [ some_reulst_struct, some_dead_struct ]
    // [ some_dead_struct, some_dead_struct ] // snake we didn't simulate
    let states = board.generate_state(snake_ids_and_moves.iter(), evaluate_mode);
    let mut dead_snakes_table = [[false; N_MOVES]; MAX_SNAKES];

    for (sid, result_row) in states.iter().enumerate() {
        for (move_index, move_result) in result_row.iter().enumerate() {
            dead_snakes_table[sid][move_index] = move_result.is_dead();
        }
    }

    let ids_and_moves_product = snake_ids_and_moves
        .into_iter()
        .map(|(snake_id, moves)| {
            let first_move = moves.borrow()[0];
            let mvs = moves
                .borrow()
                .iter()
                .filter(|mv| !dead_snakes_table[snake_id.0 as usize][mv.as_index()])
                .map(|mv| (snake_id, *mv))
                .collect_vec();
            if mvs.is_empty() {
                vec![(snake_id, first_move)]
            } else {
                mvs
            }
        })
        .multi_cartesian_product();
    let results = ids_and_moves_product.into_iter().map(move |m| {
        let action = Action::collect_from(m.iter());

        let game = board.evaluate_moves_with_state(m.iter(), &states);
        if !game.assert_consistency() {
            panic!(
                "caught an inconsistent simulate, moves: {:?} orig: {}, new: {}",
                m, board, game
            );
        }
        (action, game)
    });
    let return_value = Box::new(results);
    let end = Instant::now();
    instruments.observe_simulation(end - start);
    return_value
}
