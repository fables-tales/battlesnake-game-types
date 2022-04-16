//! traits and structs for simulating hazard algorithms in battlesnake
//! implements non-deterministic forecasting for randomized algorithms

use std::error::Error;

use crate::{wire_representation::{Game, Position}, types::Move};

/// Represents a hazard algorithms that can only be wound forward (interface permits one turn at a time)
pub trait ForwardOnlyHazardAlgorithm<T>: Clone + std::fmt::Debug {
    /// use this to initialize the hazard algorithm. See implementation
    /// specific notes for how to use for each hazard algorithm. The returned
    /// iterator is the list of positions observed on the first creation of hazards
    fn observe(&mut self, game: &Game) -> Result<Box <dyn Iterator<Item=Position>>, Box<dyn Error>>;

    /// determines if this forward only hazard algorithm is ready for inc calls
    fn is_ready_for_inc(&self) -> bool;

    /// Wind the turn forward by one. Returned iterator represents the new hazards
    /// that were created on the wound turn.
    fn inc_turn(&mut self) -> Box<dyn Iterator<Item=T>>;

    /// get the current turn of this hazard algorithm
    fn current_turn(&self) -> usize;
}

#[derive(Clone, Copy, Debug)]
/// Hazard algorithm that does not do anything
pub struct NoopHazard();

impl ForwardOnlyHazardAlgorithm<Position> for NoopHazard {
    fn observe(&mut self, _game: &Game) -> Result<Box <dyn Iterator<Item=Position>>, Box<dyn Error>> {
        Ok(Box::new(std::iter::empty()))
    }

    fn is_ready_for_inc(&self) -> bool {
        false
    }

    fn inc_turn(&mut self) -> Box<dyn Iterator<Item=Position>> {
        Box::new(std::iter::empty())
    }

    fn current_turn(&self) -> usize {
        0
    }
}


/// Spiral hazard algorithm
#[derive(Debug, Copy, Clone)]
pub struct SpiralHazard {
    hazard_every_turns: u8,
    seed_cell: Position,
    first_turn_seen: u16,
    current_turn: u16,
    next_hazard_cell: Position,
    direction: Move,
}

impl SpiralHazard {
    /// Construct an unitialized spiral hazard algorithm
    pub fn new() -> Self {
        SpiralHazard {
            hazard_every_turns: 0,
            seed_cell: Position { x: 0, y: 0 },
            first_turn_seen: 0,
            current_turn: 0,
            next_hazard_cell: Position { x: 0, y: 0 },
            direction: Move::Up,
        }
    }
}

impl Default for SpiralHazard {
    fn default() -> Self {
        Self::new()
    }
}

// the hazard algorithm forms odd squares, so like:
// x
// then
// x x x
// x x x
// x x x
// but crucially not:
// x x 
// x x
fn next_perfect_odd_square(n: u16) -> u16 {
    // 1 -> 1
    // 2 -> 9
    // 3 -> 9
    // 9 -> 25
    // what's the solution?

    // current square base
    let current_base = (n as f32).sqrt().floor() as u16;
    // next square base
    // e.g. if the input is 2, this is 2
    let mut next_base = current_base + 1;
    if next_base % 2 == 0 {
        next_base += 1;
    }

    // now it's definitely an odd number
    debug_assert!(next_base % 2 == 1);

    next_base * next_base
}

fn is_perfect_odd_square(n: u16) -> bool {
    let sqrt = (n as f32).sqrt().floor() as u16;
    sqrt * sqrt == n && sqrt % 2 == 1
}

impl ForwardOnlyHazardAlgorithm<Position> for SpiralHazard {
    /// call this with game states until the seed cell has been observed
    /// which will usually be on turn 3, once you've seen the seed cell
    /// you should stop calling observe, and start calling inc_turn to
    /// calculate forward hazard squares
    fn observe(&mut self, game: &Game) -> Result<Box <dyn Iterator<Item=Position>>, Box<dyn Error>> {
        if self.is_ready_for_inc() {
            return Err("already ready for inc".into());
        }
        if self.first_turn_seen == 0 {
            if game.board.hazards.len() > 1 {
                return Err("didn't observe spiral seed".into());
            } else if !game.board.hazards.is_empty() {
                let hazard_pos = game.board.hazards[0];
                self.seed_cell = hazard_pos;

                // TODO: no way to detect this from the payload right now
                self.hazard_every_turns = 3;

                self.first_turn_seen = game.turn as u16;
                self.current_turn = game.turn as u16;

                self.next_hazard_cell = self.seed_cell.add_vec(Move::Up.to_vector());
                self.direction = Move::Right;
                return Ok(Box::new(Some(self.seed_cell).into_iter()))
            }
        }
        Ok(Box::new(None.into_iter()))
    }

    fn is_ready_for_inc(&self) -> bool {
        self.first_turn_seen != 0
    }

