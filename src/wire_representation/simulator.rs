use crate::cross_product::cross_product;

use super::{BattleSnake, Game, Move, Position, SimulatorInstruments};
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

const HAZARD_DAMAGE: i32 = 15;

pub struct Simulator<'a> {
    g: &'a Game,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BattleSnakeResult {
    Ok(BattleSnake),
    Dead(String, BattleSnake),
}

impl BattleSnakeResult {
    fn id(&self) -> &String {
        match self {
            BattleSnakeResult::Ok(ref x) => &x.id,
            &BattleSnakeResult::Dead(ref s, _) => s,
        }
    }

    fn into_option(self) -> Option<BattleSnake> {
        match self {
            BattleSnakeResult::Ok(x) => Some(x),
            BattleSnakeResult::Dead(_, _) => None,
        }
    }

    pub(crate) fn is_dead(&self) -> bool {
        matches!(self, BattleSnakeResult::Dead(_, _))
    }

    pub(crate) fn body(&self) -> &VecDeque<Position> {
        match self {
            &BattleSnakeResult::Ok(ref x) => &x.body,
            &BattleSnakeResult::Dead(_, ref x) => &x.body,
        }
    }

    pub(crate) fn head(&self) -> Position {
        match self {
            &BattleSnakeResult::Ok(ref x) => x.head,
            &BattleSnakeResult::Dead(_, ref x) => x.head,
        }
    }
}

impl<'a> Simulator<'a> {
    pub fn new(g: &'a Game) -> Self {
        Simulator { g }
    }

    #[cfg(test)]
    pub fn simulate<T: SimulatorInstruments>(
        self,
        instruments: &T,
        snake_ids: Vec<String>,
    ) -> Vec<(Vec<(String, Move)>, Game)> {
        let moves_to_simulate = Move::all();
        let build = snake_ids
            .into_iter()
            .map(|s| (s, moves_to_simulate.clone()))
            .collect::<Vec<_>>();
        self.simulate_with_moves(instruments, build)
    }

    pub fn simulate_with_moves<T: SimulatorInstruments>(
        self,
        instruments: &T,
        snake_ids_and_moves: Vec<(String, Vec<Move>)>,
    ) -> Vec<(Vec<(String, Move)>, Game)> {
        // generate new body positions for each snake
        // take cartesian product of body positions
        // generate games
        let start = Instant::now();
        let mut new_body_map: HashMap<String, Vec<(Move, BattleSnakeResult)>> = HashMap::new(); // snake_id -> Vec<PossiblePosition>
        for (snake, moves_to_simulate) in snake_ids_and_moves.iter() {
            let find_snake = self.g.board.snakes.iter().find(|s| &s.id == snake);
            let snake_struct = find_snake.expect("we passed a bad snake id");
            new_body_map.insert(snake.clone(), vec![]);
            for mv in moves_to_simulate.clone() {
                let simulated = self.forward_simulate(snake_struct, mv);
                if let Some(simulated) = simulated {
                    let borrow_key = new_body_map.get_mut(snake).expect("it's there");
                    borrow_key.push((mv, simulated))
                }
            }
        }

        let possible_new_snakes = new_body_map.values().cloned().collect::<Vec<_>>();
        let cross_product = cross_product(possible_new_snakes);
        let mut games = vec![];

        for snakes in cross_product {
            let mut new_game = self.g.clone();
            new_game.turn += 1;
            let move_map: Vec<(String, Move)> =
                snakes.iter().map(|(mv, s)| (s.id().clone(), *mv)).collect();
            let (_, you) = snakes
                .iter()
                .find(|(_, s)| s.id() == &self.g.you.id)
                .expect("we generated a game with you in it");
            match you {
                BattleSnakeResult::Ok(y) => {
                    new_game.you = y.clone();
                }
                BattleSnakeResult::Dead(_, _) => {
                    new_game.you.health = 0;
                    new_game.board.snakes.retain(|s| s.id != self.g.you.id);
                    games.push((move_map, new_game));
                    continue;
                }
            }

            let mut kill_snakes = vec![];

            new_game.board.snakes = snakes
                .into_iter()
                .filter_map(|(_, s)| s.into_option())
                .collect();
            for snake in new_game.board.snakes.iter() {
                new_game.board.food.retain(|f| f != &snake.head);

                for snake_2 in new_game.board.snakes.iter() {
                    if snake_2.id != snake.id {
                        if snake_2.head == snake.head {
                            if snake.body.len() <= snake_2.body.len() {
                                kill_snakes.push(snake.id.clone());
                            }
                        } else if snake_2.body.contains(&snake.head) {
                            kill_snakes.push(snake.id.clone());
                        }
                    }
                }
            }
            new_game.board.snakes = new_game
                .board
                .snakes
                .into_iter()
                .filter(|s| !kill_snakes.contains(&s.id))
                .collect();
            if kill_snakes.contains(&self.g.you.id)
                || new_game
                    .board
                    .snakes
                    .iter()
                    .find(|s| s.id == self.g.you.id)
                    .is_none()
            {
                new_game.you.health = 0;
            }
            games.push((move_map, new_game));
        }
        let end = Instant::now();
        instruments.observe_simulation(end - start);

        games
    }

