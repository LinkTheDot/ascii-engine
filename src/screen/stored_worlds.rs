use model_data_structures::{
  models::{model_data::*, stored_models::StoredDisplayModel},
  prelude::ScreenError,
};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

/// A storage for the list of models that exist in a given state of the world.
#[derive(Debug)]
pub struct StoredWorld {
  models: Vec<StoredDisplayModel>,
}

pub struct StoredWorldIntoIterator {
  models: Vec<StoredDisplayModel>,
}

impl Iterator for StoredWorldIntoIterator {
  type Item = ModelData;

  fn next(&mut self) -> Option<Self::Item> {
    match ModelData::from_stored(self.models.pop()?) {
      Ok(model_data) => Some(model_data),
      Err(error) => {
        log::error!(
          "Failed to load a display model when creating the world: {:?}",
          error
        );

        None
      }
    }
  }
}

impl IntoIterator for StoredWorld {
  type Item = ModelData;
  type IntoIter = StoredWorldIntoIterator;

  fn into_iter(mut self) -> Self::IntoIter {
    StoredWorldIntoIterator {
      models: std::mem::take(&mut self.models),
    }
  }
}

impl StoredWorld {
  /// Creates a new instance of a StoredWorld with the list of models given.
  pub fn new<I>(models: I) -> Self
  where
    I: IntoIterator<Item = ModelData>,
  {
    let models: Vec<StoredDisplayModel> = models.into_iter().map(ModelData::to_stored).collect();

    Self { models }
  }

  // TODO: List the errors.
  pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ScreenError> {
    let path = path.as_ref();

    if !path.exists() {
      return Err(ScreenError::FileDoesNotExist);
    }

    let encoded_file_contents: Vec<u8> = match fs::read(path) {
      Ok(file_contents) => file_contents,
      #[cfg(not(tarpaulin_inclue))]
      Err(error) => return Err(ScreenError::Other(error.to_string())),
    };

    // Tarpaulin is bugged and doesn't include the overlapped match statement.
    #[cfg(not(tarpaulin_inclue))]
    let deserialized_stored_model_list =
      match bincode::deserialize::<Vec<StoredDisplayModel>>(&encoded_file_contents) {
        Ok(data) => data,
        Err(error) => return Err(ScreenError::FailedToLoadWorld(error.to_string())),
      };

    // Convert the StoredModels into model data.
    let models: Vec<ModelData> = deserialized_stored_model_list
      .into_iter()
      .filter_map(|model| {
        if let Ok(model) = ModelData::from_stored(model) {
          Some(model)
        } else {
          log::error!("Failed to load a model from the world");

          None
        }
      })
      .collect();

    Ok(Self::new(models))
  }

  /// Writes the data for the world in a file at the given path.
  /// Overwrites any file that was in that location.
  // TODO: List the errors.
  pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), ScreenError> {
    let serialized_world = match bincode::serialize(&self.models) {
      Ok(serialized_world) => serialized_world,
      Err(error) => return Err(ScreenError::Other(error.to_string())),
    };

    truncate_or_create_then_write(path, serialized_world)
  }

  /// Returns the amount of models stored.
  pub fn model_count(&self) -> usize {
    self.models.len()
  }
}

