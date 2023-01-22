/// Very internal, implements common board traits for any board type that embeds a cellboard
#[macro_export]
macro_rules! impl_common_board_traits {
    ($type:tt) => {
        impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
            LengthGettableGame for $type<T, D, BOARD_SIZE, MAX_SNAKES>
        {
            type LengthType = u16;

            fn get_length(&self, snake_id: &Self::SnakeIDType) -> Self::LengthType {
                self.embedded.get_length(*snake_id)
            }

            fn get_length_i64(&self, snake_id: &Self::SnakeIDType) -> i64 {
                self.embedded.get_length_i64(snake_id)
            }
        }

        impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
            HealthGettableGame for $type<T, D, BOARD_SIZE, MAX_SNAKES>
        {
            type HealthType = u8;
            const ZERO: Self::HealthType = 0;

            fn get_health(&self, snake_id: &Self::SnakeIDType) -> Self::HealthType {
                self.embedded.get_health(snake_id)
            }

            fn get_health_i64(&self, snake_id: &Self::SnakeIDType) -> i64 {
                self.embedded.get_health_i64(snake_id) as i64
            }
        }

        impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
            VictorDeterminableGame for $type<T, D, BOARD_SIZE, MAX_SNAKES>
        {
            fn is_over(&self) -> bool {
                self.embedded.is_over()
            }

            fn get_winner(&self) -> Option<Self::SnakeIDType> {
                self.embedded.get_winner()
            }

            fn alive_snake_count(&self) -> usize {
                self.embedded.alive_snake_count()
            }
        }

        impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
            YouDeterminableGame for $type<T, D, BOARD_SIZE, MAX_SNAKES>
        {
            fn is_you(&self, snake_id: &Self::SnakeIDType) -> bool {
                self.embedded.is_you(snake_id)
            }

            fn you_id(&self) -> &Self::SnakeIDType {
                self.embedded.you_id()
            }
        }

        impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
            FoodGettableGame for $type<T, D, BOARD_SIZE, MAX_SNAKES>
        {
            fn get_all_food_as_positions(&self) -> Vec<$crate::wire_representation::Position> {
                self.embedded.get_all_food_as_positions()
            }

            fn get_all_food_as_native_positions(&self) -> Vec<Self::NativePositionType> {
                self.embedded.get_all_food_as_native_positions()
            }
        }

        impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
            HeadGettableGame for $type<T, D, BOARD_SIZE, MAX_SNAKES>
        {
            fn get_head_as_position(
                &self,
                snake_id: &Self::SnakeIDType,
            ) -> $crate::wire_representation::Position {
                self.embedded.get_head_as_position(snake_id)
            }

            fn get_head_as_native_position(
                &self,
                snake_id: &Self::SnakeIDType,
            ) -> Self::NativePositionType {
                self.embedded.get_head_as_native_position(snake_id)
            }
        }

        impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
            HazardSettableGame for $type<T, D, BOARD_SIZE, MAX_SNAKES>
        {
            fn set_hazard(&mut self, pos: Self::NativePositionType) {
                self.embedded.set_hazard(pos)
            }

            fn clear_hazard(&mut self, pos: Self::NativePositionType) {
                self.embedded.clear_hazard(pos)
            }
        }

        impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
            HazardQueryableGame for $type<T, D, BOARD_SIZE, MAX_SNAKES>
        {
            fn is_hazard(&self, pos: &Self::NativePositionType) -> bool {
                self.embedded.is_hazard(pos)
            }

            fn get_hazard_damage(&self) -> u8 {
                self.embedded.get_hazard_damage()
            }
        }

        impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
            FoodQueryableGame for $type<T, D, BOARD_SIZE, MAX_SNAKES>
        {
            fn is_food(&self, pos: &Self::NativePositionType) -> bool {
                self.embedded.cell_is_food(*pos)
            }
        }

        impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
            NeckQueryableGame for $type<T, D, BOARD_SIZE, MAX_SNAKES>
        {
            fn is_neck(&self, sid: &Self::SnakeIDType, pos: &Self::NativePositionType) -> bool {
                self.embedded.is_neck(sid, pos)
            }
        }

        impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
            SnakeBodyGettableGame for $type<T, D, BOARD_SIZE, MAX_SNAKES>
        {
            fn get_snake_body_vec(
                &self,
                snake_id: &Self::SnakeIDType,
            ) -> Vec<Self::NativePositionType> {
                self.embedded.get_snake_body_vec(snake_id)
            }

            fn get_snake_body_iter<'s>(
                &'s self,
                snake_id: &Self::SnakeIDType,
            ) -> Box<dyn Iterator<Item = Self::NativePositionType> + 's> {
                self.embedded.get_snake_body_iter(snake_id)
            }
        }

        impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
            SizeDeterminableGame for $type<T, D, BOARD_SIZE, MAX_SNAKES>
        {
            fn get_width(&self) -> u32 {
                self.embedded.get_width()
            }

            fn get_height(&self) -> u32 {
                self.embedded.get_height()
            }
        }

        impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize> Display
            for $type<T, D, BOARD_SIZE, MAX_SNAKES>
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.embedded.fmt(f)
            }
        }
        impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
            SnakeIDGettableGame for $type<T, D, BOARD_SIZE, MAX_SNAKES>
        {
            type SnakeIDType = SnakeId;

            fn get_snake_ids(&self) -> Vec<Self::SnakeIDType> {
                self.embedded.get_snake_ids()
            }
        }

        impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
            PositionGettableGame for $type<T, D, BOARD_SIZE, MAX_SNAKES>
        {
            type NativePositionType = CellIndex<T>;

            fn position_is_snake_body(&self, pos: Self::NativePositionType) -> bool {
                self.embedded.position_is_snake_body(pos)
            }

            fn position_from_native(&self, pos: Self::NativePositionType) -> Position {
                self.embedded.position_from_native(pos)
            }

            fn native_from_position(&self, pos: Position) -> Self::NativePositionType {
                self.embedded.native_from_position(pos)
            }

            fn off_board(&self, pos: Position) -> bool {
                self.embedded.off_board(pos)
            }
        }

        impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
            std::convert::TryFrom<Game> for $type<T, D, BOARD_SIZE, MAX_SNAKES>
        {
            type Error = Box<dyn Error>;

            fn try_from(game: Game) -> Result<Self, Box<dyn Error>> {
                let id_map = $crate::types::build_snake_id_map(&game);

                $type::convert_from_game(game, &id_map)
            }
        }

        impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
            MaxSnakes<MAX_SNAKES> for $type<T, D, BOARD_SIZE, MAX_SNAKES>
        {
        }

        impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
            EmptyCellGettableGame for $type<T, D, BOARD_SIZE, MAX_SNAKES>
        {
            fn get_empty_cells(&self) -> Box<dyn Iterator<Item = Self::NativePositionType> + '_> {
                self.embedded.get_empty_cells()
            }
        }

        impl<T: CN, D: Dimensions, const BOARD_SIZE: usize, const MAX_SNAKES: usize>
            StandardFoodPlaceableGame for $type<T, D, BOARD_SIZE, MAX_SNAKES>
        {
            fn place_food(&mut self, rng: &mut impl rand::Rng) {
                self.embedded.place_food(rng)
            }
        }
    };
}
