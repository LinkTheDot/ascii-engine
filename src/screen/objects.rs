use crate::objects::object_data::*;
use std::collections::{hash_map::DefaultHasher, HashMap};

/// Object data will contain all object hashes and their
/// corresponding object types
// For now this will only contain the list of raw ObjectData.
// Once the Object trait is implemented this will hold a generic <O>.
// O will be any type that has the Object trait.
//
// The object trait will require a field of ObjectData.
// ObjectData will be basic things that the engine will need the object to hold.
// Said struct would contain data such as, object center point, sprite, hit box,
//   current_position, and more to be implemented.
#[allow(unused)]
pub struct Objects {
  objects: HashMap<DefaultHasher, Vec<ObjectData>>,
}

impl Objects {
  pub fn new() -> Self {
    Self {
      objects: HashMap::new(),
    }
  }
}
