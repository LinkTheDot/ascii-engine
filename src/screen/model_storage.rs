use crate::errors::*;
use crate::screen::stored_worlds::*;
use log::{error, info, warn};
use model_data_structures::models::model_data::*;
use model_data_structures::models::strata::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock, RwLockReadGuard};

/// This is the struct that contains a reference to every model that exists in the world.
#[derive(Debug, Default)]
pub(crate) struct ModelStorage {
  model_stratas: HashMap<Strata, HashSet<u64>>,
  models: HashMap<u64, ModelData>,
}

#[derive(Debug, Clone)]
pub(crate) struct ReadOnlyModelStorage {
  model_storage: Arc<RwLock<ModelStorage>>,
}

impl ModelStorage {
  /// Creates a reference to the passed in ModelData and stores it.
  ///
  /// # Errors
  ///
  /// - An error is returned when attempting to add a model that already exists.
  pub fn insert(&mut self, model: ModelData) -> Result<(), ModelError> {
    let key = model.get_hash();

    if self.models.get(&key).is_none() {
      self.models.insert(key, model);

      self.insert_strata(&key)?;
    } else {
      warn!("Attempted insert of model {key}, which already exists.",);

      return Err(ModelError::ModelAlreadyExists);
    }

    Ok(())
  }

  /// Removes the ModelData of the given key.
  ///
  /// Returns The ModelData if it existed, otherwise returns None.
  ///
  /// # Errors (yes there's technically an error)
  ///
  /// - Returns None when any existing model somehow has an impossible strata.
  pub fn remove(&mut self, key: &u64) -> Option<ModelData> {
    if self.model_exists(key) {
      self.remove_mention_of(key)
    } else {
      self.fix_strata_list().ok()?;

      self.remove_mention_of(key)
    }
  }

  /// Returns true if the model exists in both the ModelData list,
  /// and the Strata list pertaining to it's currently assigned strata.
  ///
  /// Returns false if the model's strata is somehow incorrect.
  /// Returns false if the model didn't exist internally.
  pub fn model_exists(&self, key: &u64) -> bool {
    self.models.contains_key(key)
  }

  /// Removes any mention the model corresponding to the passed in key.
  ///
  /// This means the key will be removed from the strata list and internal ModelData list.
  ///
  /// This will also delete the strata list if it's empty after removing this model.
  ///
  /// Returns the ModelData if it existed.
  /// Otherwise returns None.
  fn remove_mention_of(&mut self, key: &u64) -> Option<ModelData> {
    let model = self.models.remove(key)?;
    let model_strata = model.get_strata();

    self.model_stratas.get_mut(&model_strata)?.remove(key);

    if self.model_stratas.get(&model_strata)?.is_empty() {
      self.model_stratas.remove(&model_strata);
    }

    Some(model)
  }

  /// Returns a reference to the model hashes corresponding to the given Strata level.
  ///
  /// Returns None when no models in that Strata range exist.
  pub fn get_strata_keys(&self, key: &Strata) -> Option<&HashSet<u64>> {
    self.model_stratas.get(key)
  }

  /// Returns a copy of the requested ModelData.
  ///
  /// Returns None when the model doesn't exist.
  pub fn get_model(&self, key: &u64) -> Option<ModelData> {
    self.models.get(key).cloned()
  }

  #[allow(unused)]
  /// Returns a HashSet which contains references to the hashes of every model that exists in the world.
  pub fn get_model_keys(&self) -> HashSet<&u64> {
    self.models.keys().collect()
  }

  /// Returns a reference to the internal HashMap of <hash, ModelData.
  pub fn get_model_list(&self) -> &HashMap<u64, ModelData> {
    &self.models
  }

  /// Consumes self and returns a wrapper that contains the list of models that existed.
  pub fn extract_model_list(self) -> StoredWorld {
    let models = self.models.into_values().collect::<Vec<ModelData>>();

    StoredWorld::new(models)
  }

  /// Insert a model_hash to the model's currently assigned strata.
  ///
  /// # Errors
  ///
  /// - Returns an error when the given model_hash doesn't exist in the world.
  /// - Returns an error when a model somehow has an impossible strata range.
  fn insert_strata(&mut self, model_key: &u64) -> Result<(), ModelError> {
    let Some(model) = self.get_model(model_key) else {
      return Err(ModelError::ModelDoesntExist);
    };

    let model_strata = model.get_strata();
    let model_hash = model.get_hash();

    if let Some(strata_keys) = self.model_stratas.get_mut(&model_strata) {
      strata_keys.insert(model_hash);
    } else if model_strata.correct_range() {
      self
        .model_stratas
        .insert(model_strata, HashSet::from([(model_hash)]));
    } else {
      error!(
        "There was an attempt to insert model {} which has an impossible strata {model_strata:?}",
        model_hash
      );

      return Err(ModelError::IncorrectStrataRange(model_strata));
    }

    Ok(())
  }

