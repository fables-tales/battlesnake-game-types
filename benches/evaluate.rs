use battlesnake_game_types::compact_representation::eval::{
    MoveEvaluatableGame, MoveEvaluatableWithStateGame,
};
use battlesnake_game_types::wire_representation::Game as DEGame;
use battlesnake_game_types::{
    compact_representation::CellBoard4Snakes11x11,
    types::{build_snake_id_map, Move, SnakeId},
};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use itertools::Itertools;

fn bench_compact_repr_start_of_game_no_state(c: &mut Criterion) {
    let game_fixture = include_str!("../fixtures/start_of_game.json");
    let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
    let g = g.expect("the json literal is valid");
    let snake_id_mapping = build_snake_id_map(&g);
    let compact: CellBoard4Snakes11x11 = g.as_cell_board(&snake_id_mapping).unwrap();
    let moves = [
        (SnakeId(0), Move::Up),
        (SnakeId(1), Move::Up),
        (SnakeId(2), Move::Up),
        (SnakeId(3), Move::Up),
    ];
    c.bench_function("evaluate compact start of game", |b| {
        b.iter(|| black_box(&compact).evaluate_moves(&moves))
    });
}

fn bench_compact_repr_start_of_game_with_state(c: &mut Criterion) {
    let game_fixture = include_str!("../fixtures/start_of_game.json");
    let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
    let g = g.expect("the json literal is valid");
    let snake_id_mapping = build_snake_id_map(&g);
    let compact: CellBoard4Snakes11x11 = g.as_cell_board(&snake_id_mapping).unwrap();
    let moves = [
        (SnakeId(0), Move::Up),
        (SnakeId(1), Move::Up),
        (SnakeId(2), Move::Up),
        (SnakeId(3), Move::Up),
    ];
    let state = compact.generate_state(moves.iter().map(|(sid, m)| (*sid, vec![*m])).collect_vec());

    c.bench_function("evaluate compact start of game with state", |b| {
        b.iter(|| black_box(&compact).evaluate_moves_with_state(&moves, &state))
    });
}

fn bench_compact_repr_late_stage_no_state(c: &mut Criterion) {
    let game_fixture = include_str!("../fixtures/late_stage.json");
    let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
    let g = g.expect("the json literal is valid");
    let snake_id_mapping = build_snake_id_map(&g);
    let compact: CellBoard4Snakes11x11 = g.as_cell_board(&snake_id_mapping).unwrap();
    let moves = [(SnakeId(0), Move::Up), (SnakeId(1), Move::Up)];
    c.bench_function("evaluate compact late stage", |b| {
        b.iter(|| black_box(&compact).evaluate_moves(&moves))
    });
}

fn bench_compact_repr_late_stage_with_state(c: &mut Criterion) {
    let game_fixture = include_str!("../fixtures/late_stage.json");
    let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
    let g = g.expect("the json literal is valid");
    let snake_id_mapping = build_snake_id_map(&g);
    let compact: CellBoard4Snakes11x11 = g.as_cell_board(&snake_id_mapping).unwrap();
    let moves = [(SnakeId(0), Move::Up), (SnakeId(1), Move::Up)];
    let state = compact.generate_state(moves.iter().map(|(sid, m)| (*sid, vec![*m])).collect_vec());

    c.bench_function("evaluate compact late stage with state", |b| {
        b.iter(|| black_box(&compact).evaluate_moves_with_state(&moves, &state))
    });
}

criterion_group!(
    benches,
    bench_compact_repr_start_of_game_no_state,
    bench_compact_repr_start_of_game_with_state,
    bench_compact_repr_late_stage_no_state,
    bench_compact_repr_late_stage_with_state,
);
criterion_main!(benches);
