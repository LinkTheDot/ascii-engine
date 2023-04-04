#![allow(unused)]

use ascii_engine::prelude::*;

const WORLD_POSITION: (usize, usize) = (10, 10);
const SHAPE: &str = "xxxxx\nxxaxx\nxxxxx";
const ANCHOR_CHAR: char = 'a';
const ANCHOR_REPLACEMENT_CHAR: char = 'x';
const AIR_CHAR: char = '-';
const MODEL_NAME: &str = "Test_Model";

#[test]
fn display_logic() {
  let screen = ScreenData::default();
  // adding the height - 1 is accounting for new lines
  let expected_pixel_count =
    ((CONFIG.grid_width * CONFIG.grid_height) + CONFIG.grid_height - 1) as usize;
  let display = screen.display();

  assert_eq!(display.chars().count(), expected_pixel_count);
}

//
// -- Data for tests below --
//

#[derive(DisplayModel)]
struct TestModel {
  model_data: ModelData,
}

impl TestModel {
  fn new() -> Self {
    let test_model_path = std::path::Path::new("tests/models/test_square.model");
    let model_data = ModelData::from_file(test_model_path, WORLD_POSITION).unwrap();

    Self { model_data }
  }

  fn get_sprite() -> Sprite {
    Sprite::new(SHAPE, ANCHOR_CHAR, ANCHOR_REPLACEMENT_CHAR, AIR_CHAR).unwrap()
  }
}
