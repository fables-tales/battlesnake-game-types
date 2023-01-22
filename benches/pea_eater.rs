use std::time::{Duration, Instant};

use battlesnake_game_types::{
    compact_representation::StandardCellBoard4Snakes11x11,
    types::{
        RandomReasonableMovesGame, SimulableGame, SimulatorInstruments, StandardFoodPlaceableGame,
        VictorDeterminableGame,
    },
};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use tracing_flame::FlameLayer;
use tracing_subscriber::{fmt::Layer, prelude::*, Registry};

#[derive(Debug)]
struct Instruments {}

impl SimulatorInstruments for Instruments {
    fn observe_simulation(&self, _: std::time::Duration) {}
}

fn run_from_fixture_till_end(
    rng: &mut impl Rng,
    instrument: Instruments,
    initial_game: StandardCellBoard4Snakes11x11,
) -> u64 {
    let mut iterations = 0;

    let mut game = initial_game;

    while !game.is_over() {
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

        iterations += 1;
    }

    iterations
}

fn main() {
    // Initial Setup to happen once
    // TODO: Instead of relying on a static fixture, we should generate a random game state
    let fixture_string = include_str!("../fixtures/e80b70e7-a916-40ca-82d2-ad76e074efe1_0.json");
    let wire =
        serde_json::from_str::<battlesnake_game_types::wire_representation::Game>(fixture_string)
            .unwrap();

    let id_map = battlesnake_game_types::types::build_snake_id_map(&wire);
    let initial_game = battlesnake_game_types::compact_representation::StandardCellBoard4Snakes11x11::convert_from_game(wire, &id_map).unwrap();

    if std::env::var("TRACING").is_ok() {
        let fmt_layer = Layer::default();
        let fl = FlameLayer::with_file("./tracing.folded").unwrap();
        let subscriber = Registry::default().with(fmt_layer).with(fl.0);

        tracing::subscriber::set_global_default(subscriber).expect("Could not set global default");
    };

    let mut rng = SmallRng::from_entropy();
    let mut total_iterations = 0;
    let mut game_lengths = Vec::new();

    let runtime = Duration::from_secs(10);

    let start = Instant::now();

    while start.elapsed() < runtime {
        let length = run_from_fixture_till_end(&mut rng, Instruments {}, initial_game);
        total_iterations += length;
        game_lengths.push(length);
    }
    let total_time = start.elapsed();

    let seconds = total_time.as_secs_f64();
    let iterations_per_second = total_iterations as f64 / seconds;

    let average_game_length = game_lengths.iter().sum::<u64>() as f64 / game_lengths.len() as f64;

    println!("Total iterations: {}", total_iterations);
    println!("Total time: {:?}", total_time);
    println!("Iterations per second: {}", iterations_per_second);
    println!("Average game length: {}", average_game_length);

    // if let Ok(report) = guard.report().build() {
    //     let file = File::create("target/flamegraph.svg").unwrap();
    //     report.flamegraph(file).unwrap();
    // };
}
