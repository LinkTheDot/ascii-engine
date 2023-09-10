use crate::models::{animation::*, hitboxes::*, model_data::*, sprites::*, strata::*};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Debug, Deserialize, Serialize)]
#[allow(unused)]
pub struct StoredDisplayModel {
  pub(crate) unique_hash: u64,
  pub(crate) position: usize,
  pub(crate) name: String,

  pub(crate) strata: Strata,
  pub(crate) sprite: Sprite,
  pub(crate) hitbox: Hitbox,

  pub(crate) animation_data: Option<HashMap<String, AnimationFrames>>,
}

impl StoredDisplayModel {
  pub(crate) fn new(mut model_data: ModelData) -> Self {
    let model_animation_data = model_data.get_animation_data().map(get_animations);
    let sprite = extract_arc_rwlock(model_data.get_sprite());

    Self {
      unique_hash: model_data.get_hash(),
      position: model_data.get_frame_position(),
      name: model_data.get_name(),
      strata: model_data.get_strata(),
      sprite,
      hitbox: model_data.get_hitbox(),
      animation_data: model_animation_data,
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

fn get_animations(
  animation_data: Arc<Mutex<ModelAnimationData>>,
) -> HashMap<String, AnimationFrames> {
  animation_data.lock().unwrap().get_animation_list().clone()
}

fn extract_arc_rwlock<T: Clone>(item: Arc<RwLock<T>>) -> T {
  item.read().unwrap().clone()
}
