use std::borrow::Borrow;

use itertools::Itertools;

use crate::{
    compact_representation::{core::dimensions::Dimensions, CellNum},
    types::{self, HeadGettableGame, Move, SnakeId, N_MOVES},
};

use super::{CellBoard, CellIndex};

/// Which mode to evaluate in
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvaluateMode {
    Wrapped,
    Standard,
}

#[derive(Copy, Clone, Debug)]
/// Precomputed state for Move Evaluation
/// for a single Snake Move
pub struct AliveMoveResult<T: CellNum> {
    /// This sid for this snake
    id: SnakeId,
    /// CellIndex for where the head used to be
    old_head: CellIndex<T>,
    /// CellIndex where the new head will be
    new_head: CellIndex<T>,
    /// CellIndex where the tail was previously
    old_tail: CellIndex<T>,
    /// CellIndex where the tail will be
    new_tail: CellIndex<T>,
    /// The new health of the snake
    new_health: u8,
    /// True if the snake ate food
    ate_food: bool,
    /// The new length of the snake, after moving and potentially eating
    new_length: u16,
}

#[derive(Copy, Clone, Debug)]
/// This is the pre-captured state for the two-phase move evaluation
/// For each alive snake we store some things for easy lookups later
/// For dead snakes we don't need to record anything.
/// Snakes can 'die' in the process phase, or in the actual evaluate function.
pub enum SinglePlayerMoveResult<T: CellNum> {
    /// Represents the given snake is alive after phase 1 of evaluation
    Alive(AliveMoveResult<T>),
    /// Represents the snake died during phase 1. Cause it ran into a snake (including itself)
    /// [excluding head to heads] or went out of bounds
    Dead,
}

impl<T: CellNum> SinglePlayerMoveResult<T> {
    fn to_alive_struct(self) -> Option<AliveMoveResult<T>> {
        match self {
            SinglePlayerMoveResult::Alive(alive) => Some(alive),
            _ => None,
        }
    }

    pub fn is_dead(&self) -> bool {
        matches!(self, SinglePlayerMoveResult::Dead)
    }
}

