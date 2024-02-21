#![cfg(test)]

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
  use super::*;
  use engine_math::prelude::*;
  use std::collections::VecDeque;

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
fn check_if_movement_causes_collisions_model_does_not_exist() {
  let (_, model_manager) = setup_model_manager(vec![]);
  let movement = ModelMovement::Relative((0, 0));

  let expected_result = ModelError::ModelDoesntExist;

  let result = model_manager
    .check_if_movement_causes_collisions(&0, movement)
    .unwrap_err();

  assert_eq!(result, expected_result);
}

#[cfg(test)]
mod animation_tests {
  use super::*;

  #[cfg(test)]
  mod queue_model_animation_logic {
    use super::*;

    #[test]
    fn model_doesnt_exist() {
      let (_, mut model_manager) = setup_model_manager(vec![]);

      let expected_result = ModelError::ModelDoesntExist;

      let result = model_manager
        .queue_model_animation(&0, "", false)
        .unwrap_err();

      assert_eq!(result, expected_result);
    }

    #[test]
    fn queue_animation() {
      let (mut model, _) = TestingData::new_test_model_animated(WORLD_POSITION, ['x', 'y', 'z']);
      let (_, mut model_manager) = setup_model_manager(vec![model.clone()]);

      let expected_animation_name = Some(TestingData::ANIMATION_NAME.to_string());

      model_manager
        .queue_model_animation(&model.get_hash(), TestingData::ANIMATION_NAME, false)
        .unwrap();

      let model_appearance = model.get_appearance_data();
      let model_appearance = model_appearance.lock().unwrap();

      assert_eq!(
        model_appearance.current_animation_name(),
        expected_animation_name
      );
    }

    #[test]
    fn queue_animation_no_duplicates() {
      let (mut model, _) = TestingData::new_test_model_animated(WORLD_POSITION, ['x', 'y', 'z']);
      let (_, mut model_manager) = setup_model_manager(vec![model.clone()]);

      model_manager
        .queue_model_animation(&model.get_hash(), TestingData::ANIMATION_NAME, false)
        .unwrap();
      model_manager
        .queue_model_animation(&model.get_hash(), TestingData::ANIMATION_NAME, false)
        .unwrap();

      let model_appearance = model.get_appearance_data();
      let mut model_appearance = model_appearance.lock().unwrap();

      model_appearance.remove_current_model_animation();

      assert!(model_appearance
        .animation_is_currently_queued(TestingData::ANIMATION_NAME)
        .is_none());
    }

    #[test]
    fn queue_animation_allow_duplicates() {
      let (mut model, _) = TestingData::new_test_model_animated(WORLD_POSITION, ['x', 'y', 'z']);
      let (_, mut model_manager) = setup_model_manager(vec![model.clone()]);

      let expected_animation_name = Some(TestingData::ANIMATION_NAME.to_string());

      model_manager
        .queue_model_animation(&model.get_hash(), TestingData::ANIMATION_NAME, true)
        .unwrap();
      model_manager
        .queue_model_animation(&model.get_hash(), TestingData::ANIMATION_NAME, true)
        .unwrap();

      let model_appearance = model.get_appearance_data();
      let mut model_appearance = model_appearance.lock().unwrap();

      model_appearance.remove_current_model_animation();

      assert_eq!(
        model_appearance.current_animation_name(),
        expected_animation_name
      );
    }
  }

  #[cfg(test)]
  mod overwrite_current_model_animation_logic {
    use super::*;

    #[test]
    fn model_doesnt_exist() {
      let (_, mut model_manager) = setup_model_manager(vec![]);

      let expected_result = ModelError::ModelDoesntExist;

      let result = model_manager
        .overwrite_current_model_animation(&0, "")
        .unwrap_err();

      assert_eq!(result, expected_result);
    }

