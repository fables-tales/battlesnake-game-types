use battlesnake_game_types::types::{
    HealthGettableGame, RandomReasonableMovesGame, SimulableGame, SimulatorInstruments, SnakeId,
    VictorDeterminableGame,
};
use rand::{rngs::ThreadRng, thread_rng};

#[derive(Debug)]
struct Instruments {}

impl SimulatorInstruments for Instruments {
    fn observe_simulation(&self, _: std::time::Duration) {}
}

fn run_from_fixture_till_end(rng: &mut ThreadRng, instrument: Instruments) {
    let fixture_string = include_str!("../fixtures/e80b70e7-a916-40ca-82d2-ad76e074efe1_0.json");
    let wire =
        serde_json::from_str::<battlesnake_game_types::wire_representation::Game>(fixture_string)
            .unwrap();
    let id_map = battlesnake_game_types::types::build_snake_id_map(&wire);
    let initial_game = battlesnake_game_types::compact_representation::StandardCellBoard4Snakes11x11::convert_from_game(wire, &id_map).unwrap();

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

        dbg!(game.get_health(&SnakeId(0)));
    }
}
fn main() {
    let instrument = Instruments {};
    let mut rng = thread_rng();

    run_from_fixture_till_end(&mut rng, instrument);

    println!("Hello, world!");
}
