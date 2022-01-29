use battlesnake_game_types::compact_representation::eval::{
    MoveEvaluatableWithStateGame,
};
use battlesnake_game_types::wire_representation::Game as DEGame;
use battlesnake_game_types::{
    compact_representation::CellBoard4Snakes11x11,
    types::{build_snake_id_map, Move, SnakeId},
};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use itertools::Itertools;

fn bench_compact_repr_start_of_game_with_state(c: &mut Criterion) {
    let game_fixture = include_str!("../fixtures/start_of_game.json");
    let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
    let g = g.expect("the json literal is valid");
    let snake_id_mapping = build_snake_id_map(&g);
    let compact: CellBoard4Snakes11x11 = g.as_cell_board(&snake_id_mapping).unwrap();
    let moves = [
        (SnakeId(0), vec![Move::Up]),
        (SnakeId(1), vec![Move::Up]),
        (SnakeId(2), vec![Move::Up]),
        (SnakeId(3), vec![Move::Up]),
    ];
    let state = compact.generate_state(moves.iter());
    let individual_moves = moves.iter().map(|(sid, mvs)| (*sid, mvs[0])).collect_vec();

    c.bench_function("evaluate compact start of game with state", |b| {
        b.iter(|| black_box(&compact).evaluate_moves_with_state(individual_moves.iter(), &state))
    });
}
fn bench_compact_repr_late_stage_with_state(c: &mut Criterion) {
    let game_fixture = include_str!("../fixtures/late_stage.json");
    let g: Result<DEGame, _> = serde_json::from_slice(game_fixture.as_bytes());
    let g = g.expect("the json literal is valid");
    let snake_id_mapping = build_snake_id_map(&g);
    let compact: CellBoard4Snakes11x11 = g.as_cell_board(&snake_id_mapping).unwrap();
    let moves = [(SnakeId(0), vec![Move::Up]), (SnakeId(1), vec![Move::Up])];
    let state = compact.generate_state(moves.iter());
    let individual_moves = moves.iter().map(|(sid, mvs)| (*sid, mvs[0])).collect_vec();

    c.bench_function("evaluate compact late stage with state", |b| {
        b.iter(|| black_box(&compact).evaluate_moves_with_state(individual_moves.iter(), &state))
    });
}

criterion_group!(
    benches,
    bench_compact_repr_start_of_game_with_state,
    bench_compact_repr_late_stage_with_state,
);
criterion_main!(benches);
