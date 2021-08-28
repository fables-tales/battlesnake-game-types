use battlesnake_game_types::types::SimulatorInstruments;
use battlesnake_game_types::wire_representation::Game as DEGame;
use battlesnake_game_types::{
    compact_representation::CellBoard4Snakes11x11,
    types::{build_snake_id_map, Move, SimulableGame, SnakeIDGettableGame, SnakeId},
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[derive(Debug)]
struct Instruments {}

impl SimulatorInstruments for Instruments {
    fn observe_simulation(&self, _: std::time::Duration) {}
}
fn bench_this(compact: &CellBoard4Snakes11x11, instruments: &Instruments) {
    compact.simulate_with_moves(
        instruments,
        vec![
            (SnakeId(0), vec![Move::Up]),
            (SnakeId(1), vec![Move::Right]),
            (SnakeId(2), vec![Move::Down]),
            (SnakeId(3), vec![Move::Left]),
        ],
    );
}

fn bench_compact_full(compact: &CellBoard4Snakes11x11, instruments: &Instruments) {
    compact.simulate(instruments, compact.get_snake_ids());
}

fn bench_non_compact_full(game: &DEGame, instruments: &Instruments) {
    game.simulate(instruments, game.get_snake_ids());
}

fn bench_this_2(compact: &DEGame, instruments: &Instruments) {
    let moves = vec![
        ("gs_YkwKKSmYwqFFgDk9BycMvWf8".to_string(), vec![Move::Up]),
        ("gs_vbvwfwk6jBc4jmCrKCbdJh3G".to_string(), vec![Move::Right]),
        ("gs_6kQVWJXt9BFpD6dchrmX8qpM".to_string(), vec![Move::Down]),
        ("gs_vbvwfwk6jBc4jmCrKCbdJh3G".to_string(), vec![Move::Left]),
    ];
    compact.simulate_with_moves(instruments, moves);
}

fn bench_compact_repr_start_of_game(c: &mut Criterion) {
    let game_fixture = include_str!("../fixtures/start_of_game.json");
    let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
    let g = g.expect("the json literal is valid");
    let snake_id_mapping = build_snake_id_map(&g);
    let compact: CellBoard4Snakes11x11 = g.as_cell_board(&snake_id_mapping).unwrap();
    let instruments = Instruments {};
    c.bench_function("compact start of game", |b| {
        b.iter(|| bench_this(black_box(&compact), &instruments))
    });
}

fn late_stage_compact_repr(c: &mut Criterion) {
    let game_fixture = include_str!("../fixtures/late_stage.json");
    let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
    let g = g.expect("the json literal is valid");
    let snake_id_mapping = build_snake_id_map(&g);
    let compact: CellBoard4Snakes11x11 = g.as_cell_board(&snake_id_mapping).unwrap();
    let instruments = Instruments {};
    c.bench_function("compact late stage", |b| {
        b.iter(|| bench_compact_full(black_box(&compact), &instruments))
    });
}

fn late_stage_vec_repr(c: &mut Criterion) {
    let game_fixture = include_str!("../fixtures/late_stage.json");
    let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
    let g = g.expect("the json literal is valid");
    let instruments = Instruments {};
    c.bench_function("vec late stage", |b| {
        b.iter(|| bench_non_compact_full(black_box(&g), &instruments))
    });
}
fn bench_vec_repr_start_of_game(c: &mut Criterion) {
    let game_fixture = include_str!("../fixtures/start_of_game.json");
    let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
    let g = g.expect("the json literal is valid");
    let instruments = Instruments {};
    c.bench_function("vec game start of game", |b| {
        b.iter(|| bench_this_2(black_box(&g), &instruments))
    });
}

criterion_group!(
    benches,
    bench_compact_repr_start_of_game,
    bench_vec_repr_start_of_game,
    late_stage_compact_repr,
    late_stage_vec_repr
);
criterion_main!(benches);
