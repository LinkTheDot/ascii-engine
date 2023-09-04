#![cfg(test)]
#![allow(clippy::items_after_test_module)] // I prefer to put testing data at the bottom of the file.

use ascii_engine::prelude::*;
use model_data_structures::models::testing_data::TestingData;

const WORLD_POSITION: (usize, usize) = (10, 10);

#[test]
fn model_list_contains_models() {
  let model_one = TestingData::new_test_model(WORLD_POSITION);
  let model_two = TestingData::new_test_model(WORLD_POSITION);
  let (_, model_manager) = setup_model_manager(vec![model_one.clone(), model_two.clone()]);

  model_manager.get_model_list(|model_list| {
    assert!(model_list.contains_key(&model_one.get_hash()));
    assert!(model_list.contains_key(&model_two.get_hash()));
  })
}

#[cfg(test)]
mod get_model_logic {
  use super::*;

  #[test]
  fn model_exists() {
    let model = TestingData::new_test_model(WORLD_POSITION);
    let (_, model_manager) = setup_model_manager(vec![model.clone()]);

    assert!(model_manager.get_model(&model.get_hash()).is_some());
  }

  #[test]
  fn model_does_not_exists() {
    let (_, model_manager) = setup_model_manager(vec![]);

    assert!(model_manager.get_model(&0).is_none());
  }
}

#[cfg(test)]
mod model_exists_logic {
  use super::*;

  #[test]
  fn model_exists() {
    let model = TestingData::new_test_model(WORLD_POSITION);
    let (_, model_manager) = setup_model_manager(vec![model.clone()]);

    assert!(model_manager.model_exists(&model.get_hash()));
  }

  #[test]
  fn model_does_not_exists() {
    let (_, model_manager) = setup_model_manager(vec![]);

    assert!(!model_manager.model_exists(&0));
  }
}

#[cfg(test)]
mod model_movement_and_collision_logic {
  use engine_math::prelude::*;

  use super::*;

  #[test]
  fn absolute_movement_logic() {
    let model = TestingData::new_test_model(WORLD_POSITION);
    let (_, mut model_manager) = setup_model_manager(vec![model.clone()]);
    let new_position = (11, 10);
    let movement = ModelMovement::Absolute(new_position.to_isize());
    let previous_model_position = model.get_frame_position();

    let expected_position = model.calculate_top_left_index_from(new_position).unwrap();

    let collisions = model_manager
      .move_model(model.get_hash(), movement)
      .unwrap();
    let new_model_position = model.get_frame_position();

    assert!(collisions.is_none());
    assert_eq!(
      new_model_position, expected_position,
      "{} -> {} != {}",
      previous_model_position, new_model_position, expected_position
    );
  }
}

// data for tests

fn setup_model_manager(models_to_add: Vec<ModelData>) -> (ScreenData, ModelManager) {
  let mut screen_data = ScreenData::new();
  let model_manager = screen_data.get_model_manager();

  for model in models_to_add {
    screen_data
      .add_model(model)
      .expect("Added duplicate model to world.");
  }

  (screen_data, model_manager)
}