  /// Iterates through every known model and checks if their assigned stratas are different from their current stratas.
  /// Any model who's strata was found to be incorrect is moved to the strata it's currently assigned to.
  ///
  /// # Errors
  ///
  /// - Returns an error when a model somehow has an impossible strata.
  pub fn fix_strata_list(&mut self) -> Result<(), ModelError> {
    for strata_number in 0..=100 {
      let current_strata = Strata(strata_number);

      let Some(strata_keys) = self.get_strata_keys(&current_strata) else {
        continue;
      };

      let incorrect_strata_list: Vec<(Strata, u64)> = strata_keys
        .iter()
        .map(|key| self.get_model(key).unwrap())
        .filter_map(|model| {
          let model_strata = model.get_strata();
          let model_hash = model.get_hash();

          if model_strata != current_strata {
            Some((current_strata, model_hash))
          } else {
            None
          }
        })
        .collect();

      incorrect_strata_list
        .into_iter()
        .try_for_each(|(new_strata, model_hash)| {
          info!("{model_hash} changed stratas to {current_strata:?}");

          self.fix_model_strata(&model_hash, current_strata, new_strata)
        })?;
    }

    Ok(())
  }

  /// Removes the given model_hash from 'old_strata' and moves it to 'new_strata'
  ///
  /// # Errors
  ///
  /// - Returns an error when the 'old_strata' doesn't exist. (meaning when it has no hashes)
  /// - Returns an error when the model wasn't contained in 'old_strata'.
  fn fix_model_strata(
    &mut self,
    key: &u64,
    old_strata: Strata,
    new_strata: Strata,
  ) -> Result<(), ModelError> {
    if !new_strata.correct_range() {
      return Err(ModelError::IncorrectStrataRange(new_strata));
    }

    if let Some(strata_keys) = self.model_stratas.get_mut(&old_strata) {
      let model_existed = strata_keys.remove(key);

      if model_existed {
        if strata_keys.is_empty() {
          self.model_stratas.remove(&old_strata);
        }

        return self.insert_strata(key);
      }
    }

    Err(ModelError::ModelDoesntExist)
  }

  pub(crate) fn create_read_only(model_storage: Arc<RwLock<Self>>) -> ReadOnlyModelStorage {
    ReadOnlyModelStorage { model_storage }
  }
}

impl ReadOnlyModelStorage {
  pub fn read_model_storage(&self) -> RwLockReadGuard<ModelStorage> {
    self.model_storage.read().unwrap()
  }
}

