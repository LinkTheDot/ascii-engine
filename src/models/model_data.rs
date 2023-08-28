// pub(crate) fn get_hitbox_world_position_from(&self, new_position: usize) -> (isize, isize) {
//   let (x, y) = new_position.index_to_coordinates(CONFIG.grid_width as usize + 1);
//   let skin_anchor_to_hitbox_anchor = self.get_skin_anchor_to_hitbox_anchor();
//
//   (
//     x as isize + skin_anchor_to_hitbox_anchor.0,
//     y as isize + skin_anchor_to_hitbox_anchor.1,
//   )
// }
//
// /// Returns the relative distance of the skin's anchor to the hitbox's anchor when their
// /// top left positions are aligned.
// pub(crate) fn get_skin_anchor_to_hitbox_anchor(&self) -> (isize, isize) {
//   let hitbox_width = self.get_hitbox_dimensions().0 as f32;
//   let hitbox_anchor_index = self.get_hitbox_anchor_index() as f32;
//   let skin_anchor_coordinates = self.get_sprite_anchor_coordinates();
//
//   let hitbox_anchor_to_top_left_x = (hitbox_anchor_index % hitbox_width).round() as isize;
//   let hitbox_anchor_to_top_left_y = (hitbox_anchor_index / hitbox_width).round() as isize;
//
//   (
//     hitbox_anchor_to_top_left_x - skin_anchor_coordinates.0,
//     hitbox_anchor_to_top_left_y - skin_anchor_coordinates.1,
//   )
// }

// #[cfg(test)]
// mod tests {
//   use super::*;
//
//   const WORLD_POSITION: (usize, usize) = (10, 10);
//   const SHAPE: &str = "x-x\nxcx\nx-x";
//   const ANCHOR_CHAR: char = 'c';
//   const ANCHOR_REPLACEMENT_CHAR: char = '-';
//   const AIR_CHAR: char = '-';
//
//   #[test]
//   fn get_frame_index_to_world_placement_anchor_logic() {
//     let sprite = test_sprite();
//
//     let expected_position = (-1, -1);
//
//     let relative_position = get_frame_index_to_world_placement_anchor(&sprite);
//
//     assert_eq!(relative_position, expected_position);
//   }
//
//   #[test]
//   fn change_sprite_appearance() {
//     let mut model_data = new_test_model();
//     let new_sprite = "sssss\nxxaxx\nxxxxx";
//
//     let expected_appearance = "sssss\nxxxxx\nxxxxx".to_string();
//     let expected_sprite_anchor_coordinates = (2, 1);
//     let expected_sprite_anchor_index = 7;
//     let expected_hitbox_relative_top_left = (0, 0);
//
//     model_data
//       .change_sprite_appearance(new_sprite, None)
//       .unwrap();
//
//     let internal_data = model_data.get_internal_data();
//
//     let new_appearance = internal_data.sprite.get_shape();
//     let sprite_anchor_index = internal_data.sprite.get_anchor_character_index();
//     let sprite_anchor_coordinates = internal_data.sprite_internal_anchor_coordinates;
//     let hitbox_relative_top_left = internal_data.hitbox.get_relative_top_left();
//
//     assert_eq!(new_appearance, expected_appearance);
//     assert_eq!(sprite_anchor_index, expected_sprite_anchor_index);
//     assert_eq!(
//       sprite_anchor_coordinates,
//       expected_sprite_anchor_coordinates
//     );
//     assert_eq!(hitbox_relative_top_left, expected_hitbox_relative_top_left);
//   }
//
//   //
//   // Functions used for tests
//   //
//
//   fn new_test_model() -> ModelData {
//     let test_model_path = std::path::Path::new("tests/models/test_square.model");
//     ModelData::from_file(test_model_path, WORLD_POSITION).unwrap()
//   }
//
//   fn test_sprite() -> Sprite {
//     Sprite::new(SHAPE, ANCHOR_CHAR, ANCHOR_REPLACEMENT_CHAR, AIR_CHAR).unwrap()
//   }
// }