    fn forward_simulate(&self, s: &BattleSnake, mv: Move) -> Option<BattleSnakeResult> {
        let old_head = s.head;
        let new_head = s.head.add_vec(mv.to_vector());
        if s.body[1] == new_head {
            return None;
        }
        if self.g.off_board(new_head) {
            let mut new_snake = s.clone();
            new_snake.body.pop_back();
            return Some(BattleSnakeResult::Dead(s.id.clone(), new_snake));
        }

        let mut new_snake = s.clone();
        new_snake.body.pop_back();
        if !self.g.board.food.contains(&new_head) {
            new_snake.health -= 1;
        } else {
            let last = *new_snake.body.back().expect("it's nonempty");
            new_snake.body.push_back(last);
            new_snake.health = 100;
        }
        if new_head == old_head {
            return Some(BattleSnakeResult::Dead(s.id.clone(), new_snake));
        }
        if new_snake.body.contains(&new_head) {
            return Some(BattleSnakeResult::Dead(s.id.clone(), new_snake));
        }
        new_snake.body.push_front(new_head);
        if self.g.board.hazards.contains(&new_head) {
            new_snake.health -= HAZARD_DAMAGE;
        }
        if new_snake.health <= 0 {
            return Some(BattleSnakeResult::Dead(s.id.clone(), new_snake));
        }
        new_snake.head = new_head;
        Some(BattleSnakeResult::Ok(new_snake))
    }
}

#[cfg(test)]
mod tests {
    use super::Game as DEGame;
    use super::*;
    use std::time::Duration;

    #[derive(Debug)]
    struct Instruments {}
    impl SimulatorInstruments for Instruments {
        fn observe_simulation(&self, _: Duration) {}
    }

    impl Instruments {
        pub fn new(_: String) -> Self {
            Instruments {}
        }
    }

    fn test_simulate_games(g: Game, g2: Game) {
        eprintln!("{:?}", g2.you);
        if !g2.board.snakes.iter().find(|s| s.id == g2.you.id).is_some() {
            return;
        }
        let gid = g.game.id.clone();
        let turn = g.turn;
        let snake_ids = g.board.snakes.iter().map(|s| s.id.clone()).collect();
        let sim = Simulator::new(&g);
        let instruments = Instruments::new("test".to_string());
        let res = sim.simulate(&instruments, snake_ids);
        let mut specific_res = res
            .into_iter()
            .filter(|(_, game)| game.you.head == g2.you.head);
        if gid == "dcfedae2-5e97-480b-8c22-72ecf1d198f4" && g.turn == 28 {
            eprintln!("!!!!!!!!!!!!!!!!!!!!!!!!!!");
            eprintln!("!!!!!!!!!!!!!!!!!!!!!!!!!!");
            eprintln!("!!!!!!!!!!!!!!!!!!!!!!!!!!");
            eprintln!("!!!!!!!!!!!!!!!!!!!!!!!!!!AAAAAAAAAA");
            for (_, x) in specific_res.clone() {
                eprintln!("!!!!!!!!!!!!!!!!!!!!!!!!!!BBBBBBBBB");
                eprintln!("{}", x.board);
                eprintln!("!!!!!!!!!!!!!!!!!!!!!!!!!!");
            }
            eprintln!("!!!!!!!!!!!!!!!!!!!!!!!!!!");
            eprintln!("!!!!!!!!!!!!!!!!!!!!!!!!!!");
            eprintln!("!!!!!!!!!!!!!!!!!!!!!!!!!!");
        }
        let matching_game = specific_res.find(|(_, res)| {
            res.board.snakes.iter().all(|s| {
                let c = g2.board.snakes.contains(s);
                c
            })
        });
        let (_, matching_game) =
            matching_game.expect(&format!("we generated a matching game {} {}", gid, turn));
        assert!(matching_game
            .board
            .snakes
            .into_iter()
            .filter(|s| s.health == 100)
            .all(|s| g.board.food.contains(&s.head)));
        assert_eq!(g2.you, matching_game.you, "you match");
    }

