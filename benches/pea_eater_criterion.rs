use std::time::Instant;

use battlesnake_game_types::{
    compact_representation::{
        dimensions::{Dimensions, FixedWithStoredWidth},
        standard::CellBoard,
        CellNum, StandardCellBoard, StandardCellBoard4Snakes11x11,
    },
    types::{
        RandomReasonableMovesGame, SimulableGame, SimulatorInstruments, StandardFoodPlaceableGame,
        VictorDeterminableGame,
    },
};
use rand::{rngs::SmallRng, SeedableRng};

#[derive(Debug)]
struct Instruments {}

impl SimulatorInstruments for Instruments {
    fn observe_simulation(&self, _: std::time::Duration) {}
}

trait BenchableGame {
    fn from_wire(wire: battlesnake_game_types::wire_representation::Game) -> Self;

    fn run(self, initial_game: &Self, rng: &mut SmallRng, total_iterations: &mut u64) -> Self;
}

impl<T: CellNum, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize> BenchableGame
    for CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    fn from_wire(wire: battlesnake_game_types::wire_representation::Game) -> Self {
        let id_map = battlesnake_game_types::types::build_snake_id_map(&wire);
        Self::convert_from_game(wire, &id_map).unwrap()
    }

    fn run(self, initial_game: &Self, rng: &mut SmallRng, total_iterations: &mut u64) -> Self {
        let instrument = Instruments {};
        let mut game = self;

        if game.is_over() {
            game = *initial_game;
        } else {
            game = black_box(game);
            let next_move = game
                .random_reasonable_move_for_each_snake(rng)
                .map(|(id, m)| (id, [m]));

            let new_game = game
                .simulate_with_moves(&instrument, next_move)
                .next()
                .unwrap()
                .1;
            game = new_game;

            game.place_food(rng);

            *total_iterations += 1;
        }
        game
    }
}

fn bench_cellboard<Board: BenchableGame + Clone>(b: &mut Bencher) {
    b.iter_custom(|iter_count| {
        let fixture_string =
            include_str!("../fixtures/e80b70e7-a916-40ca-82d2-ad76e074efe1_0.json");
        let wire = serde_json::from_str::<battlesnake_game_types::wire_representation::Game>(
            fixture_string,
        )
        .unwrap();

        let initial_game = Board::from_wire(wire);

        let mut rng = SmallRng::from_entropy();
        let mut total_iterations = 0;

        let mut game = initial_game.clone();

        let start = Instant::now();

        while total_iterations < iter_count {
            game = game.run(&initial_game, &mut rng, &mut total_iterations);
        }

        start.elapsed()
    });
}

use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let mut g = c.benchmark_group("CellBoards");
    g.bench_function("Big Board Not Optimized CellBoard", |b| {
        bench_cellboard::<StandardCellBoard<u8, FixedWithStoredWidth<11, 11, 11>, { 11 * 16 }, 4>>(
            b,
        );
    });
    g.bench_function("Optimized CellBoard", |b| {
        bench_cellboard::<StandardCellBoard<u8, FixedWithStoredWidth<11, 11, 16>, { 11 * 16 }, 4>>(
            b,
        );
    });

    g.bench_function("Standard CellBoard", |b| {
        bench_cellboard::<StandardCellBoard4Snakes11x11>(b);
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
