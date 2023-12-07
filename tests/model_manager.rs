#![cfg(test)]
// #![allow(clippy::items_after_test_module)] // I prefer to put testing data at the bottom of the file.

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
    #[ignore]
    fn animation_doesnt_exist() {
      // let model = TestingData::new_test_model_with_animation((10, 10), vec![]);
      // let (_, mut model_manager) = setup_model_manager(vec![model.clone()]);
      //
      // let expected_result = ModelError::AnimationError(AnimationError::AnimationDoesntExist);
      //
      // let result = model_manager
      //   .queue_model_animation(&model.get_hash(), "")
      //   .unwrap_err();
      //
      // assert_eq!(result, expected_result);
    }

    #[test]
    #[ignore]
    fn model_isnt_animated() {
      // let model = TestingData::new_test_model((10, 10));
      // let (_, mut model_manager) = setup_model_manager(vec![model.clone()]);
      //
      // let expected_result = ModelError::AnimationError(AnimationError::ModelHasNoAnimationData);
      //
      // let result = model_manager
      //   .queue_model_animation(&model.get_hash(), "")
      //   .unwrap_err();
      //
      // assert_eq!(result, expected_result);
    }

    #[test]
    #[ignore]
    fn expected_result() {
      // let animation_name = TestingData::ANIMATION_NAME;
      // let (mut model, test_animation) =
      //   TestingData::new_test_model_animated((10, 10), ['1', '2', '3']);
      // let (_, mut model_manager) = setup_model_manager(vec![model.clone()]);
      //
      // let expected_first_frame = test_animation.get_frames().get(0).cloned();
      //
      // model_manager
      //   .queue_model_animation(&model.get_hash(), animation_name)
      //   .unwrap();
      //
      // let model_animation_data = model.get_animation_data().unwrap();
      // let mut model_animation_data = model_animation_data.lock().unwrap();
      // let mut model_animator = model_animation_data.get_model_animator();
      //
      // let first_frame = model_animator.next_frame();
      //
      // assert_eq!(first_frame, expected_first_frame);
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
    #[ignore]
    fn animation_doesnt_exist() {
      // let model = TestingData::new_test_model_with_animation((10, 10), vec![]);
      // let (_, mut model_manager) = setup_model_manager(vec![model.clone()]);
      //
      // let expected_result = ModelError::AnimationError(AnimationError::AnimationDoesntExist);
      //
      // let result = model_manager
      //   .overwrite_current_model_animation(&model.get_hash(), "")
      //   .unwrap_err();
      //
      // assert_eq!(result, expected_result);
    }

    #[test]
    #[ignore]
    fn model_isnt_animated() {
      // let model = TestingData::new_test_model((10, 10));
      // let (_, mut model_manager) = setup_model_manager(vec![model.clone()]);
      //
      // let expected_result = ModelError::AnimationError(AnimationError::ModelHasNoAnimationData);
      //
      // let result = model_manager
      //   .overwrite_current_model_animation(&model.get_hash(), "")
      //   .unwrap_err();
      //
      // assert_eq!(result, expected_result);
    }

    #[test]
    #[ignore]
    fn expected_result() {
      // let animation_one =
      //   TestingData::get_test_animation(['o', 'n', 'e'], AnimationLoopCount::Limited(2));
      // let animation_two =
      //   TestingData::get_test_animation(['t', 'w', 'o'], AnimationLoopCount::Limited(2));
      // let mut model = TestingData::new_test_model_with_animation(
      //   (10, 10),
      //   vec![
      //     ("test_one".to_string(), animation_one),
      //     ("test_two".to_string(), animation_two.clone()),
      //   ],
      // );
      // let (_, mut model_manager) = setup_model_manager(vec![model.clone()]);
      //
      // let expected_first_frame = animation_two.get_frames().get(0).cloned();
      //
      // model_manager
      //   .queue_model_animation(&model.get_hash(), "test_one")
      //   .unwrap();
      // model_manager
      //   .overwrite_current_model_animation(&model.get_hash(), "test_two")
      //   .unwrap();
      //
      // let model_animation_data = model.get_animation_data().unwrap();
      // let mut model_animation_data = model_animation_data.lock().unwrap();
      // let mut model_animator = model_animation_data.get_model_animator();
      //
      // let first_frame = model_animator.next_frame();
      //
      // assert_eq!(first_frame, expected_first_frame);
    }
  }

  #[cfg(test)]
  mod add_animation_to_model_logic {
    // use super::*;
    // use std::collections::HashMap;

    #[test]
    #[ignore]
    fn model_doesnt_exist() {
      // let animation =
      //   TestingData::get_test_animation(['1', '2', '3'], AnimationLoopCount::Limited(2));
      // let (_, mut model_manager) = setup_model_manager(vec![]);
      //
      // let expected_result = ModelError::ModelDoesntExist;
      //
      // let result = model_manager
      //   .add_animation_to_model(&0, animation, "This model does not exist.".to_string())
      //   .unwrap_err();
      //
      // assert_eq!(result, expected_result);
    }

    #[test]
    #[ignore]
    fn model_isnt_animated() {
      // let animation =
      //   TestingData::get_test_animation(['1', '2', '3'], AnimationLoopCount::Limited(2));
      // let model = TestingData::new_test_model((10, 10));
      // let (_, mut model_manager) = setup_model_manager(vec![model.clone()]);
      //
      // let expected_result = ModelError::AnimationError(AnimationError::ModelHasNoAnimationData);
      //
      // let result = model_manager
      //   .add_animation_to_model(
      //     &model.get_hash(),
      //     animation,
      //     "This model isn't event animated.".to_string(),
      //   )
      //   .unwrap_err();
      //
      // assert_eq!(result, expected_result);
    }

    #[test]
    #[ignore]
    fn expected_result() {
      // let animation_name = "test".to_string();
      // let test_animation =
      //   TestingData::get_test_animation(['o', 'n', 'e'], AnimationLoopCount::Limited(2));
      // let mut model = TestingData::new_test_model_with_animation((10, 10), vec![]);
      // let (_, mut model_manager) = setup_model_manager(vec![model.clone()]);
      //
      // let expected_animations = HashMap::from([(animation_name.clone(), test_animation.clone())]);
      //
      // model_manager
      //   .add_animation_to_model(&model.get_hash(), test_animation, animation_name)
      //   .unwrap();
      //
      // let model_animation_data = model.get_animation_data().unwrap();
      // let model_animation_data = model_animation_data.lock().unwrap();
      //
      // assert_eq!(
      //   model_animation_data.get_animation_list(),
      //   &expected_animations
      // );
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
    #[ignore]
    fn model_isnt_animated() {
      // let model = TestingData::new_test_model((10, 10));
      // let (_, mut model_manager) = setup_model_manager(vec![model.clone()]);
      //
      // let expected_result = ModelError::AnimationError(AnimationError::ModelHasNoAnimationData);
      //
      // let result = model_manager
      //   .clear_model_animation_queue(&model.get_hash())
      //   .unwrap_err();
      //
      // assert_eq!(result, expected_result);
    }

    #[test]
    #[ignore]
    fn expected_result() {
      // let animation_name = TestingData::ANIMATION_NAME;
      // let (mut model, _) = TestingData::new_test_model_animated((10, 10), ['1', '2', '3']);
      // let (_, mut model_manager) = setup_model_manager(vec![model.clone()]);
      //
      // // Queue the animation twice to have an instance running and another in queue.
      // for _ in 0..2 {
      //   model_manager
      //     .queue_model_animation(&model.get_hash(), animation_name)
      //     .unwrap();
      // }
      //
      // model_manager
      //   .clear_model_animation_queue(&model.get_hash())
      //   .unwrap();
      //
      // let model_animation_data = model.get_animation_data().unwrap();
      // let mut model_animation_data = model_animation_data.lock().unwrap();
      // let model_animator = model_animation_data.get_model_animator();
      //
      // assert!(
      //   !model_animator.has_animations_queued(),
      //   "{:#?}",
      //   model_animator
      // );
    }
  }
}

#[test]
fn model_tag_logic() {
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
