use ascii_engine::data_types::*;
use ascii_engine::screen::screen_data::*;
use std::collections::HashMap;

pub const OBJECT_1_NAME: &str = "O1";
pub const OBJECT_1_DISPLAY: &str = "a";

pub const OBJECT_2_NAME: &str = "O2";
pub const OBJECT_2_DISPLAY: &str = "b";

pub const OBJECT_3_NAME: &str = "O3";
pub const OBJECT_3_DISPLAY: &str = "c";

#[test]
fn display_logic() {
  let screen = ScreenData::default();
  let expected_pixel_count = (GRID_WIDTH * GRID_HEIGHT) + GRID_HEIGHT;
  let display = screen.display();

  assert_eq!(display.len(), expected_pixel_count);
}

pub fn get_object_list(number_assignment: u32) -> [KeyAndObjectDisplay; 3] {
  [
    (
      OBJECT_1_NAME.to_string(),
      (number_assignment, OBJECT_1_DISPLAY.to_string()),
    ),
    (
      OBJECT_2_NAME.to_string(),
      (number_assignment, OBJECT_2_DISPLAY.to_string()),
    ),
    (
      OBJECT_3_NAME.to_string(),
      (number_assignment, OBJECT_3_DISPLAY.to_string()),
    ),
  ]
}

pub fn convert_object_for_assertion(object: KeyAndObjectDisplay) -> (Key, AssignedObjects) {
  let mut assigned_object = HashMap::new();
  assigned_object.insert(object.1 .0, object.1 .1);

  (object.0, assigned_object)
}