    #[test]
    fn model_exists() {
      let (mut model, _) = TestingData::new_test_model_animated(WORLD_POSITION, ['x', 'y', 'z']);
      let (_, mut model_manager) = setup_model_manager(vec![model.clone()]);
      let new_animation =
        TestingData::get_test_animation(['l', 'm', 'n'], AnimationLoopCount::Limited(2));
      let new_animation_name = "new_animation".to_string();

      model_manager
        .add_animation_to_model(&model.get_hash(), new_animation_name.clone(), new_animation)
        .unwrap();

      model_manager
        .queue_model_animation(&model.get_hash(), TestingData::ANIMATION_NAME, true)
        .unwrap();

      model_manager
        .overwrite_current_model_animation(&model.get_hash(), &new_animation_name)
        .unwrap();

      let model_appearance = model.get_appearance_data();
      let model_appearance = model_appearance.lock().unwrap();

      assert_eq!(
        model_appearance.current_animation_name(),
        Some(new_animation_name)
      );
    }
  }

  #[cfg(test)]
  mod add_animation_to_model_logic {
    use super::*;

    #[test]
    fn model_doesnt_exist() {
      let animation =
        TestingData::get_test_animation(['1', '2', '3'], AnimationLoopCount::Limited(2));
      let (_, mut model_manager) = setup_model_manager(vec![]);

      let expected_result = ModelError::ModelDoesntExist;

      let result = model_manager
        .add_animation_to_model(&0, "This model does not exist.".to_string(), animation)
        .unwrap_err();

      assert_eq!(result, expected_result);
    }
  }

  #[cfg(test)]
  mod clear_model_animation_queue_logic {
    use super::*;

    #[test]
    fn model_doesnt_exist() {
      let (_, mut model_manager) = setup_model_manager(vec![]);

      let expected_result = ModelError::ModelDoesntExist;

      let result = model_manager.clear_model_animation_queue(&0).unwrap_err();

      assert_eq!(result, expected_result);
    }

    #[test]
    fn model_exists() {
      let (mut model, _) = TestingData::new_test_model_animated(WORLD_POSITION, ['x', 'y', 'z']);
      let (_, mut model_manager) = setup_model_manager(vec![model.clone()]);

      model_manager
        .queue_model_animation(&model.get_hash(), TestingData::ANIMATION_NAME, true)
        .unwrap();

      model_manager
        .clear_model_animation_queue(&model.get_hash())
        .unwrap();

      let model_appearance = model.get_appearance_data();
      let model_appearance = model_appearance.lock().unwrap();

      assert_eq!(
        model_appearance.animation_is_currently_queued(TestingData::ANIMATION_NAME),
        None
      );
    }
  }
}

#[cfg(test)]
mod model_tag_logic {
  use super::*;
  use std::collections::HashSet;

  #[test]
  fn get_models_with_tags() {
    let mut model_1 = TestingData::new_test_model(WORLD_POSITION);
    let mut model_2 = TestingData::new_test_model(WORLD_POSITION);
    let tags = vec!["Player".to_string(), "Test".to_string()];
    model_1.add_tags(vec![tags[0].clone(), tags[1].clone()]);
    model_2.add_tags(vec![tags[1].clone()]);
    let (_, model_manager) = setup_model_manager(vec![model_1.clone(), model_2.clone()]);

    let mut expected_matching_tag_models = vec![model_1.get_hash(), model_2.get_hash()];
    expected_matching_tag_models.sort();

    let mut single_tag_models = model_manager.get_models_with_tags(vec![&tags[1]]);
    single_tag_models.sort();
    let multiple_tag_models = model_manager.get_models_with_tags(vec![&tags[0], &tags[1]]);

    assert_eq!(single_tag_models, expected_matching_tag_models);
    assert_eq!(multiple_tag_models, vec![model_1.get_hash()]);
  }