    #[allow(dead_code)]
    fn test_simulation(before: &str, after: &str) {
        let g: Result<DEGame, _> = serde_json::from_slice(before.as_bytes());
        let g = g.expect("the json literal is valid");
        eprintln!("before: {}", g.board);
        let g = Game::from(g);

        let g2: Result<DEGame, _> = serde_json::from_slice(after.as_bytes());
        let g2 = g2.expect("the json literal is valid");
        eprintln!("after: {}", g2.board);
        let g2 = Game::from(g2);

        test_simulate_games(g, g2);
    }

    #[test]
    fn test_simple_simulation() {
        let game_fixture = include_str!("../../fixtures/goes_for_food.json");
        let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
        let g = g.expect("the json literal is valid");
        let game = Game::from(g.clone());
        let sim = Simulator::new(&game);
        let instruments = Instruments::new("test".to_string());
        let res = sim.simulate(&instruments, vec![g.you.id]);
        // we don't simulate self crashes
        assert_eq!(res.len(), 3)
    }

    #[test]
    fn test_kill_logic() {
        let game_fixture = include_str!("../../fixtures/dont_crash_other_snakes");
        let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
        let g = g.expect("the json literal is valid");
        eprintln!("g: {}", g.board);
        let snake_ids = g.board.snakes.iter().map(|s| s.id.clone()).collect();
        let game = Game::from(g.clone());
        let sim = Simulator::new(&game);
        let instruments = Instruments::new("test".to_string());
        let res = sim.simulate(&instruments, snake_ids);
        let interesting = res.into_iter().filter(|(mm, _)| {
            mm.contains(&("gs_XYGMgHpHV64q78BgQKH8kYxP".to_string(), Move::Up))
                && mm.contains(&("gs_Hh7hhfbfM9rTDx3GcyHKKxdc".to_string(), Move::Right))
        });
        for (_, g) in interesting {
            assert!(!g
                .board
                .snakes
                .iter()
                .map(|s| s.id.clone())
                .collect::<Vec<_>>()
                .contains(&"gs_XYGMgHpHV64q78BgQKH8kYxP".to_string()));
            assert!(!g
                .board
                .snakes
                .iter()
                .map(|s| s.id.clone())
                .collect::<Vec<_>>()
                .contains(&"gs_Hh7hhfbfM9rTDx3GcyHKKxdc".to_string()));
        }
    }
    #[test]
    fn test_kill_logic_2() {
        let game_fixture = include_str!("../../fixtures/body_collision.json");
        let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
        let g = g.expect("the json literal is valid");
        eprintln!("g: {}", g.board);
        let snake_ids = g.board.snakes.iter().map(|s| s.id.clone()).collect();
        let game = Game::from(g.clone());
        let sim = Simulator::new(&game);
        let instruments = Instruments::new("test".to_string());
        let res = sim.simulate(&instruments, snake_ids);
        let interesting = res.into_iter().filter(|(mm, _)| {
            mm.contains(&("bees2".to_string(), Move::Up))
                && mm.contains(&("bees".to_string(), Move::Left))
        });
        for (_, g) in interesting {
            eprintln!("{}", g.board);
            assert!(!g
                .board
                .snakes
                .iter()
                .map(|s| s.id.clone())
                .collect::<Vec<_>>()
                .contains(&"bees".to_string()));
            assert!(g
                .board
                .snakes
                .iter()
                .map(|s| s.id.clone())
                .collect::<Vec<_>>()
                .contains(&"bees2".to_string()));
            assert!(g.you.health == 0)
        }
    }

    #[test]
    fn test_this_game() {
        let game_fixture = include_str!("../../fixtures/tree_search_collision.json");
        let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
        let g = g.expect("the json literal is valid");
        eprintln!("g: {}", g.board);
        let snake_ids = g.board.snakes.iter().map(|s| s.id.clone()).collect();
        let g = Game::from(g.clone());
        let sim = Simulator::new(&g);
        let instruments = Instruments::new("test".to_string());
        let res = sim.simulate(&instruments, snake_ids);
        for (mm, game) in res.iter() {
            if mm
                .iter()
                .find_map(|(sid, mv)| if sid == &g.you.id { Some(mv) } else { None })
                .unwrap_or(&Move::Down)
                == &Move::Right
                && mm
                    .iter()
                    .find_map(|(sid, mv)| if sid != &g.you.id { Some(mv) } else { None })
                    .unwrap_or(&Move::Down)
                    == &Move::Up
            {
                assert_eq!(game.you.health, 0);
            }
        }
    }
}
