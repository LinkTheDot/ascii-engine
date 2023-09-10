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
  use std::collections::VecDeque;

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
      .move_model(&model.get_hash(), movement)
      .unwrap();
    let new_model_position = model.get_frame_position();

    assert!(collisions.is_none());
    assert_eq!(
      new_model_position, expected_position,
      "{} -> {} != {}",
      previous_model_position, new_model_position, expected_position
    );
  }

  #[test]
  fn relative_movement_logic() {
    let model = TestingData::new_test_model(WORLD_POSITION);
    let (_, mut model_manager) = setup_model_manager(vec![model.clone()]);
    let new_position = (11, 10);
    let movement = ModelMovement::Relative((1, 0));
    let previous_model_position = model.get_frame_position();

    let expected_position = model.calculate_top_left_index_from(new_position).unwrap();

    let collisions = model_manager
      .move_model(&model.get_hash(), movement)
      .unwrap();
    let new_model_position = model.get_frame_position();

    assert!(collisions.is_none());
    assert_eq!(
      new_model_position, expected_position,
      "{} -> {} != {}",
      previous_model_position, new_model_position, expected_position
    );
  }

  #[test]
  fn model_does_not_exist() {
    let model = TestingData::new_test_model(WORLD_POSITION);
    let (_, mut model_manager) = setup_model_manager(vec![model.clone()]);
    let movement = ModelMovement::Relative((0, 0));

    let expected_error = ModelError::ModelDoesntExist;

    let result = model_manager.move_model(&0, movement).unwrap_err();

    assert_eq!(result, expected_error);
  }

  #[test]
  fn model_out_of_bounds() {
    let model = TestingData::new_test_model(WORLD_POSITION);
    let (_, mut model_manager) = setup_model_manager(vec![model.clone()]);
    let new_position = (0, 0);
    let movement = ModelMovement::Absolute(new_position);

    let expected_error = ModelError::ModelOutOfBounds;

    let result = model_manager
      .move_model(&model.get_hash(), movement)
      .unwrap_err();

    assert_eq!(result, expected_error);
  }

  #[test]
  fn single_model_collision() {
    let model_one = TestingData::new_test_model(WORLD_POSITION);
    let model_two = TestingData::new_test_model(WORLD_POSITION.add((5, 0)));
    let (_, mut model_manager) = setup_model_manager(vec![model_one.clone(), model_two.clone()]);
    let movement = (1, 0);
    let movement = ModelMovement::Relative(movement);

    let expected_collisions = VecDeque::from([model_two.get_hash()]);

    let model_collisions = model_manager
      .move_model(&model_one.get_hash(), movement)
      .unwrap()
      .expect("There were no collisions detected.");

    assert_eq!(model_one.get_hash(), model_collisions.collider);
    assert_eq!(movement, model_collisions.caused_movement);
    assert_eq!(expected_collisions, model_collisions.collision_list);
  }

  #[test]
  fn multiple_model_collisions() {
    let model_one = TestingData::new_test_model(WORLD_POSITION);
    let model_two = TestingData::new_test_model(WORLD_POSITION.add((5, 0)));
    let model_three = TestingData::new_test_model(WORLD_POSITION.add((5, 0)));
    let (_, mut model_manager) = setup_model_manager(vec![
      model_one.clone(),
      model_two.clone(),
      model_three.clone(),
    ]);
    let movement = (1, 0);
    let movement = ModelMovement::Relative(movement);

    let mut expected_collisions = VecDeque::from([model_two.get_hash(), model_three.get_hash()]);

    let mut model_collisions = model_manager
      .move_model(&model_one.get_hash(), movement)
      .unwrap()
      .expect("There were no collisions detected.");

    // Ensures the order of the lists is the same every time.
    model_collisions.collision_list.make_contiguous().sort();
    expected_collisions.make_contiguous().sort();

    assert_eq!(model_one.get_hash(), model_collisions.collider);
    assert_eq!(movement, model_collisions.caused_movement);
    assert_eq!(expected_collisions, model_collisions.collision_list);
  }

  #[test]
  fn empty_hitbox_makes_no_collisions() {
    let model_one = TestingData::new_test_model(WORLD_POSITION);
    let model_two = TestingData::new_test_model_no_hitbox(WORLD_POSITION.add((5, 0)));
    let (_, mut model_manager) = setup_model_manager(vec![model_one.clone(), model_two.clone()]);
    let movement = (1, 0);
    let movement = ModelMovement::Relative(movement);

    let model_collisions = model_manager
      .move_model(&model_one.get_hash(), movement)
      .unwrap();

    assert!(model_collisions.is_none());
  }

  #[test]
  fn check_if_movement_causes_collisions_logic() {
    let model_one = TestingData::new_test_model(WORLD_POSITION);
    let model_two = TestingData::new_test_model(WORLD_POSITION.add((5, 0)));
    let (_, model_manager) = setup_model_manager(vec![model_one.clone(), model_two.clone()]);
    let movement = (1, 0);
    let movement = ModelMovement::Relative(movement);

    let expected_collisions = VecDeque::from([model_two.get_hash()]);
    let expected_model_one_position = model_one.get_world_position();

    let model_collisions = model_manager
      .check_if_movement_causes_collisions(&model_one.get_hash(), movement)
      .unwrap()
      .expect("There were no collisions detected.");

    assert_eq!(model_one.get_hash(), model_collisions.collider);
    assert_eq!(movement, model_collisions.caused_movement);
    assert_eq!(expected_collisions, model_collisions.collision_list);
    assert_eq!(model_one.get_world_position(), expected_model_one_position); // Ensure there was no movement.
  }

  #[test]
  fn check_if_movement_causes_collisions_no_collisions() {
    let model = TestingData::new_test_model(WORLD_POSITION);
    let (_, model_manager) = setup_model_manager(vec![model.clone()]);
    let movement = (1, 0);
    let movement = ModelMovement::Relative(movement);

    let result = model_manager
      .check_if_movement_causes_collisions(&model.get_hash(), movement)
      .unwrap();

    assert!(result.is_none());
  }
}

#[test]
fn add_animation_connection_through_screen() {
  let mut screen = ScreenData::new();
  let mut model_manager = screen.get_model_manager();

  screen.start_animation_thread().unwrap();
  screen.connect_model_manager_to_animation_thread(&mut model_manager);

  assert!(model_manager.is_connected_to_animation_thread());
}

#[test]
fn check_if_movement_causes_collisions_model_does_not_exist() {
  let (_, model_manager) = setup_model_manager(vec![]);
  let movement = ModelMovement::Relative((0, 0));

  let expected_result = ModelError::ModelDoesntExist;

  let result = model_manager
    .check_if_movement_causes_collisions(&0, movement)
    .unwrap_err();

  assert_eq!(result, expected_result);
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