impl<I> From<I> for ModelStorage
where
  I: IntoIterator<Item = ModelData>,
{
  fn from(item: I) -> Self {
    let models = item.into_iter().collect::<Vec<ModelData>>();
    let mut model_storage = ModelStorage::default();

    for model in models {
      let _ = model_storage.insert(model);
    }

    model_storage
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use model_data_structures::models::testing_data::*;

  const WORLD_POSITION: (usize, usize) = (10, 10);

  #[cfg(test)]
  mod insert_and_remove_logic {
    use super::*;

    #[test]
    fn insert_once() {
      let mut model_list = ModelStorage::default();
      let model_data = TestingData::new_test_model(WORLD_POSITION);

      let result = model_list.insert(model_data);

      assert!(result.is_ok());
    }

    #[test]
    fn insert_twice() {
      let mut model_list = ModelStorage::default();
      let model_data = TestingData::new_test_model(WORLD_POSITION);

      let expected_result = Err(ModelError::ModelAlreadyExists);

      let result = model_list.insert(model_data.clone());
      assert!(result.is_ok());

      let result = model_list.insert(model_data);
      assert_eq!(result, expected_result);
    }

    #[test]
    fn insert_then_remove() {
      let mut model_list = ModelStorage::default();
      let model_data = TestingData::new_test_model(WORLD_POSITION);

      model_list.insert(model_data.clone()).unwrap();

      let model_hash = model_data.get_hash();
      let removed_data = model_list.remove(&model_hash).unwrap();

      assert_eq!(removed_data.get_hash(), model_hash);
    }

    #[test]
    fn remove_model_that_doesnt_exist() {
      let mut model_list = ModelStorage::default();
      let fake_key: u64 = 0;

      let result = model_list.remove(&fake_key);

      assert!(result.is_none());
    }
  }

  #[cfg(test)]
  mod get_logic {
    use super::*;

    #[test]
    fn get_strata_keys_invalid_strata() {
      let model_list = ModelStorage::default();

      let result = model_list.get_strata_keys(&Strata(0));

      assert!(result.is_none());
    }

    #[test]
    fn get_strata_keys_valid_strata() {
      let mut model_list = ModelStorage::default();
      let model_data = TestingData::new_test_model(WORLD_POSITION);
      let model_strata = model_data.get_strata();

      model_list.insert(model_data.clone()).unwrap();

      let strata_keys = model_list.get_strata_keys(&model_strata).unwrap();

      assert!(strata_keys.contains(&model_data.get_hash()));
    }

    #[test]
    fn get_existing_model() {
      let mut model_list = ModelStorage::default();
      let model_data = TestingData::new_test_model(WORLD_POSITION);

      model_list.insert(model_data.clone()).unwrap();

      let result = model_list.get_model(&model_data.get_hash()).unwrap();

      assert_eq!(result.get_hash(), model_data.get_hash());
    }

    #[test]
    fn get_model_doesnt_exist() {
      let model_list = ModelStorage::default();
      let fake_hash: u64 = 0;

      let result = model_list.get_model(&fake_hash);

      assert!(result.is_none());
    }

    #[test]
    fn get_model_keys() {
      let mut model_list = ModelStorage::default();
      let model_data = TestingData::new_test_model(WORLD_POSITION);

      model_list.insert(model_data.clone()).unwrap();

      let model_keys = model_list.get_model_keys();

      assert!(model_keys.contains(&model_data.get_hash()));
    }

    #[test]
    fn get_model_list() {
      let mut model_list = ModelStorage::default();
      let model_data = TestingData::new_test_model(WORLD_POSITION);

      model_list.insert(model_data.clone()).unwrap();

      let model_keys = model_list.get_model_list();

      assert!(model_keys.contains_key(&model_data.get_hash()));
    }
  }

  #[cfg(test)]
  mod insert_strata_logic {
    use super::*;

    #[test]
    fn model_exists() {
      let mut model_list = ModelStorage::default();
      let model_data = TestingData::new_test_model(WORLD_POSITION);
      let model_hash = model_data.get_hash();
      let model_strata = model_data.get_strata();

      model_list.models.insert(model_hash, model_data);

      let insert_result = model_list.insert_strata(&model_hash);
      let strata_keys = model_list.get_strata_keys(&model_strata).unwrap();

      assert!(insert_result.is_ok());
      assert!(strata_keys.contains(&model_hash));
    }

    #[test]
    fn list_already_exists() {
      let mut model_list = ModelStorage::default();
      let model_data = TestingData::new_test_model(WORLD_POSITION);
      let model_data_with_same_strata = TestingData::new_test_model(WORLD_POSITION);
      let model_hash = model_data.get_hash();
      let model_strata = model_data.get_strata();

      model_list.insert(model_data_with_same_strata).unwrap();

      model_list.models.insert(model_hash, model_data);

      let insert_result = model_list.insert_strata(&model_hash);
      let strata_keys = model_list.get_strata_keys(&model_strata).unwrap();

      assert!(insert_result.is_ok());
      assert!(strata_keys.contains(&model_hash));
    }

    #[test]
    fn model_doesnt_exist() {
      let mut model_list = ModelStorage::default();

      let fake_hash: u64 = 0;

      let insert_result = model_list.insert_strata(&fake_hash);

      assert!(insert_result.is_err());
    }
  }

  #[cfg(test)]
  mod fix_strata_list_logic {
    use super::*;

    #[test]
    fn misplaced_strata() {
      let mut model_list = ModelStorage::default();
      let model_data = TestingData::new_test_model(WORLD_POSITION);
      let model_hash = model_data.get_hash();
      let model_strata = model_data.get_strata();

      let fake_strata = Strata(0);
      let fake_strata_list = HashSet::from([(model_hash)]);

      model_list.models.insert(model_hash, model_data);
      model_list
        .model_stratas
        .insert(fake_strata, fake_strata_list);

      let fix_result = model_list.fix_strata_list();
      let real_strata_list = model_list.get_strata_keys(&model_strata).unwrap();
      let fake_strata_list = model_list.get_strata_keys(&fake_strata);

      assert!(fix_result.is_ok());
      assert!(fake_strata_list.is_none());
      assert!(real_strata_list.contains(&model_hash));
    }

    #[test]
    fn model_list_is_empty() {
      let mut model_list = ModelStorage::default();

      let result = model_list.fix_strata_list();

      assert!(result.is_ok())
    }

    #[test]
    fn fix_model_strata_fake_old_strata() {
      let mut model_list = ModelStorage::default();

      let expected_result = Err(ModelError::ModelDoesntExist);

      let fake_hash: u64 = 0;
      let fake_old_strata = Strata(0);
      let fake_new_strata = Strata(10);

      let result = model_list.fix_model_strata(&fake_hash, fake_old_strata, fake_new_strata);

      assert_eq!(result, expected_result);
    }
  }

  #[test]
  fn extract_model_list_logic() {
    let models: Vec<ModelData> = (0..5)
      .map(|_| TestingData::new_test_model(WORLD_POSITION))
      .collect();
    let model_storage = ModelStorage::from(models.clone());

    let expected_list: HashMap<u64, ModelData> = models
      .into_iter()
      .map(|model| (model.get_hash(), model))
      .collect();

    let stored_world = model_storage.extract_model_list();
    let model_list = ModelStorage::from(stored_world);

    assert_eq!(model_list.get_model_list(), &expected_list);
  }
}
