use rand::prelude::*;
use rand::rngs::OsRng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Generates a unique hash
pub fn get_unique_hash() -> u64 {
  let mut seed = vec![0; 16];
  OsRng.fill_bytes(&mut seed);

  let mut hasher = DefaultHasher::new();
  seed.hash(&mut hasher);

  hasher.finish()
}
