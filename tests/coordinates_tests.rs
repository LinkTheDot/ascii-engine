use interactable_screen::general_data::coordinates::*;

#[test]
fn get_coordinates_in_between_logic() {
  let top_left = (0, 0);
  let bottom_right = (9, 9);
  let expected_amount_of_coordinates = 100;

  let in_between = &top_left.get_coordinates_in_between(&bottom_right);

  assert_eq!(in_between.len(), expected_amount_of_coordinates);
}