  #[test]
  fn get_tags_of_model() {
    let mut model = TestingData::new_test_model(WORLD_POSITION);
    let mut tags = vec!["Player".to_string(), "Test".to_string()];
    model.add_tags(tags.clone());
    let (_, model_manager) = setup_model_manager(vec![model.clone()]);

    let expected_result: Option<HashSet<String>> =
      Some(HashSet::from([tags.remove(0), tags.remove(0)]));
    let result = model_manager.get_tags_of_model(model.get_hash());

    assert_eq!(result, expected_result);
  }
}

#[cfg(test)]
mod collision_event_logic {
  use std::collections::VecDeque;

  #[allow(unused)]
  use super::*;

  #[test]
  fn take_collision_events_does_remove_events() {
    let model_mover = TestingData::new_test_model(WORLD_POSITION);
    let model_collided = TestingData::new_test_model(WORLD_POSITION);
    let (_, mut model_manager) =
      setup_model_manager(vec![model_mover.clone(), model_collided.clone()]);
    let movement = ModelMovement::Relative((1, 0));

    let expected_collision = ModelCollisions {
      collider: model_mover.get_hash(),
      caused_movement: movement,
      collision_list: VecDeque::from(vec![model_collided.get_hash()]),
    };

    let _ = model_manager.move_model(&model_mover.get_hash(), movement);
    let collision_list: VecDeque<ModelCollisions> = model_manager
      .take_collision_events()
      .into_iter()
      .map(|(_, collisions)| collisions)
      .collect();
    let empty_list = model_manager.take_collision_events();

    assert_eq!(collision_list, VecDeque::from(vec![expected_collision]));
    assert!(empty_list.is_empty());
  }

  #[test]
  fn copy_collision_events_doesnt_remove_events() {
    let model_mover = TestingData::new_test_model(WORLD_POSITION);
    let model_collided = TestingData::new_test_model(WORLD_POSITION);
    let (_, mut model_manager) =
      setup_model_manager(vec![model_mover.clone(), model_collided.clone()]);
    let movement = ModelMovement::Relative((1, 0));

    let expected_collision = ModelCollisions {
      collider: model_mover.get_hash(),
      caused_movement: movement,
      collision_list: VecDeque::from(vec![model_collided.get_hash()]),
    };

    let _ = model_manager.move_model(&model_mover.get_hash(), movement);
    let collision_list: VecDeque<ModelCollisions> = model_manager
      .clone_collision_events()
      .into_iter()
      .map(|(_, collisions)| collisions)
      .collect();
    let collision_list_second: VecDeque<ModelCollisions> = model_manager
      .clone_collision_events()
      .into_iter()
      .map(|(_, collisions)| collisions)
      .collect();

    assert_eq!(
      collision_list,
      VecDeque::from(vec![expected_collision.clone()])
    );
    assert_eq!(
      collision_list_second,
      VecDeque::from(vec![expected_collision])
    );
  }

  #[test]
  fn collisions_are_being_tracked_properly() {
    let model_mover = TestingData::new_test_model(WORLD_POSITION);
    let model_collided = TestingData::new_test_model(WORLD_POSITION);
    let (_, mut model_manager) =
      setup_model_manager(vec![model_mover.clone(), model_collided.clone()]);
    let movement = ModelMovement::Relative((1, 0));

    let expected_collision = ModelCollisions {
      collider: model_mover.get_hash(),
      caused_movement: movement,
      collision_list: VecDeque::from(vec![model_collided.get_hash()]),
    };
    let expected_collision_list =
      VecDeque::from(vec![expected_collision.clone(), expected_collision.clone()]);

    let _ = model_manager.move_model(&model_mover.get_hash(), movement);
    let _ = model_manager.move_model(&model_mover.get_hash(), movement);

    let collision_list: VecDeque<ModelCollisions> = model_manager
      .take_collision_events()
      .into_iter()
      .map(|(_, collisions)| collisions)
      .collect();

    assert_eq!(collision_list, expected_collision_list);
  }

