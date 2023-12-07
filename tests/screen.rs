#![cfg(test)]

use ascii_engine::prelude::*;
use model_data_structures::models::testing_data::TestingData;

const WORLD_POSITION: (usize, usize) = (10, 10);

#[cfg(test)]
mod display_logic {
  use super::*;

  #[test]
  fn empty_screen() {
    let screen = ScreenData::default();
    // adding the height - 1 is accounting for new lines
    let expected_pixel_count =
      ((CONFIG.grid_width * CONFIG.grid_height) + CONFIG.grid_height - 1) as usize;
    let display = screen.display();

    assert_eq!(display.chars().count(), expected_pixel_count);
  }

  #[test]
  fn with_model() {
    let mut screen = ScreenData::new();
    let test_model = TestingData::new_test_model(WORLD_POSITION);

    let expected_pixel_count =
      ((CONFIG.grid_width * CONFIG.grid_height) + CONFIG.grid_height - 1) as usize;

    screen.add_model(test_model).unwrap();

    let display = screen.display();

    assert_eq!(display.chars().count(), expected_pixel_count);
  }

  #[test]
  fn get_screen_printer_logic() {
    let screen = ScreenData::new();
    let _screen_printer = screen.get_screen_printer();
  }
}

#[test]
fn add_and_remove_model() {
  let mut screen = ScreenData::new();
  let test_model = TestingData::new_test_model(WORLD_POSITION);
  let test_model_hash = test_model.get_hash();

  screen.add_model(test_model).unwrap();

  let result_data = screen.remove_model(&test_model_hash).unwrap();

  assert_eq!(result_data.get_hash(), test_model_hash);
}

#[cfg(test)]
mod world_management_logic {
  use std::path::PathBuf;

  use ascii_engine::screen::stored_worlds::StoredWorld;

  use super::*;

  #[test]
  fn reset_world_logic() {
    let test_world = StoredWorld::load(PathBuf::from("tests/worlds/test_template.world")).unwrap();
    let mut screen_data = ScreenData::from_world(test_world);
    let model_manager = screen_data.get_model_manager();

    let reset_world = screen_data.reset_world();

    assert_eq!(reset_world.model_count(), 5);
    model_manager.get_model_list(|list| {
      assert_eq!(list.iter().count(), 0);
    });
  }

  #[test]
  fn load_world_logic() {
    let test_world = StoredWorld::load(PathBuf::from("tests/worlds/test_template.world")).unwrap();
    let mut screen_data = ScreenData::new();
    let model_manager = screen_data.get_model_manager();

    let empty_world = screen_data.load_world(test_world);
    assert!(empty_world.model_count() == 0);

    model_manager.get_model_list(|model_list| {
      assert!(model_list.keys().count() == 5);
    });
  }
}