/// Takes a given path and creates the file if it doesn't exist, then writing the data to it.
/// If the file in the path already exists, this function will truncate it.
///
/// # Errors
///
/// - When the parent directory didn't exist.
/// - When the file couldn't be opened.
/// - When the file couldn't be written to.
pub fn truncate_or_create_then_write<P: AsRef<Path>>(
  path: P,
  data: Vec<u8>,
) -> Result<(), ScreenError> {
  let path = path.as_ref();
  let path_parent = path.parent().unwrap();

  if !path_parent.exists() && path_parent != std::path::Path::new("") {
    return Err(ScreenError::Other(
      "Parent directory for the save path does not exist.".to_string(),
    ));
  }

  let open_options_result = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open(path);

  let mut file = match open_options_result {
    Ok(file) => file,
    Err(error) => return Err(ScreenError::Other(error.to_string())),
  };

  if let Err(error) = file.write_all(&data) {
    return Err(ScreenError::Other(error.to_string()));
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::screen::screen_data::ScreenData;
  use engine_math::hasher::get_unique_hash;
  use model_data_structures::models::testing_data::TestingData;
  use std::path::PathBuf;

  #[test]
  fn world_iteration() {
    let model_count = 5;
    let test_models = TestingData::get_multiple_test_models((10, 10), model_count);
    let stored_world = StoredWorld::new(test_models.clone());

    assert!(stored_world.into_iter().count() == model_count as usize);
  }

  #[test]
  fn test_world_from_file() {
    let screen = ScreenData::from_world(get_test_world());
    let model_manager = screen.get_model_manager();

    model_manager.get_model_list(|model_list| {
      assert!(model_list.keys().count() == 5);
    })
  }

  #[test]
  fn save_and_load_file() {
    let temporary_test_file_path: PathBuf = generate_temporary_test_file_path();
    let model_count = 5;
    let test_models = TestingData::get_multiple_test_models((10, 10), model_count);
    let stored_world = StoredWorld::new(test_models.clone());

    stored_world.save(temporary_test_file_path.clone()).unwrap();
    let loaded_world = StoredWorld::load(temporary_test_file_path.clone()).unwrap();

    assert!(loaded_world.model_count() == model_count as usize);

    // Remove the file
    fs::remove_file(temporary_test_file_path.clone()).unwrap();
    assert!(!temporary_test_file_path.exists());
  }

  #[test]
  fn save_path_parent_does_not_exist() {
    let mut temporary_test_file_path: PathBuf = generate_temporary_test_file_path();
    temporary_test_file_path.push(generate_temporary_test_file_path());
    let test_models = TestingData::get_multiple_test_models((10, 10), 5);
    let stored_world = StoredWorld::new(test_models.clone());

    assert!(stored_world.save(temporary_test_file_path).is_err());
  }

  #[test]
  fn corrupted_file_does_not_load() {
    let temporary_test_file_path: PathBuf = generate_temporary_test_file_path();
    let test_models = TestingData::get_multiple_test_models((10, 10), 5);
    let stored_world = StoredWorld::new(test_models.clone());

    stored_world.save(temporary_test_file_path.clone()).unwrap();

    // Write junk to the file.
    let mut file = fs::File::options()
      .write(true)
      .open(temporary_test_file_path.clone())
      .unwrap();
    write!(file, "{x}{x}{x}", x = get_unique_hash()).unwrap();
    drop(file);

    let result = StoredWorld::load(temporary_test_file_path.clone());

    fs::remove_file(temporary_test_file_path.clone()).unwrap();
    assert!(!temporary_test_file_path.exists());
    assert!(result.is_err());
  }

  #[test]
  fn load_fake_path() {
    let path: PathBuf = generate_temporary_test_file_path();

    let expected_result = ScreenError::FileDoesNotExist;

    let result = StoredWorld::load(path).unwrap_err();

    assert_eq!(result, expected_result);
  }

  // data for tests

  #[cfg(test)]
  #[cfg(not(tarpaulin_include))]
  pub fn get_test_world() -> StoredWorld {
    let test_world_path = PathBuf::from("tests/worlds/test_template.world");

    if !test_world_path.exists() {
      let test_models = TestingData::get_multiple_test_models((10, 10), 5);
      let stored_world = StoredWorld::new(test_models);

      stored_world.save(test_world_path).unwrap();

      stored_world
    } else {
      StoredWorld::load(test_world_path).unwrap()
    }
  }

  #[cfg(test)]
  fn generate_temporary_test_file_path() -> PathBuf {
    PathBuf::from(format!("test_file-{}.world", get_unique_hash()))
  }
}
