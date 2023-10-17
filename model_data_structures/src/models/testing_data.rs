// #![cfg(test)]

use crate::prelude::*;
use lazy_static::lazy_static;
use std::path::PathBuf;

lazy_static! {
  static ref TEST_MODEL_PATH: PathBuf = {
    let mut path = PathBuf::from("tests/models");

    // Ensure the path is accessable for tests in sub-directories within the workspace.
    if !path.exists() {
      path = PathBuf::from("..").join(path);
    }

    path
  };
}

#[test]
fn test_model_path_exists() {
  assert!(TEST_MODEL_PATH.exists());
}

pub struct TestingData;

impl TestingData {
  pub const ANIMATION_NAME: &str = "test";

  /// Returns a test model from the models file.
  pub fn new_test_model(world_position: (usize, usize)) -> ModelData {
    let mut test_model_path = TEST_MODEL_PATH.clone();
    test_model_path.push("test_square.model");

    assert!(test_model_path.exists(), "{:?}", test_model_path);
    assert!(test_model_path.is_file(), "{:?}", test_model_path);

    ModelData::from_file(&test_model_path, world_position).unwrap()
  }

  /// Returns a test model without a hitbox from the models file.
  pub fn new_test_model_no_hitbox(world_position: (usize, usize)) -> ModelData {
    let mut test_model_path = TEST_MODEL_PATH.clone();
    test_model_path.push("test_model_no_hitbox.model");

    ModelData::from_file(&test_model_path, world_position).unwrap()
  }

  /// Returns an animated model with frames build from the characters passed in, and the animation in question.
  /// The animation can be used as comparisons for testing.
  ///
  /// The animation's name is "test" which can be accessed through TestingData::ANIMATION_NAME.
  pub fn new_test_model_animated(
    world_position: (usize, usize),
    animation_characters: [char; 3],
  ) -> (ModelData, AnimationFrames) {
    let animation_name = Self::ANIMATION_NAME.to_string();
    let animation = Self::get_test_animation(animation_characters, AnimationLoopCount::Limited(2));

    let model = Self::new_test_model_with_animation(
      world_position,
      vec![(animation_name.clone(), animation.clone())],
    );

    (model, animation)
  }

  /// Creates an animated model with the list of animations and names.
  ///
  /// This can be used instead of [`new_test_model_animated`](TestingData::new_test_model_animated) if
  /// you want an animated model with no animations or more than 1 animation.
  pub fn new_test_model_with_animation(
    _world_position: (usize, usize),
    _animations: Vec<(String, AnimationFrames)>,
  ) -> ModelData {
    todo!()
    // let mut model_data = Self::new_test_model(world_position);
    // let animation_data = ModelAnimationData::new(model_data.clone(), animations);
    //
    // model_data.assign_model_animation(animation_data);
    //
    // model_data
  }

  /// Creates a list of the given amount of models at the given position.
  pub fn get_multiple_test_models(position: (usize, usize), count: u32) -> Vec<ModelData> {
    (0..count).map(|_| Self::new_test_model(position)).collect()
  }

  // This is temporary until animation file parsers are a thing.
  /// Crates an animation with 3 frames.
  /// The frames have:
  ///   duration: 1 tick
  ///   anchor: 'a'
  ///   appearance: The characters passed in.
  ///   loop_count: Loop count passed in.
  pub fn get_test_animation(
    frame_characters: [char; 3],
    loop_count: AnimationLoopCount,
  ) -> AnimationFrames {
    let frames: Vec<AnimationFrame> = Self::get_test_frames(
      frame_characters
        .into_iter()
        .map(|frame_char| (Self::get_frame_appearance(frame_char), 1, frame_char))
        .collect::<Vec<(String, u32, char)>>(),
    );

    AnimationFrames::new(frames, loop_count, None)
  }

  /// Gets a list of frames with the passed in data (appearance, duration, anchor replacement).
  /// Default anchor is 'a'.
  pub fn get_test_frames(appearances: Vec<(String, u32, char)>) -> Vec<AnimationFrame> {
    appearances
      .into_iter()
      .map(|(appearance, duration, anchor_replacement)| {
        let sprite = Sprite::new(appearance, 'a', anchor_replacement, '-').unwrap();

        AnimationFrame::new(sprite, duration)
      })
      .collect()
  }

  /// Creates a 5x3 frame of the given character with 'a' as the anchor at index 7.
  pub fn get_frame_appearance(character: char) -> String {
    // let top_bottom_row: String = std::iter::repeat(character).take(5).collect();
    let row_of_character: String = character.to_string().as_str().repeat(5);
    let middle_row = format!("{c}{c}a{c}{c}", c = character);

    format!("{}\n{}\n{}", row_of_character, middle_row, row_of_character)
  }

  pub fn get_test_model_animation_data() -> ModelAnimationData {
    todo!()
  }
}
