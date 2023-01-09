use crate::objects::{errors::ObjectError, object_data::*};
use std::collections::HashMap;

/// Object data will contain all object hashes and their
/// corresponding object types
// Might need to just store a reference to the object data in an
// object instead of the object itself.
// On top of that it might have to be an Arc<Mutex<ObjectData>> in both
// the object and this Objects struct
pub struct Objects<'a, O: Object> {
  objects: HashMap<Strata, HashMap<u64, &'a mut O>>,
}

impl<'a, O> Objects<'a, O>
where
  O: Object,
{
  pub fn new() -> Self {
    Self {
      objects: HashMap::new(),
    }
  }

  pub fn insert(&mut self, key: u64, object: &'a mut O) -> Result<(), ObjectError> {
    let object_strata = *object.get_strata();

    if let Some(strata_objects) = self.objects.get_mut(&object_strata) {
      strata_objects.insert(key, object);
    } else if object_strata.correct_range() {
      self
        .objects
        .insert(object_strata, HashMap::from([(key, object)]));
    } else {
      // This error is probably unreachable.
      return Err(ObjectError::IncorrectStrataRange(object_strata));
    }

    Ok(())
  }

  pub fn get(&self, key: &Strata) -> Option<&HashMap<u64, &'a mut O>> {
    self.objects.get(key)
  }
}