    fn current_turn(&self) -> usize {
        self.current_turn as usize
    }

    fn inc_turn(&mut self) -> Box<dyn Iterator<Item=Position>> {
        self.current_turn += 1;
        if self.current_turn % self.hazard_every_turns as u16 == 0 {
            let turns_elapsed = self.current_turn - self.first_turn_seen;
            // plus 1 because the seed cell
            let spawns_elapsed = (turns_elapsed / self.hazard_every_turns as u16) + 1;
            let next_square = next_perfect_odd_square(spawns_elapsed);
            let radius = ((next_square as f32).sqrt()/2.0).floor() as u16;
            let result = self.next_hazard_cell;
            self.next_hazard_cell = self.next_hazard_cell.add_vec(self.direction.to_vector());

            if self.next_hazard_cell.x - self.seed_cell.x == radius as i32 && self.next_hazard_cell.y - self.seed_cell.y == radius as i32 {
                self.direction = Move::Down;
            } else if self.next_hazard_cell.x - self.seed_cell.x == radius as i32 && self.next_hazard_cell.y - self.seed_cell.y == -(radius as i32) {
                self.direction = Move::Left;
            } else if self.next_hazard_cell.x - self.seed_cell.x == -(radius as i32) && self.next_hazard_cell.y - self.seed_cell.y == -(radius as i32) {
                self.direction = Move::Up;
            } else if self.next_hazard_cell.x - self.seed_cell.x == -(radius as i32) && self.next_hazard_cell.y - self.seed_cell.y == radius as i32 {
                debug_assert!(is_perfect_odd_square(spawns_elapsed+1), "spawns_elapsed: {}", spawns_elapsed);
                self.direction = Move::Up;
            }
            if is_perfect_odd_square(spawns_elapsed) {
                self.direction = Move::Right;
            }

            Box::new(Some(result).into_iter())
        } else {
            Box::new(None.into_iter())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, collections::HashSet, iter::FromIterator, path};

    use crate::{wire_representation::{Position, Game}, types::Move};

    use super::{SpiralHazard, ForwardOnlyHazardAlgorithm};

    #[test]
    fn test_next_perfect_square() {
        assert_eq!(9, super::next_perfect_odd_square(1));
        assert_eq!(9, super::next_perfect_odd_square(2));
        assert_eq!(9, super::next_perfect_odd_square(8));
        assert_eq!(25, super::next_perfect_odd_square(9));
    }

    #[test]
    fn test_is_odd_perfect_square() {
        assert!(super::is_perfect_odd_square(1));
        assert!(!super::is_perfect_odd_square(4));
        assert!(super::is_perfect_odd_square(9));
        assert!(super::is_perfect_odd_square(25));
    }

    #[test]
    fn test_spiral_alg() {
        let mut s = SpiralHazard {
            hazard_every_turns: 3,
            seed_cell: Position { x: 0, y: 0 },
            first_turn_seen: 3,
            current_turn: 3,
            next_hazard_cell: Position { x: 0, y: 1 },
            direction: Move::Right,
        };
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: 0, y: 1 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: 1, y: 1 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: 1, y: 0 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: 1, y: -1 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: 0, y: -1 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: -1, y: -1 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: -1, y: 0 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: -1, y: 1 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: -1, y: 2 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: 0, y: 2 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: 1, y: 2 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: 2, y: 2 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: 2, y: 1 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: 2, y: 0 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: 2, y: -1 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: 2, y: -2 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: 1, y: -2 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: 0, y: -2 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: -1, y: -2 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: -2, y: -2 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: -2, y: -1 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: -2, y: 0 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: -2, y: 1 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: -2, y: 2 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: -2, y: 3 });
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().is_none());
        assert!(s.inc_turn().next().unwrap() == Position { x: -1, y: 3 });
    }

    #[test]
    fn test_matches_frames_from_game() {
        let mut maintained_hazards = HashSet::new();
        let mut hazard_alg = SpiralHazard::new();
        let self_file = path::Path::new(env!("CARGO_MANIFEST_DIR"));
        eprintln!("{:?}", self_file);
        for i in 1..=193 {
            let file_name = self_file.join(format!("fixtures/debug_wrapped/debug_game_{}.json", i));
            let file_bytes = fs::read(file_name).unwrap();
            let game: Game = serde_json::from_slice(&file_bytes).unwrap();

            if !hazard_alg.is_ready_for_inc() {
                let iter = hazard_alg.observe(&game).unwrap();
                maintained_hazards.extend(iter);
            } else {
                let new_hazards = hazard_alg.inc_turn();
                maintained_hazards.extend(new_hazards);
                let hazards_set = HashSet::from_iter(game.board.hazards.into_iter());
                assert!(hazard_alg.current_turn == game.turn as u16);
                assert!(hazards_set == maintained_hazards);

            }
        }
    }
}