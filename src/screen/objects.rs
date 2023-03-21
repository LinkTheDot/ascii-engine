use crate::errors::*;
use crate::objects::object_data::*;
use guard::guard;
use log::{error, info, warn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// Object data will contain all object hashes and their corresponding object types.
#[derive(Debug)]
pub struct Objects {
  object_stratas: HashMap<Strata, HashSet<u64>>,
  objects: HashMap<u64, Arc<Mutex<ObjectData>>>,
}

impl Objects {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    Self {
      object_stratas: HashMap::new(),
      objects: HashMap::new(),
    }
  }

  pub fn insert<O: Object>(&mut self, key: &u64, object: &O) -> Result<(), ObjectError> {
    if self.objects.get(key).is_none() {
      self.objects.insert(*key, object.get_object_data());

      self.insert_strata(key)?;
    } else {
      warn!(
        "Attempted insert of object {}, which already exists.",
        object.get_unique_hash()
      );

      return Err(ObjectError::ObjectAlreadyExists);
    }

    Ok(())
  }

  /// Returns a reference to the object hashes corresponding to the given Strata level.
  ///
  /// Returns None when no objects in that Strata range exist.
  pub fn get_strata_keys(&self, key: &Strata) -> Option<&HashSet<u64>> {
    self.object_stratas.get(key)
  }

  /// Returns a copy of the requested ObjectData.
  ///
  /// Returns None when the object doesn't exist.
  pub fn get_object(&self, key: &u64) -> Option<Arc<Mutex<ObjectData>>> {
    self.objects.get(key).cloned()
  }

  pub fn get_object_keys(&self) -> HashSet<&u64> {
    self.objects.keys().collect()
  }

  pub fn get_object_list(&self) -> &HashMap<u64, Arc<Mutex<ObjectData>>> {
    &self.objects
  }

  fn change_object_strata(&mut self, key: &u64, new_strata: Strata) -> Result<(), ObjectError> {
    if !new_strata.correct_range() {
      return Err(ObjectError::IncorrectStrataRange(new_strata));
    }

    if let Some(object_lock) = self.objects.get_mut(key) {
      let object_guard = object_lock.lock().unwrap();
      let old_strata = *object_guard.get_strata();
      drop(object_guard);

      if let Some(strata_keys) = self.object_stratas.get_mut(&old_strata) {
        let object_existed = strata_keys.remove(key);

        if object_existed {
          return self.insert_strata(key);
        }
      }
    }

    Err(ObjectError::ObjectDoesntExist)
  }

  fn insert_strata(&mut self, object_key: &u64) -> Result<(), ObjectError> {
    guard!( let Some(object) = self.get_object(object_key) else {
      return Err(ObjectError::ObjectDoesntExist)
    });
    let object = object.lock().unwrap();

    let object_strata = *object.get_strata();
    let object_hash = *object.get_unique_hash();
    drop(object);

    if let Some(strata_keys) = self.object_stratas.get_mut(&object_strata) {
      strata_keys.insert(object_hash);
    } else if object_strata.correct_range() {
      self
        .object_stratas
        .insert(object_strata, HashSet::from([(object_hash)]));
    } else {
      // Might be an impossible error, if so remove it.
      error!(
        "There was an attempt to insert object {} which has an impossible strata {object_strata:?}",
        object_hash
      );

      return Err(ObjectError::IncorrectStrataRange(object_strata));
    }

    Ok(())
  }

  pub fn fix_strata_list(&mut self) -> Result<(), ObjectError> {
    for strata_number in 0..=100 {
      let current_strata = Strata(strata_number);

      guard!( let Some(strata_keys) = self.get_strata_keys(&current_strata) else { continue; } );
      let object_list: Vec<Arc<Mutex<ObjectData>>> = strata_keys
        .iter()
        .map(|key| self.get_object(key).unwrap())
        .collect();

      for object in object_list {
        let object_guard = object.lock().unwrap();
        let object_strata = object_guard.get_strata();

        if object_strata != &current_strata {
          let object_hash = *object_guard.get_unique_hash();
          drop(object_guard);

          info!("{object_hash} changed stratas to {current_strata:?}");

          self.change_object_strata(&object_hash, current_strata)?;
        }
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::objects::hitboxes::HitboxCreationData;
  use crate::CONFIG;

  #[derive(Object)]
  struct TestObject {
    object_data: Arc<Mutex<ObjectData>>,
  }

  impl TestObject {
    fn new() -> Self {
      let hitbox_data = HitboxCreationData::new("xxx\nxcx", 'c');
      let skin = Skin::new("xxx\nxcx", 'c', ' ', '-').unwrap();
      let name = String::from("Test_Object");
      let strata = Strata(0);
      let sprite = Sprite::new(skin).unwrap();
      let object_data = ObjectData::new((0, 0), sprite, hitbox_data, strata, name).unwrap();

      Self {
        object_data: Arc::new(Mutex::new(object_data)),
      }
    }
  }

  #[cfg(test)]
  mod change_object_strata_logic {
    use super::*;

    #[test]
    fn used_impossible_strata() {
      let (mut objects, test_object) = get_test_data();
      let impossible_strata = Strata(101);
      let object_key = test_object.get_unique_hash();

      let expected_result = Err(ObjectError::IncorrectStrataRange(Strata(101)));

      let result = objects.change_object_strata(&object_key, impossible_strata);

      assert_eq!(expected_result, result);
    }
  }

  fn get_test_data() -> (Objects, TestObject) {
    let mut objects = Objects::new();
    let test_object = TestObject::new();

    objects
      .insert(&test_object.get_unique_hash(), &test_object)
      .unwrap();

    (objects, test_object)
  }
}
