use crate::models::model_appearance::*;
use crate::models::{hitboxes::*, model_data::*, strata::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[allow(unused)]
pub struct StoredDisplayModel {
  pub(crate) unique_hash: u64,
  pub(crate) position: usize,
  pub(crate) name: String,

  pub(crate) strata: Strata,
  pub(crate) appearance_data: ModelAppearance,
  pub(crate) hitbox: Hitbox,
}

impl StoredDisplayModel {
  pub(crate) fn new(mut model_data: ModelData) -> Self {
    let appearance_data = model_data.get_appearance_data().lock().unwrap().clone();

    Self {
      unique_hash: model_data.get_hash(),
      position: model_data.get_frame_position(),
      name: model_data.get_name(),
      strata: model_data.get_strata(),
      appearance_data,
      hitbox: model_data.get_hitbox(),
    }
  }

  pub fn get_hash(&self) -> u64 {
    self.unique_hash
  }

  pub fn get_name(&self) -> String {
    self.name.clone()
  }
}

impl PartialEq for StoredDisplayModel {
  fn eq(&self, other: &Self) -> bool {
    self.unique_hash == other.unique_hash
  }
}

impl Eq for StoredDisplayModel {}

impl PartialOrd for StoredDisplayModel {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for StoredDisplayModel {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.unique_hash.cmp(&other.unique_hash)
  }
}
