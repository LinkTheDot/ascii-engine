use crate::errors::*;
use crate::models::model_data::*;
use guard::guard;
use log::{error, info, warn};
use std::collections::{HashMap, HashSet};

/// This the the struct that contains a reference to every model that exists in the world.
/// InternalModels contains a list of stratas 0-100 and the hashes of which objects correspond to those stratas.
///
/// The user will never create this, it will merely be apart of the screen.
/// Every model will also hold a reference to this for two reasons.
///
/// One, when a model changes it's strata it needs to reflect that internally. and;
///
/// Two, models need to know about the existence of every other model for collision checks.
#[derive(Debug)]
pub struct InternalModels {
  model_stratas: HashMap<Strata, HashSet<u64>>,
  models: HashMap<u64, ModelData>,
}

impl InternalModels {
  #[allow(clippy::new_without_default)]
  /// Creates a new instance of the InternalModels struct.
  ///
  /// This will only be called by the screen and is not needed anywhere else.
  pub fn new() -> Self {
    Self {
      model_stratas: HashMap::new(),
      models: HashMap::new(),
    }
  }

  /// Creates a reference to the passed in ModelData and stores it.
  ///
  /// # Errors
  ///
  /// - An error is returned when attempting to add a model that already exists.
  pub fn insert(&mut self, model: ModelData) -> Result<(), ModelError> {
    let key = model.get_unique_hash();

    if self.models.get(&key).is_none() {
      self.models.insert(key, model);

      self.insert_strata(&key)?;
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
  pub fn get_model(&self, key: &u64) -> Option<ModelData> {
    self.models.get(key).cloned()
  }

  /// Returns a HashSet which contains references to the hashes of every model that exists in the world.
  pub fn get_model_keys(&self) -> HashSet<&u64> {
    self.models.keys().collect()
  }

  /// Returns a reference to the internal HashMap of <hash, ModelData>.
  pub fn get_model_list(&self) -> &HashMap<u64, ModelData> {
    &self.models
  }

  /// Insert a model_hash to the model's currently assigned strata.
  ///
  /// # Errors
  ///
  /// - Returns an error when the given model_hash doesn't exist in the world.
  // (I don't know if this below is possible or not. I'll leave it just incase something comes up in the future that makes it possible.)
  /// - Returns an error when a model somehow has an impossible strata range.
  fn insert_strata(&mut self, model_key: &u64) -> Result<(), ModelError> {
    guard!( let Some(model) = self.get_model(model_key) else {
      return Err(ModelError::ModelDoesntExist)
    });
    let model_strata = model.get_strata();
    let model_hash = model.get_unique_hash();

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

      guard!( let Some(strata_keys) = self.get_strata_keys(&current_strata) else { continue; } );

      let incorrect_strata_list: Vec<(Strata, u64)> = strata_keys
        .iter()
        .map(|key| self.get_model(key).unwrap())
        .filter_map(|model| {
          let model_strata = model.get_strata();
          let model_hash = model.get_unique_hash();

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
        return self.insert_strata(key);
      }
    }

    Err(ModelError::ModelDoesntExist)
  }
}
