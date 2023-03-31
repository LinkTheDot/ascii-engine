use crate::errors::*;
use crate::models::model_data::*;
use guard::guard;
use log::{error, info, warn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// Model data will contain all model hashes and their corresponding model types.
#[derive(Debug)]
pub struct Models {
  model_stratas: HashMap<Strata, HashSet<u64>>,
  models: HashMap<u64, Arc<Mutex<ModelData>>>,
}

impl Models {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    Self {
      model_stratas: HashMap::new(),
      models: HashMap::new(),
    }
  }

  pub fn insert<O: DisplayModel>(&mut self, key: &u64, model: &O) -> Result<(), ModelError> {
    if self.models.get(key).is_none() {
      self.models.insert(*key, model.get_model_data());

      self.insert_strata(key)?;
    } else {
      warn!(
        "Attempted insert of model {}, which already exists.",
        model.get_unique_hash()
      );

      return Err(ModelError::ModelAlreadyExists);
    }

    Ok(())
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
  pub fn get_model(&self, key: &u64) -> Option<Arc<Mutex<ModelData>>> {
    self.models.get(key).cloned()
  }

  pub fn get_model_keys(&self) -> HashSet<&u64> {
    self.models.keys().collect()
  }

  pub fn get_model_list(&self) -> &HashMap<u64, Arc<Mutex<ModelData>>> {
    &self.models
  }

  fn change_model_strata(&mut self, key: &u64, new_strata: Strata) -> Result<(), ModelError> {
    if !new_strata.correct_range() {
      return Err(ModelError::IncorrectStrataRange(new_strata));
    }

    if let Some(model_lock) = self.models.get_mut(key) {
      let model_guard = model_lock.lock().unwrap();
      let old_strata = *model_guard.get_strata();
      drop(model_guard);

      if let Some(strata_keys) = self.model_stratas.get_mut(&old_strata) {
        let model_existed = strata_keys.remove(key);

        if model_existed {
          return self.insert_strata(key);
        }
      }
    }

    Err(ModelError::ModelDoesntExist)
  }

  fn insert_strata(&mut self, model_key: &u64) -> Result<(), ModelError> {
    guard!( let Some(model) = self.get_model(model_key) else {
      return Err(ModelError::ModelDoesntExist)
    });
    let model = model.lock().unwrap();

    let model_strata = *model.get_strata();
    let model_hash = *model.get_unique_hash();
    drop(model);

    if let Some(strata_keys) = self.model_stratas.get_mut(&model_strata) {
      strata_keys.insert(model_hash);
    } else if model_strata.correct_range() {
      self
        .model_stratas
        .insert(model_strata, HashSet::from([(model_hash)]));
    } else {
      // Might be an impossible error, if so remove it.
      error!(
        "There was an attempt to insert model {} which has an impossible strata {model_strata:?}",
        model_hash
      );

      return Err(ModelError::IncorrectStrataRange(model_strata));
    }

    Ok(())
  }

  pub fn fix_strata_list(&mut self) -> Result<(), ModelError> {
    for strata_number in 0..=100 {
      let current_strata = Strata(strata_number);

      guard!( let Some(strata_keys) = self.get_strata_keys(&current_strata) else { continue; } );

      let incorrect_strata_list: Vec<(Strata, u64)> = strata_keys
        .iter()
        .map(|key| self.get_model(key).unwrap())
        .filter_map(|model| {
          let model_guard = model.lock().unwrap();
          let model_strata = model_guard.get_strata();
          let model_hash = *model_guard.get_unique_hash();

          if model_strata != &current_strata {
            drop(model_guard);

            Some((current_strata, model_hash))
          } else {
            drop(model_guard);
            None
          }
        })
        .collect();

      incorrect_strata_list
        .into_iter()
        .try_for_each(|(strata, model_hash)| {
          info!("{model_hash} changed stratas to {current_strata:?}");

          self.change_model_strata(&model_hash, strata)
        })?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::hitboxes::HitboxCreationData;
  use crate::CONFIG;

  #[derive(DisplayModel)]
  struct TestModel {
    model_data: Arc<Mutex<ModelData>>,
  }

  impl TestModel {
    fn new() -> Self {
      let hitbox_data = HitboxCreationData::new("xxx\nxcx", 'c');
      let skin = Skin::new("xxx\nxcx", 'c', ' ', '-').unwrap();
      let name = String::from("Test_Model");
      let strata = Strata(0);
      let sprite = Sprite::new(skin).unwrap();
      let model_data = ModelData::new((0, 0), sprite, hitbox_data, strata, name).unwrap();

      Self {
        model_data: Arc::new(Mutex::new(model_data)),
      }
    }
  }

  #[cfg(test)]
  mod change_model_strata_logic {
    use super::*;

    #[test]
    fn used_impossible_strata() {
      let (mut models, test_model) = get_test_data();
      let impossible_strata = Strata(101);
      let model_key = test_model.get_unique_hash();

      let expected_result = Err(ModelError::IncorrectStrataRange(Strata(101)));

      let result = models.change_model_strata(&model_key, impossible_strata);

      assert_eq!(expected_result, result);
    }
  }

  fn get_test_data() -> (Models, TestModel) {
    let mut models = Models::new();
    let test_model = TestModel::new();

    models
      .insert(&test_model.get_unique_hash(), &test_model)
      .unwrap();

    (models, test_model)
  }
}