  #[test]
  fn model_has_collided_logic() {
    let model_mover = TestingData::new_test_model(WORLD_POSITION);
    let model_collided = TestingData::new_test_model(WORLD_POSITION);
    let (_, mut model_manager) =
      setup_model_manager(vec![model_mover.clone(), model_collided.clone()]);
    let movement = ModelMovement::Relative((1, 0));

    let expected_collision = ModelCollisions {
      collider: model_mover.get_hash(),
      caused_movement: movement,
      collision_list: VecDeque::from(vec![model_collided.get_hash()]),
    };
    let expected_collision_list =
      VecDeque::from(vec![expected_collision.clone(), expected_collision.clone()]);

    let _ = model_manager.move_model(&model_mover.get_hash(), movement);
    let _ = model_manager.move_model(&model_mover.get_hash(), movement);

    let collidee_collision_list: VecDeque<ModelCollisions> = model_manager
      .model_has_collided(&model_collided.get_hash())
      .unwrap()
      .into_iter()
      .map(|(_, collisions)| collisions)
      .collect();
    let collider_collision_list: VecDeque<ModelCollisions> = model_manager
      .model_has_collided(&model_mover.get_hash())
      .unwrap()
      .into_iter()
      .map(|(_, collisions)| collisions)
      .collect();

    assert_eq!(collidee_collision_list, expected_collision_list);
    assert_eq!(collider_collision_list, expected_collision_list);
  }

  #[test]
  fn timestamps_are_accurate() {
    let model_mover = TestingData::new_test_model(WORLD_POSITION);
    let model_collided = TestingData::new_test_model(WORLD_POSITION);
    let (_, mut model_manager) =
      setup_model_manager(vec![model_mover.clone(), model_collided.clone()]);
    let movement = ModelMovement::Relative((1, 0));

    let expected_time_gap = std::time::Duration::from_millis(20);

    let _ = model_manager.move_model(&model_mover.get_hash(), movement);
    std::thread::sleep(expected_time_gap);
    let _ = model_manager.move_model(&model_mover.get_hash(), movement);

    let mut collisions = model_manager.take_collision_events();
    let first_timestamp = collisions.pop_front().unwrap().0;
    let second_timestamp = collisions.pop_front().unwrap().0;

    assert!(second_timestamp.duration_since(first_timestamp) >= expected_time_gap);
  }

  #[test]
  fn event_list_is_shared_between_managers() {
    let model_mover = TestingData::new_test_model(WORLD_POSITION);
    let model_collided = TestingData::new_test_model(WORLD_POSITION);
    let (screen, mut model_manager) =
      setup_model_manager(vec![model_mover.clone(), model_collided.clone()]);
    let mut second_model_manager = screen.get_model_manager();
    let movement = ModelMovement::Relative((1, 0));

    let expected_collision = ModelCollisions {
      collider: model_mover.get_hash(),
      caused_movement: movement,
      collision_list: VecDeque::from(vec![model_collided.get_hash()]),
    };
    let expected_collision_list =
      VecDeque::from(vec![expected_collision.clone(), expected_collision.clone()]);

    let _ = model_manager.move_model(&model_mover.get_hash(), movement);
    let _ = model_manager.move_model(&model_mover.get_hash(), movement);

    let collision_list: VecDeque<ModelCollisions> = model_manager
      .clone_collision_events()
      .into_iter()
      .map(|(_, collisions)| collisions)
      .collect();
    let second_collision_list: VecDeque<ModelCollisions> = second_model_manager
      .take_collision_events()
      .into_iter()
      .map(|(_, collisions)| collisions)
      .collect();

    assert_eq!(collision_list, expected_collision_list);
    assert_eq!(second_collision_list, expected_collision_list);
    assert!(model_manager.take_collision_events().is_empty());
    assert!(second_model_manager.take_collision_events().is_empty());
  }
}

//
// data for tests
//

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
