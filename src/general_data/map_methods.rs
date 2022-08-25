use crate::screen::screen_data::Key;
use std::collections::{BTreeMap, HashMap};

pub trait BTreeMapMethods<Key, V> {
  fn get_first_key(&self) -> Option<&Key>;
}

impl<Key, V> BTreeMapMethods<Key, V> for BTreeMap<Key, V> {
  fn get_first_key(&self) -> Option<&Key> {
    self.keys().next()
  }
}

trait HashMapMethods<Key, V> {}

impl<Key, V> HashMapMethods<Key, V> for BTreeMap<Key, V> {}
