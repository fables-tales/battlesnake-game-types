use super::*;

/// Specialized Trait for Move Evaluation in Simulation
/// Some steps of eval can be precomputed for each snake move and don't rely on the cartersian
/// product.
/// This allows the algorithm to be significantly faster in simulation
pub trait MoveEvaluatableWithStateGame: SnakeIDGettableGame + PositionGettableGame + Sized {
    /// The type that is prepared ahead of time and passed into each evaluate usage
    type PreparedState;

    /// Prepare the state for each snake move
    fn generate_state(
        &self,
        snake_ids_and_moves: Vec<(Self::SnakeIDType, Vec<crate::types::Move>)>,
    ) -> Self::PreparedState;

    /// Evaluate the given moves with the precomputed state from Self::generate_state
    fn evaluate_moves_with_state(
        &self,
        moves: &[(Self::SnakeIDType, Move)],
        state: &Self::PreparedState,
    ) -> Self;
}

/// Evaluate the given set of moves on the Board and return a new Game for the result
pub trait MoveEvaluatableGame: SnakeIDGettableGame + PositionGettableGame + Sized {
    /// Evaluate the given moves on this Board
    fn evaluate_moves(&self, moves: &[(Self::SnakeIDType, Move)]) -> Self;
}

impl<T: MoveEvaluatableWithStateGame> MoveEvaluatableGame for T {
    fn evaluate_moves(&self, moves: &[(Self::SnakeIDType, Move)]) -> Self {
        let simulate_ver = moves
            .iter()
            .map(|(sid, m)| (sid.clone(), vec![*m]))
            .collect_vec();
        self.evaluate_moves_with_state(moves, &self.generate_state(simulate_ver))
    }
}

#[derive(Copy, Clone, Debug)]
/// This is the pre-captured state for the two-phase move evaluation
/// For each alive snake we store some things for easy lookups later
/// For dead snakes we don't need to record anything.
/// Snakes can 'die' in the process phase, or in the actual evaluate function.
pub enum SinglePlayerMoveResult<T: CellNum> {
    /// Represents the given snake is alive after phase 1 of evaluation
    Alive {
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
    },
    /// Represents the snake died during phase 1. Cause it ran into a snake (including itself)
    /// [excluding head to heads] or went out of bounds
    Dead,
}

impl<T: CellNum> SinglePlayerMoveResult<T> {
    #[allow(clippy::type_complexity)]
    fn to_tuple(
        self,
    ) -> Option<(
        SnakeId,
        CellIndex<T>,
        CellIndex<T>,
        CellIndex<T>,
        CellIndex<T>,
        u8,
        bool,
        u16,
    )> {
        match self {
            SinglePlayerMoveResult::Alive {
                id,
                new_head,
                old_head,
                new_tail,
                old_tail,
                new_health,
                ate_food,
                new_length,
            } => Some((
                id, old_head, new_head, old_tail, new_tail, new_health, ate_food, new_length,
            )),
            _ => None,
        }
    }
}

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> MoveEvaluatableWithStateGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    type PreparedState = [[SinglePlayerMoveResult<T>; 4]; MAX_SNAKES];

    fn generate_state(
        &self,
        moves: Vec<(Self::SnakeIDType, Vec<crate::types::Move>)>,
    ) -> Self::PreparedState {
        let mut new_heads = [[SinglePlayerMoveResult::Dead; 4]; MAX_SNAKES];

        for (id, mvs) in moves.iter() {
            for m in mvs {
                let old_head = self.get_head_as_native_position(id);
                let old_tail = self
                    .get_cell(old_head)
                    .get_tail_position(old_head)
                    .expect("We came from a head so we should have a tail");

                let new_head_position =
                    old_head.into_position(Self::width()).add_vec(m.to_vector());
                let new_head = if self.off_board(new_head_position) {
                    continue;
                } else {
                    CellIndex::<T>::new(new_head_position, Self::width())
                };

                // TWe calculate the 'neck' so that we can avoid the 'instant death'
                // of moving into your neck
                let neck = {
                    let mut curr = old_tail;
                    let mut prev = curr;

                    while curr != old_head {
                        prev = curr;
                        curr = self.get_cell(curr).get_next_index().unwrap();
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

                if new_health == Self::ZERO {
                    continue;
                };

                new_heads[id.as_usize()][m.as_index()] = SinglePlayerMoveResult::Alive {
                    id: *id,
                    new_head,
                    old_head,
                    new_tail,
                    old_tail,
                    new_health,
                    ate_food,
                    new_length,
                };
            }
        }

        new_heads
    }

    fn evaluate_moves_with_state(
        &self,
        moves: &[(Self::SnakeIDType, Move)],
        new_heads: &Self::PreparedState,
    ) -> Self {
        let mut new = *self;

        for (id, m) in moves {
            let result = new_heads[id.as_usize()][m.as_index()];

            match result {
                SinglePlayerMoveResult::Alive {
                    id,
                    old_head,
                    new_tail,
                    old_tail,
                    new_health,
                    ate_food,
                    new_length,
                    ..
                } => {
                    // Step 1a is delayed and done later. This is to not run into issues with
                    // overriding someone elses tail which would break the representation and make it
                    // impossible to correctly remove the tail if the snake dies.

                    // Remove old tail
                    let old_tail_cell = new.get_cell(old_tail);
                    if old_tail_cell.is_double_stacked_piece() {
                        new.set_cell_body_piece(old_tail, id, old_tail_cell.idx);
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
                        new.set_cell_double_stacked(new_tail, id, new_tail_cell.idx);

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
        for (id, m) in moves {
            let result = new_heads[id.as_usize()][m.as_index()];

            if let SinglePlayerMoveResult::Alive { id, new_head, .. } = result {
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
            .filter_map(|result| result.to_tuple())
            .into_group_map_by(|t| t.2);
        let head_to_head_collistions = grouped_heads
            .iter()
            .filter(|(_key, values)| values.len() >= 2);

        for (_pos, snake_move_info) in head_to_head_collistions {
            let all_snakes_same_length = snake_move_info
                .iter()
                .map(|x| new.get_length(x.0))
                .dedup()
                .count()
                == 1;

            let winner = if all_snakes_same_length {
                None
            } else {
                Some(
                    snake_move_info
                        .iter()
                        .map(|i| (*i, new.get_length(i.0)))
                        .max_by_key(|x| x.1)
                        .unwrap()
                        .0,
                )
            };

            for (loser, _, _, _, _, _, _, _) in snake_move_info
                .iter()
                .filter(|x| Some(x.0) != winner.map(|x| x.0))
            {
                to_kill[loser.as_usize()] = true;
            }
        }

        for (id, old_head, new_head, _old_tail, new_tail, _, _, _) in moves
            .iter()
            .map(|(id, m)| new_heads[id.as_usize()][m.as_index()])
            .flat_map(|result| result.to_tuple())
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

        new
    }
}
