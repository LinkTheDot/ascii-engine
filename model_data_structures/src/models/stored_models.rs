use crate::models::model_appearance::*;
use crate::models::{hitboxes::*, model_data::*, strata::*};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Deserialize, Serialize)]
#[allow(unused)]
pub struct StoredDisplayModel {
  pub(crate) position: Option<usize>,
  pub(crate) name: Option<String>,
  pub(crate) strata: Option<Strata>,
  pub(crate) appearance_data: Option<ModelAppearance>,
  pub(crate) hitbox: Option<Hitbox>,
  pub(crate) tags: Option<HashSet<String>>,
}

impl StoredDisplayModel {
  pub(crate) fn new(mut model_data: ModelData) -> Self {
    let appearance_data = model_data.get_appearance_data().lock().unwrap().clone();

    Self {
      position: Some(model_data.get_frame_position()),
      name: Some(model_data.get_name()),
      strata: Some(model_data.get_strata()),
      appearance_data: Some(appearance_data),
      hitbox: Some(model_data.get_hitbox()),
      tags: Some(model_data.get_tags()),
    }
  }

  /// Returns whether or not the StoredDisplayModel could be repaired.
  /// If false is returned, then any attempt of restoring this model should be lost.
  ///
  /// Defaults any possibly repairable fields missing from the model.
  /// If the model is missing something simple like its Strata, then that is just replaced.
  /// If the model is missing something like its ModelAppearance or Tags, then it should no be fixed.
  pub(crate) fn repair_missing_fields(&mut self) -> bool {
    if self.position.is_none() {
      log::error!("Attempted to create a model from a stored model missing its position.");
      return false;
    }

    if self.name.is_none() {
      log::error!("Attempted to create a model from a stored model missing its name.");
      return false;
    }

    if self.appearance_data.is_none() {
      log::error!("Attempted to create a model from a stored model missing its appearance.");
      return false;
    }

    if self.hitbox.is_none() {
      log::error!("Attempted to create a model from a stored model missing its hitbox.");
      return false;
    }

    if self.tags.is_none() {
      log::error!("Attempted to create a model from a stored model missing its tags.");
      return false;
    }

    if self.strata.is_none() {
      log::error!(
        "Attempted to create a model from a stored model missing its Strata, setting strata to 0."
      );

      self.strata = Some(Strata(0));
    }

    true
  }
}
//
// impl PartialEq for StoredDisplayModel {
//   fn eq(&self, other: &Self) -> bool {
//     self.unique_hash == other.unique_hash
//   }
// }
//
// impl Eq for StoredDisplayModel {}
//
// impl PartialOrd for StoredDisplayModel {
//   fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//     Some(self.cmp(other))
//   }
// }
//
// impl Ord for StoredDisplayModel {
//   fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//     self.unique_hash.cmp(&other.unique_hash)
//   }
// }
