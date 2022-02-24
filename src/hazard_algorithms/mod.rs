//! traits and structs for simulating hazard algorithms in battlesnake
//! implements non-deterministic forecasting for randomized algorithms

use std::error::Error;

use crate::{wire_representation::{Game, Position}, types::Move};

trait HazardAlgorithm<T>: Clone + std::fmt::Debug {
    fn observe(&mut self, game: &Game) -> Result<(), Box<dyn Error>>;
    fn inc_turn(&mut self) -> Vec<T>;
}

#[derive(Debug, Copy, Clone)]
struct SpiralHazard {
    hazard_every_turns: u8,
    seed_cell: Position,
    first_turn_seen: u16,
    current_turn: u16,
    next_hazard_cell: Position,
    direction: Move,
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

impl HazardAlgorithm<Position> for SpiralHazard {
    fn observe(&mut self, game: &Game) -> Result<(), Box<dyn Error>> {
        if self.first_turn_seen == 0 {
            if game.board.hazards.len() > 1 {
                return Err("didn't observe spiral seed".into());
            } else {
                let hazard_pos = game.board.hazards[0];
                self.seed_cell = hazard_pos;

                // TODO: no way to detect this from the payload right now
                self.hazard_every_turns = 3;

                self.first_turn_seen = game.turn as u16;
                self.current_turn = game.turn as u16;

                self.next_hazard_cell = self.seed_cell.add_vec(Move::Up.to_vector());
                self.direction = Move::Right;
            }
        }
        Ok(())
    }

    fn inc_turn(&mut self) -> Vec<Position> {
        self.current_turn += 1;
        if self.current_turn % self.hazard_every_turns as u16 == 0 {
            let turns_elapsed = self.current_turn - self.first_turn_seen;
            // plus 1 because the seed cell
            let spawns_elapsed = (turns_elapsed / self.hazard_every_turns as u16) + 1;
            let next_square = next_perfect_odd_square(spawns_elapsed);
            let radius = ((next_square as f32).sqrt()/2.0).floor() as u16;
            let result = vec![self.next_hazard_cell];
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

            result
        } else {
            vec![]
        }
    }


}

#[cfg(test)]
mod tests {
    use crate::{wire_representation::Position, types::Move};

    use super::{SpiralHazard, HazardAlgorithm};

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
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: 0, y: 1 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: 1, y: 1 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: 1, y: 0 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: 1, y: -1 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: 0, y: -1 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: -1, y: -1 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: -1, y: 0 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: -1, y: 1 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: -1, y: 2 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: 0, y: 2 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: 1, y: 2 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: 2, y: 2 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: 2, y: 1 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: 2, y: 0 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: 2, y: -1 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: 2, y: -2 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: 1, y: -2 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: 0, y: -2 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: -1, y: -2 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: -2, y: -2 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: -2, y: -1 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: -2, y: 0 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: -2, y: 1 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: -2, y: 2 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: -2, y: 3 }]);
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn().is_empty());
        assert!(s.inc_turn() == vec![Position { x: -1, y: 3 }]);
    }
}