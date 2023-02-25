use crate::errors::*;
use crate::objects::object_data::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Object data will contain all object hashes and their
/// corresponding object types
// Might need to just store a reference to the object data in an
// object instead of the object itself.
// On top of that it might have to be an Arc<Mutex<ObjectData>> in both
// the object and this Objects struct
pub struct Objects {
  objects: HashMap<Strata, HashMap<u64, Arc<Mutex<ObjectData>>>>,
}

impl Objects {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    Self {
      objects: HashMap::new(),
    }
  }

  pub fn insert<O: Object>(&mut self, key: u64, object: &O) -> Result<(), ObjectError> {
    let object_strata = object.get_strata();

    if let Some(strata_objects) = self.objects.get_mut(&object_strata) {
      strata_objects.insert(key, object.get_object_data());
    } else if object_strata.correct_range() {
      self.objects.insert(
        object_strata,
        HashMap::from([(key, object.get_object_data())]),
      );
    } else {
      return Err(ObjectError::IncorrectStrataRange(object_strata));
    }

    Ok(())
  }

  /// Returns a reference to the objects corresponding to the given Strata level.
  pub fn get(&self, key: &Strata) -> Option<&HashMap<u64, Arc<Mutex<ObjectData>>>> {
    self.objects.get(key)
  }
}
