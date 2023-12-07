#![cfg(test)]

use ascii_engine::prelude::*;
use model_data_structures::models::testing_data::TestingData;

const WORLD_POSITION: (usize, usize) = (10, 10);

#[test]
fn from_file_invalid_file_extension() {
  let path = std::path::Path::new("src/lib.rs");

  let expected_result = Err(ModelCreationError::NonModelFile.into());

  let result = ModelData::from_file(path, WORLD_POSITION);

  assert_eq!(result, expected_result);
}

#[test]
fn from_file_model_doesnt_exist() {
  let path = std::path::Path::new("this_is_a_name_nobody_should_take_ionuwvuiobnwvnbiouervw.model");

  // Check if the file exists
  assert!(
    !path.exists(),
    "A test file that isn't suppose to exist was found."
  );

  let file_path_string = path.file_name().map(|path_string| path_string.to_owned());
  let path_error = ModelCreationError::ModelFileDoesntExist(file_path_string);
  let expected_result = Err(ModelError::ModelCreationError(path_error));

  let result = ModelData::from_file(path, (0, 0));

  assert_eq!(result, expected_result);
}

#[test]
fn change_name() {
  let test_model = TestingData::new_test_model(WORLD_POSITION);
  let model_name = test_model.get_name();

  let expected_old_name = String::from("Test_Square");
  let expected_new_name = String::from("New Name");

  test_model.change_name(String::from("New Name"));

  let new_name = test_model.get_name();

  assert_eq!(model_name, expected_old_name);
  assert_eq!(new_name, expected_new_name);
}

#[test]
fn eq_logic() {
  let test_model = TestingData::new_test_model(WORLD_POSITION);

  #[allow(clippy::redundant_clone)]
  let cloned_model_data = test_model.clone();

  assert_eq!(test_model, cloned_model_data);
}

#[test]
fn display_model_logic() {
  #[derive(DisplayModel)]
  struct _X;
}

#[test]
fn tag_logic() {
  let mut model = TestingData::new_test_model(WORLD_POSITION);
  let tags = vec!["Player".to_string(), "Test".to_string()];
  model.add_tags(tags);

  assert!(model.contains_tag("Player"));
  assert!(model.contains_tag("Test"));
  assert!(model.contains_tags(&["Test", "Player"]));
}
