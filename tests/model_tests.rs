#![allow(unused)]

use ascii_engine::prelude::*;

const WORLD_POSITION: (usize, usize) = (10, 10);
const SHAPE: &str = "xxxxx\nxxaxx\nxxxxx";
const ANCHOR_CHAR: char = 'a';
const ANCHOR_REPLACEMENT_CHAR: char = 'x';
const AIR_CHAR: char = '-';
const MODEL_NAME: &str = "Test_Model";

#[test]
fn absolute_movement_logic() {
  let mut screen = ScreenData::new();
  let mut test_model = TestModel::new();

  screen.add_model(&test_model).unwrap();

  let expected_collisions = 0;
  let expected_position = ((CONFIG.grid_width + 1) as usize * 11) + 11;

  let collisions = test_model.absolute_movement((11, 11));

  let new_model_position = test_model.get_position();

  assert_eq!(collisions.len(), expected_collisions);
  assert_eq!(new_model_position, expected_position);
}

#[test]
fn relative_movement_logic() {
  let mut screen = ScreenData::new();
  let mut test_model = TestModel::new();

  screen.add_model(&test_model).unwrap();

  let expected_collisions = 0;
  let expected_position = ((CONFIG.grid_width + 1) as usize * 11) + 11;

  let collisions = test_model.relative_movement((1, 1));

  let new_model_position = test_model.get_position();

  assert_eq!(collisions.len(), expected_collisions);
  assert_eq!(new_model_position, expected_position);
}

#[test]
fn from_file_invalid_file_extension() {
  let path = std::path::Path::new("src/lib.rs");

  let expected_result = Err(ModelError::NonModelFile);

  let result = ModelData::from_file(path, (0, 0));

  assert_eq!(result, expected_result);
}

#[test]
fn from_file_model_doesnt_exist() {
  let path = std::path::Path::new("this_is_a_name_nobody_should_take.model");

  let file_path_string = path.file_name().map(|path_string| path_string.to_owned());
  let path_error = ModelCreationError::ModelFileDoesntExist(file_path_string);
  let expected_result = Err(ModelError::ModelCreationError(path_error));

  let result = ModelData::from_file(path, (0, 0));

  assert_eq!(result, expected_result);
}

#[test]
fn change_strata() {
  let mut screen = ScreenData::new();
  let mut test_model = TestModel::new();

  screen.add_model(&test_model).unwrap();

  let expected_strata = Strata(50);

  test_model.change_strata(Strata(50)).unwrap();

  let model_strata = test_model.get_strata();

  assert_eq!(model_strata, expected_strata);
}

#[test]
fn change_name() {
  let mut test_model = TestModel::new();
  let model_name = test_model.get_name();

  let expected_old_name = String::from("Test_Square");
  let expected_new_name = String::from("New Name");

  test_model.change_name(String::from("New Name"));

  let new_name = test_model.get_name();

  assert_eq!(model_name, expected_old_name);
  assert_eq!(new_name, expected_new_name);
}

#[cfg(test)]
mod check_collisions_against_all_models {
  use super::*;

  #[test]
  fn no_other_models() {
    let mut screen = ScreenData::new();
    let test_model = TestModel::new();
    let model_data = test_model.get_model_data();

    screen.add_model(&test_model).unwrap();

    let expected_collisions = vec![];

    let collisions = model_data.check_collisions_against_all_models();

    assert_eq!(collisions, expected_collisions);
  }

  #[test]
  fn one_other_model_no_collision() {
    let mut screen = ScreenData::new();
    let model_one = TestModel::new();
    let model_two = TestModel::new();
    let model_data_one = model_one.get_model_data();
    let mut model_data_two = model_two.get_model_data();

    model_data_two.absolute_movement((20, 20));
    screen.add_model(&model_one).unwrap();
    screen.add_model(&model_two).unwrap();

    let expected_collisions = vec![];

    let collisions = model_data_one.check_collisions_against_all_models();

    assert_eq!(collisions, expected_collisions);
  }

  #[test]
  fn one_other_model_colliding() {
    let mut screen = ScreenData::new();
    let model_one = TestModel::new();
    let model_two = TestModel::new();
    let model_data_one = model_one.get_model_data();
    let model_data_two = model_two.get_model_data();

    screen.add_model(&model_one).unwrap();
    screen.add_model(&model_two).unwrap();

    let expected_collisions = vec![model_data_two];

    let collisions = model_data_one.check_collisions_against_all_models();

    assert_eq!(collisions, expected_collisions);
  }

  #[test]
  fn two_other_models_colliding() {
    let mut screen = ScreenData::new();
    let model_one = TestModel::new();
    let model_two = TestModel::new();
    let model_three = TestModel::new();
    let model_data_one = model_one.get_model_data();
    let model_data_two = model_two.get_model_data();
    let model_data_three = model_three.get_model_data();

    screen.add_model(&model_one).unwrap();
    screen.add_model(&model_two).unwrap();
    screen.add_model(&model_three).unwrap();

    let mut expected_collisions: Vec<u64> = vec![model_data_two, model_data_three]
      .into_iter()
      .map(|model_data| model_data.get_unique_hash())
      .collect();

    let mut collisions: Vec<u64> = model_data_one
      .check_collisions_against_all_models()
      .into_iter()
      .map(|model_data| model_data.get_unique_hash())
      .collect();

    collisions.sort();
    expected_collisions.sort();

    assert_eq!(collisions, expected_collisions);
  }
}

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
}
