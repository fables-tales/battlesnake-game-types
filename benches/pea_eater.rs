use std::time::{Duration, Instant};

use battlesnake_game_types::{
    compact_representation::{dimensions::FixedWithStoredWidth, StandardCellBoard4Snakes11x11},
    types::{
        RandomReasonableMovesGame, SimulableGame, SimulatorInstruments, StandardFoodPlaceableGame,
        VictorDeterminableGame,
    },
};
use num_format::{Locale, ToFormattedString};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::fs::File;
use tracing_flame::FlameLayer;
use tracing_subscriber::{fmt::Layer, prelude::*, Registry};

type OptimizedCellBoard = battlesnake_game_types::compact_representation::StandardCellBoard<
    u8,
    FixedWithStoredWidth<11, 11, 16>,
    { 16 * 11 },
    4,
>;
#[derive(Debug)]
struct Instruments {}

impl SimulatorInstruments for Instruments {
    fn observe_simulation(&self, _: std::time::Duration) {}
}

fn run_from_fixture_till_end(
    rng: &mut impl Rng,
    instrument: Instruments,
    initial_game: OptimizedCellBoard,
) -> u64 {
    let mut iterations = 0;

    let mut game = initial_game;

    while !game.is_over() {
        let next_move = game
            .random_reasonable_move_for_each_snake(rng)
            .map(|(id, m)| (id, [m]));

        let mut new_game = game
            .simulate_with_moves(&instrument, next_move)
            .next()
            .unwrap()
            .1;
        new_game.place_food(rng);

        game = new_game;

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
    let initial_game = OptimizedCellBoard::convert_from_game(wire, &id_map).unwrap();

    let _flame_layer_guard: Option<_> = if std::env::var("TRACING").is_ok() {
        let fmt_layer = Layer::default();
        let fl = FlameLayer::with_file("./target/tracing.folded").unwrap();
        let subscriber = Registry::default().with(fmt_layer).with(fl.0);

        tracing::subscriber::set_global_default(subscriber).expect("Could not set global default");

        Some(fl.1)
    } else {
        None
    };

    let pprof_guard = if std::env::var("PPROF").is_ok() {
        Some(
            pprof::ProfilerGuardBuilder::default()
                .frequency(70_000)
                .blocklist(&["libc", "libgcc", "pthread", "vdso"])
                .build()
                .unwrap(),
        )
    } else {
        None
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

    let average_iteration_duration = total_time / total_iterations as u32;

    let locale = Locale::en;
    println!(
        "Total iterations: {}",
        total_iterations.to_formatted_string(&locale)
    );
    println!("Total time: {:?}", total_time);
    println!(
        "Iterations per second (rounded down): {}",
        (iterations_per_second.trunc() as u64).to_formatted_string(&locale)
    );
    println!("Time per Iteration: {:?}", average_iteration_duration);
    println!("Average game length: {:.3}", average_game_length);

    if let Some(guard) = pprof_guard {
        if let Ok(report) = guard.report().build() {
            let file = File::create("target/flamegraph.svg").unwrap();
            report.flamegraph(file).unwrap();
        };
    };
}
