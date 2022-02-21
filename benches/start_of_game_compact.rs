use battlesnake_game_types::types::SimulatorInstruments;
use battlesnake_game_types::wire_representation::Game as DEGame;
use battlesnake_game_types::{
    compact_representation::StandardCellBoard4Snakes11x11,
    types::{build_snake_id_map, Move, SimulableGame, SnakeIDGettableGame, SnakeId},
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[derive(Debug)]
struct Instruments {}

impl SimulatorInstruments for Instruments {
    fn observe_simulation(&self, _: std::time::Duration) {}
}
fn bench_this(compact: &StandardCellBoard4Snakes11x11, instruments: &Instruments) {
    compact
        .simulate_with_moves(
            instruments,
            vec![
                (SnakeId(0), [Move::Up].as_slice()),
                (SnakeId(1), [Move::Right].as_slice()),
                (SnakeId(2), [Move::Down].as_slice()),
                (SnakeId(3), [Move::Left].as_slice()),
            ],
        )
        .for_each(|_| {});
}

fn bench_compact_full(compact: &StandardCellBoard4Snakes11x11, instruments: &Instruments) {
    compact
        .simulate(instruments, compact.get_snake_ids())
        .for_each(|_| {});
}

fn bench_compact_repr_start_of_game(c: &mut Criterion) {
    let game_fixture = include_str!("../fixtures/start_of_game.json");
    let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
    let g = g.expect("the json literal is valid");
    let snake_id_mapping = build_snake_id_map(&g);
    let compact: StandardCellBoard4Snakes11x11 = g.as_cell_board(&snake_id_mapping).unwrap();
    let instruments = Instruments {};
    c.bench_function("compact start of game", |b| {
        b.iter(|| bench_this(black_box(&compact), &instruments))
    });
}

fn bench_compact_repr_start_of_game_full(c: &mut Criterion) {
    let game_fixture = include_str!("../fixtures/start_of_game.json");
    let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
    let g = g.expect("the json literal is valid");
    let snake_id_mapping = build_snake_id_map(&g);
    let compact: StandardCellBoard4Snakes11x11 = g.as_cell_board(&snake_id_mapping).unwrap();
    let instruments = Instruments {};
    c.bench_function("compact start of game - all moves", |b| {
        b.iter(|| bench_compact_full(black_box(&compact), &instruments))
    });
}

fn late_stage_compact_repr(c: &mut Criterion) {
    let game_fixture = include_str!("../fixtures/late_stage.json");
    let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
    let g = g.expect("the json literal is valid");
    let snake_id_mapping = build_snake_id_map(&g);
    let compact: StandardCellBoard4Snakes11x11 = g.as_cell_board(&snake_id_mapping).unwrap();
    let instruments = Instruments {};
    c.bench_function("compact late stage", |b| {
        b.iter(|| bench_compact_full(black_box(&compact), &instruments))
    });
}

criterion_group!(
    benches,
    bench_compact_repr_start_of_game,
    bench_compact_repr_start_of_game_full,
    late_stage_compact_repr,
);
criterion_main!(benches);
