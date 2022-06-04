use crate::{
    compact_representation::{core::dimensions::Dimensions, CellNum},
    types::{HeadGettableGame, SnakeBodyGettableGame},
};

use super::CellBoard;

impl<T: CellNum, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
    SnakeBodyGettableGame for CellBoard<T, D, BOARD_SIZE, MAX_SNAKES>
{
    fn get_snake_body_vec(&self, snake_id: &Self::SnakeIDType) -> Vec<Self::NativePositionType> {
        let mut body = vec![];
        body.reserve(self.get_length(*snake_id).into());
        let head = self.get_head_as_native_position(snake_id);

        let mut cur = Some(self.get_cell(head).get_tail_position(head).unwrap());

        while let Some(c) = cur {
            body.push(c);
            if self.get_cell(c).is_double_stacked_piece() {
                body.push(c);
            }
            if self.get_cell(c).is_triple_stacked_piece() {
                body.push(c);
                body.push(c);
            }
            cur = self.get_cell(c).get_next_index();
        }

        body.reverse();

        body
    }

    fn get_snake_body_iter<'s>(
        &'s self,
        snake_id: &Self::SnakeIDType,
    ) -> Box<dyn Iterator<Item = Self::NativePositionType> + 's> {
        let head = self.get_head_as_native_position(snake_id);

        let mut cur = Some(self.get_cell(head).get_tail_position(head).unwrap());

        Box::new(std::iter::from_fn(move || {
            if let Some(c) = cur {
                let to_return = c;
                cur = self.get_cell(c).get_next_index();

                Some(to_return)
            } else {
                None
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        compact_representation::{core::dimensions::Custom, CellIndex},
        types::{build_snake_id_map, SnakeId},
        wire_representation::Game,
    };

    use super::*;

    type CellBoard4Snakes11x11 = CellBoard<u8, Custom, { 11 * 11 }, 4>;

    #[test]
    fn test_body_stacking() {
        let game_fixture = include_str!("../../../../fixtures/start_of_game.json");
        let g: Result<Game, _> = serde_json::from_slice(game_fixture.as_bytes());
        let g = g.expect("the json literal is valid");
        let snake_id_mapping = build_snake_id_map(&g);
        let compact: CellBoard4Snakes11x11 =
            CellBoard4Snakes11x11::convert_from_game(g, &snake_id_mapping).unwrap();

        let full_body = compact.get_snake_body_vec(&SnakeId(0));

        assert_eq!(
            full_body,
            vec![
                CellIndex(8 * 11 + 5),
                CellIndex(9 * 11 + 5),
                CellIndex(9 * 11 + 5)
            ]
        );

        let mut body_iter = compact.get_snake_body_iter(&SnakeId(0));

        assert_eq!(body_iter.next(), Some(CellIndex(9 * 11 + 5)));
        assert_eq!(body_iter.next(), Some(CellIndex(8 * 11 + 5)));
        assert_eq!(body_iter.next(), None);
    }

    #[test]
    fn test_body_vec_and_iter_match_no_stacking() {
        let game_fixture = include_str!("../../../../fixtures/late_stage.json");
        let g: Result<Game, _> = serde_json::from_slice(game_fixture.as_bytes());
        let g = g.expect("the json literal is valid");
        let snake_id_mapping = build_snake_id_map(&g);
        let compact: CellBoard4Snakes11x11 =
            CellBoard4Snakes11x11::convert_from_game(g, &snake_id_mapping).unwrap();

        for (_, sid) in snake_id_mapping.into_iter() {
            let mut from_vec = compact.get_snake_body_vec(&sid);
            let mut from_iter = compact.get_snake_body_iter(&sid).collect::<Vec<_>>();

            from_vec.sort();
            from_iter.sort();

            assert_eq!(from_vec, from_iter);
        }
    }
}
