// #![cfg(test)]

use crate::prelude::*;
use lazy_static::lazy_static;
use std::path::PathBuf;

lazy_static! {
  static ref TEST_MODEL_PATH: PathBuf = {
    // let mut project_root = project_root::get_project_root().unwrap();
    //
    // if !project_root.ends_with("ascii_engine") {
    //   project_root.pop();
    // }
    //
    // project_root.push("tests/models");
    //
    // project_root
    let mut path = PathBuf::from("tests/models");

    if !path.exists() {
      path = PathBuf::from("..").join(path);
    }

    path
  };
}
// const TEST_MODEL_PATH: &str = "tests/models/";

#[test]
fn test_model_path_exists() {
  assert!(TEST_MODEL_PATH.exists());
}

pub struct TestingData;

impl TestingData {
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

    AnimationFrames::new(frames, loop_count)
  }

  /// Gets a list of frames with the passed in data (appearance, duration, anchor replacement).
  /// Default anchor is 'a'.
  pub fn get_test_frames(appearances: Vec<(String, u32, char)>) -> Vec<AnimationFrame> {
    let mut base_frame = Sprite::new();
    base_frame.change_anchor_character('a').unwrap();

    appearances
      .into_iter()
      .map(|(appearance, duration, center_replacement)| {
        base_frame
          .change_shape(appearance, None, Some(center_replacement))
          .unwrap();

        AnimationFrame::new(base_frame.clone(), duration)
      })
      .collect()
  }

  /// Creates a 5x3 frame of the given character with 'a' as the anchor at index 7.
  pub fn get_frame_appearance(character: char) -> String {
    let top_bottom_row: String = std::iter::repeat(character).take(5).collect();
    let middle_row = format!("{c}{c}a{c}{c}", c = character);

    format!("{}\n{}\n{}", top_bottom_row, middle_row, top_bottom_row)
  }
}