impl<T: CellNum, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
    CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    pub fn generate_state<'a, S: 'a>(
        &self,
        moves: impl Iterator<Item = &'a (SnakeId, S)>,
        mode: EvaluateMode,
    ) -> [[SinglePlayerMoveResult<T>; N_MOVES]; MAX_SNAKES]
    where
        S: Borrow<[Move]>,
    {
        let mut new_heads = [[SinglePlayerMoveResult::Dead; 4]; MAX_SNAKES];

        for (id, mvs) in moves {
            if self.healths[id.as_usize()] == 0 {
                continue;
            }
            for m in mvs.borrow() {
                let old_head = self.get_head_as_native_position(id);
                let old_tail = self
                    .get_cell(old_head)
                    .get_tail_position(old_head)
                    .unwrap_or_else(|| {
                        panic!(
                            "We came from a head so we should have a tail snake: {} health: {}",
                            id.0,
                            self.healths[id.as_usize()]
                        )
                    });

                let new_head_position =
                    old_head.into_position(Self::width()).add_vec(m.to_vector());
                let new_head = match mode {
                    EvaluateMode::Wrapped => self.as_wrapped_cell_index(new_head_position),
                    EvaluateMode::Standard => {
                        if self.off_board(new_head_position) {
                            continue;
                        } else {
                            CellIndex::<T>::new(new_head_position, Self::width())
                        }
                    }
                };

                // TWe calculate the 'neck' so that we can avoid the 'instant death'
                // of moving into your neck
                let neck = {
                    let mut curr = old_tail;
                    let mut prev = curr;

                    while curr != old_head {
                        prev = curr;
                        curr = self.get_cell(curr).get_next_index().unwrap_or_else(|| {
                            eprintln!("{}", self);
                            panic!("snake is inconsistent")
                        });
                    }

                    prev
                };
                if new_head == neck {
                    continue;
                }

                let old_tail_cell = self.get_cell(old_tail);
                let new_tail = if old_tail_cell.is_stacked() {
                    old_tail
                } else {
                    old_tail_cell
                        .get_next_index()
                        .expect("We specifically went to a tail so this shouldn't fail")
                };

                let mut new_health = self.healths[id.as_usize()];
                new_health = new_health.saturating_sub(1);
                if self.get_cell(new_head).is_hazard() {
                    new_health = new_health.saturating_sub(self.hazard_damage);
                }

                let ate_food = self.get_cell(new_head).is_food();
                let mut new_length = self.lengths[id.as_usize()];

                if ate_food {
                    new_health = 100;
                    new_length = new_length.saturating_add(1);
                };

                if new_health == 0 {
                    continue;
                };

                new_heads[id.as_usize()][m.as_index()] =
                    SinglePlayerMoveResult::Alive(AliveMoveResult {
                        id: *id,
                        new_head,
                        old_head,
                        new_tail,
                        old_tail,
                        new_health,
                        ate_food,
                        new_length,
                    });
            }
        }

        new_heads
    }

    pub fn evaluate_moves_with_state<'a>(
        &self,
        moves: impl Iterator<Item = &'a (SnakeId, crate::types::Move)>,
        new_heads: &[[SinglePlayerMoveResult<T>; N_MOVES]; MAX_SNAKES],
    ) -> Self
    where
        <Self as types::SnakeIDGettableGame>::SnakeIDType: 'a,
    {
        let moves = moves.collect_vec();
        let mut new = *self;

        for (id, m) in moves.iter() {
            let result = new_heads[id.as_usize()][m.as_index()];

            match result {
                SinglePlayerMoveResult::Alive(AliveMoveResult {
                    id,
                    old_head,
                    new_tail,
                    old_tail,
                    new_health,
                    ate_food,
                    new_length,
                    ..
                }) => {
                    // Step 1a is delayed and done later. This is to not run into issues with
                    // overriding someone elses tail which would break the representation and make it
                    // impossible to correctly remove the tail if the snake dies.

                    // Remove old tail
                    let old_tail_cell = new.get_cell(old_tail);
                    if old_tail_cell.is_double_stacked_piece() {
                        new.set_cell_body_piece(old_tail, id, old_tail_cell.get_idx());
                    } else {
                        new.cell_remove(old_tail);
                        new.set_cell_head(old_head, id, new_tail)
                    }

                    // Apply new health
                    new.healths[id.as_usize()] = new_health;
                    new.lengths[id.as_usize()] = new_length;

                    // Step 2: Any Battlesnake that has found food will consume it
                    // Reset health to max if ate food
                    if ate_food {
                        let new_tail_cell = new.get_cell(new_tail);
                        new.set_cell_double_stacked(new_tail, id, new_tail_cell.get_idx());

                        // Food is removed naturally by overriding the Cell with the body, which will
                        // happen later
                    }
                }
                SinglePlayerMoveResult::Dead => new.kill_and_remove(*id),
            }
        }

        // Step 3: Any new food spawning will be placed in empty squares on the board.
        // This step is ignored because we don't want to guess at food spawn locations as they are
        // random
        let mut to_kill = [false; MAX_SNAKES];

        // Step 4c-d: Collision besides head to head
        for (id, m) in moves.iter() {
            let result = new_heads[id.as_usize()][m.as_index()];

            if let SinglePlayerMoveResult::Alive(AliveMoveResult { id, new_head, .. }) = result {
                let new_head_cell = new.get_cell(new_head);

                if new_head_cell.is_body_segment() || new_head_cell.is_head() {
                    to_kill[id.as_usize()] = true;
                }
            }
        }

        // Step 4e: Head to Head collisions
        let grouped_heads = moves
            .iter()
            .map(|(id, m)| new_heads[id.as_usize()][m.as_index()])
            .filter_map(|result| result.to_alive_struct())
            .into_group_map_by(|t| t.new_head);
        let head_to_head_collistions = grouped_heads
            .iter()
            .filter(|(_key, values)| values.len() >= 2);

        for (head_to_head_collision_pos, snake_move_info) in head_to_head_collistions {
            let max_length = snake_move_info
                .iter()
                .map(|i| (*i, new.get_length(i.id)))
                .max_by_key(|x| x.1)
                .unwrap()
                .1;
            let snake_ids = snake_move_info.iter().map(|i| i.id).collect_vec();
            let cell = new.get_cell(*head_to_head_collision_pos);
            // consider this board:
            //   s . . f . . s s s 3 s
            //   s s s . . . . s s . .
            //   . . s . . . . . . . .
            //   . f s . . . . . . . .
            //   s s s . . . . . . . s
            //   s s f . . s s s s s s
            //   s s . . 2 s . . . . s
            //   s s s s . . . . s . s
            //   . . . . . . s s s . .
            //   s s s s . . s . . 0 .
            //   s . . . . . s . 1 s s
            // it's a little hard to see, but if at the same time
            // snake 3 moves up: it will warp around on the second column from the top row to the bottom row (from 10,9 to 0,9),
            // snake 1 moves right from (0,8 to 0,9) it will also be on 0,9
            // and snake 0 has a body segment (currently it's neck) on 0,
            // this will cause a head to head collision between snake 1 and snake 3 on snake 0's neck.
            // this statement needs to be added to the winner check, because if it isn't, the neck cell for
            // snake 0 will be removed, causing the body to go in to an inconsistent state
            let head_to_head_collision_on_another_snake = cell.is_body_segment()
                && !cell.is_head()
                && !snake_ids.contains(&cell.get_snake_id().unwrap());

            let multiple_snakes_max_length = snake_move_info
                .iter()
                .filter(|x| new.get_length(x.id) == max_length)
                .count()
                != 1;

            let winner = if multiple_snakes_max_length || head_to_head_collision_on_another_snake {
                None
            } else {
                Some(
                    snake_move_info
                        .iter()
                        .map(|i| (*i, new.get_length(i.id)))
                        .max_by_key(|x| x.1)
                        .unwrap()
                        .0,
                )
            };

            for AliveMoveResult { id: dead, .. } in snake_move_info
                .iter()
                .filter(|x| Some(x.id) != winner.map(|x| x.id))
            {
                to_kill[dead.as_usize()] = true;
            }

            if winner.is_none() && !head_to_head_collision_on_another_snake {
                new.cell_remove(*head_to_head_collision_pos);
            }
        }

        for result in moves
            .iter()
            .map(|(id, m)| new_heads[id.as_usize()][m.as_index()])
        {
            if let SinglePlayerMoveResult::Alive(AliveMoveResult {
                id,
                old_head,
                new_head,
                new_tail,
                ..
            }) = result
            {
                if to_kill[id.as_usize()] {
                    // Kill any player killed via collisions
                    new.kill_and_remove(id);
                } else {
                    // Move Head
                    new.heads[id.as_usize()] = new_head;
                    new.set_cell_head(new_head, id, new_tail);

                    let old_head_cell = self.get_cell(old_head);
                    if old_head_cell.is_triple_stacked_piece() {
                        new.set_cell_double_stacked(old_head, id, new_head);
                    } else {
                        new.set_cell_body_piece(old_head, id, new_head);
                    }
                }
            }
        }

        new
    }
}
