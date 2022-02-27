use crate::{
    compact_representation::CellNum,
    types::{HeadGettableGame, NeckQueryableGame},
};

use super::CellBoard;

impl<T: CellNum, const BOARD_SIZE: usize, const MAX_SNAKES: usize> NeckQueryableGame
    for CellBoard<T, BOARD_SIZE, MAX_SNAKES>
{
    fn is_neck(&self, sid: &Self::SnakeIDType, pos: &Self::NativePositionType) -> bool {
        let potential_neck = self.get_cell(*pos);

        potential_neck.get_snake_id() == Some(*sid)
            && potential_neck.get_next_index() == Some(self.get_head_as_native_position(sid))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        compact_representation::CellIndex,
        types::{build_snake_id_map, SnakeId},
        wire_representation::Game,
    };

    use super::*;

    type CellBoard4Snakes11x11 = CellBoard<u8, { 11 * 11 }, 4>;

    #[test]
    fn test_is_neck() {
        let game_fixture = include_str!("../../../../fixtures/start_of_game.json");
        let g: Result<Game, _> = serde_json::from_slice(game_fixture.as_bytes());
        let g = g.expect("the json literal is valid");
        let snake_id_mapping = build_snake_id_map(&g);
        let compact: CellBoard4Snakes11x11 =
            CellBoard4Snakes11x11::convert_from_game(g, &snake_id_mapping).unwrap();

        assert!(compact.is_neck(&SnakeId(0), &CellIndex(9 * 11 + 5)));
        assert!(!compact.is_neck(&SnakeId(0), &CellIndex(8 * 11 + 5)));
        assert!(!compact.is_neck(&SnakeId(0), &CellIndex(9 * 11 + 6)));

        assert!(compact.is_neck(&SnakeId(1), &CellIndex(5 * 11 + 1)));
    }
}