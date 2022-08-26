use std::collections::{BTreeMap, HashMap};

pub trait BTreeMapMethods<K, V> {
  fn get_first_key(&self) -> Option<&K>;
}

impl<K, V> BTreeMapMethods<K, V> for BTreeMap<K, V> {
  fn get_first_key(&self) -> Option<&K> {
    self.keys().next()
  }
}

pub trait HashMapMethods<K, V> {
  fn get_lowest_key(&self) -> Option<&K>
  where
    K: Ord;
}

impl<K, V> HashMapMethods<K, V> for HashMap<K, V> {
  /// Gets the lowest valued key within a HashMap
  /// #Example
  /// ```
  /// use interactable_screen::general_data::map_methods::HashMapMethods;
  /// use std::collections::HashMap;
  ///
  /// let mut map = HashMap::new();
  ///
  /// map.insert(2, "2");
  /// map.insert(0, "0");
  /// map.insert(1, "1");
  ///
  /// assert_eq!(map.get_lowest_key(), Some(&0))
  /// ```
  fn get_lowest_key(&self) -> Option<&K>
  where
    K: Ord,
  {
    let keys = self.keys();
    let mut smallest_key = None;

    for key in keys {
      if let Some(small_key) = smallest_key {
        if small_key > key {
          smallest_key = Some(key)
        }
      } else {
        smallest_key = Some(key)
      }
    }

    smallest_key
  }
}
