use crate::screen::model_storage::*;
use crate::{models::model_data::ModelData, prelude::ModelError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

// Describe the use of the model_manager
#[derive(Debug)]
pub struct ModelManager {
  model_storage: Arc<RwLock<ModelStorage>>,
}

/// Holds a reference to a read guard on the internal ModelStorage.
///
/// This is for passing references to data contained in the model storage that would potentially be
/// too large to clone otherwise.
pub(crate) struct ReadGuardHolder<'a> {
  read_guard: Option<RwLockReadGuard<'a, ModelStorage>>,
}

/// Holds a reference to a write guard on the internal ModelStorage.
///
/// This is for passing references to data contained in the model storage that would potentially be
/// too large to clone otherwise.
pub(crate) struct WriteGuardHolder<'a> {
  write_guard: Option<RwLockWriteGuard<'a, ModelStorage>>,
}

impl ModelManager {
  pub(crate) fn new(model_storage: Arc<RwLock<ModelStorage>>) -> Self {
    Self { model_storage }
  }

  pub(crate) fn fix_strata_list(&mut self) -> Result<(), ModelError> {
    self.model_storage.write().unwrap().fix_strata_list()
  }

  /// Returns a reference to the list of internally existing models.
  ///
  /// Also returns a [`ReadGuardHolder`](ReadGuardHolder).
  /// This is returned because cloning the entire map of existing models would be too costly.
  pub(crate) fn get_model_list<'a>(&self) -> (ReadGuardHolder<'a>, &'a HashMap<u64, ModelData>) {
    let model_storage_read_guard = self.model_storage.read().unwrap();
    let internal_model_list = model_storage_read_guard.get_model_list();
    let guard_holder = ReadGuardHolder::new(model_storage_read_guard);

    (guard_holder, internal_model_list)
  }

  /// Returns a copy of the Model with the given hash.
  ///
  /// None is returned if there was no model in the world with the given hash.
  pub fn get_model(&self, model_hash: &u64) -> Option<ModelData> {
    self.model_storage.read().unwrap().get_model(model_hash)
  }

  /// Returns true if the model of the given hash exists in the world.
  pub fn model_exists(&self, model_hash: &u64) -> bool {
    self.model_storage.read().unwrap().model_exists(model_hash)
  }
}

impl<'a> ReadGuardHolder<'a> {
  fn new(read_guard: RwLockReadGuard<ModelStorage>) -> Self {
    Self {
      read_guard: Some(read_guard),
    }
  }
}

impl<'a> WriteGuardHolder<'a> {
  fn new(write_guard: RwLockWriteGuard<ModelStorage>) -> Self {
    Self {
      write_guard: Some(write_guard),
    }
  }
}
