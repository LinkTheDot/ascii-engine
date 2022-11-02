#![allow(unused_imports)]

use interactable_screen::screen::pixel::*;
use interactable_screen::screen::screen_data::{Key, KeyAndObjectDisplay, ObjectDisplay};
use std::collections::{BTreeMap, HashMap};

/*

- Test Structure -

fn test_name() {
 // make pixel(s)

 // make object(s)

 // make expected data

 // work with pixel(s) and object(s)

 // get the resulting data from the pixels

 // assert the data with the expected data
}

- Types -

 - The object's name for the BTreeMap
Key: String

 - The object's individual pixel appearance
ObjectDisplay: String

 - An objects assigned number,
   this is based on how many of this object exists.
   The object's number is based on the order in which it was created,
   meaning, the 5th object has an assigned number of '4'.
AssignedNumber: u32

 - An assigned object, as in, an object with it's assigned number
AssignedObject: (AssignedNumber, ObjectDisplay)

 - What goes into the BTreeMap
   This is the map for a group of assigned numbers and their ObjectDisplay
AssignedObjects: Hashmap<AssignedNumber, ObjectDisplay>
*/

pub const OBJECT_1_NAME: &str = "O1";
pub const OBJECT_1_DISPLAY: &str = "a";

pub const OBJECT_2_NAME: &str = "O2";
pub const OBJECT_2_DISPLAY: &str = "b";

pub const OBJECT_3_NAME: &str = "O3";
pub const OBJECT_3_DISPLAY: &str = "c";

#[cfg(test)]
mod insert_object_logic {
  use super::*;

  #[test]
  fn no_existing_objects() {
    let mut pixel = Pixel::new();
    let [object, _, _] = get_object_list(0);

    let (key, assigned_objects) = convert_object_for_assertion(object.clone());
    let expected_data = (&key, &assigned_objects);

    pixel.insert_object(object.0, object.1, false);

    let data = pixel.get_all_data()[0];

    assert_eq!(data, expected_data);
  }

  #[test]
  fn existing_object_same_name() {
    let mut pixel = Pixel::new();

    let [object0, _, _] = get_object_list(0);
    let [object1, _, _] = get_object_list(1);

    let merged_objects = merge_objects(vec![object0.clone(), object1.clone()]);
    let (key, assigned_objects) = merged_objects[0].to_owned();
    let expected_data = (&key, &assigned_objects);

    pixel.insert_object(object0.0, object0.1, false);
    pixel.insert_object(object1.0, object1.1, false);

    let data = pixel.get_all_data()[0];

    assert_eq!(data, expected_data);
  }

  #[test]
  fn existing_object_different_name() {
    let mut pixel = Pixel::new();

    let [object0, object1, _] = get_object_list(0);

    let (key0, assigned_objects0) = convert_object_for_assertion(object0.clone());
    let (key1, assigned_objects1) = convert_object_for_assertion(object1.clone());
    let object0_data = (&key0, &assigned_objects0);
    let object1_data = (&key1, &assigned_objects1);
    let expected_data = vec![object0_data, object1_data];

    pixel.insert_object(object0.0, object0.1, false);
    pixel.insert_object(object1.0, object1.1, false);

    let data = pixel.get_all_data();

    assert_eq!(expected_data, data);
  }
}

// #[cfg(test)]
// mod assign_display_data {
// use super::*;

// #[test]
// fn no_existing_display_data() {
// let mut pixel = Pixel::new();

// let [object0, _, _] = get_object_list(0);
// }
// }

pub fn get_object_list(number_assignment: u32) -> [KeyAndObjectDisplay; 3] {
  [
    (
      OBJECT_1_NAME.to_string(),
      (number_assignment, OBJECT_1_DISPLAY.to_string()),
    ),
    (
      OBJECT_2_NAME.to_string(),
      (number_assignment, OBJECT_2_DISPLAY.to_string()),
    ),
    (
      OBJECT_3_NAME.to_string(),
      (number_assignment, OBJECT_3_DISPLAY.to_string()),
    ),
  ]
}

pub fn convert_object_for_assertion(object: KeyAndObjectDisplay) -> (Key, AssignedObjects) {
  let mut assigned_object = HashMap::new();
  assigned_object.insert(object.1 .0, object.1 .1);

  (object.0, assigned_object)
}

pub fn merge_objects(objects: Vec<KeyAndObjectDisplay>) -> Vec<(Key, AssignedObjects)> {
  let mut data_set = vec![];
  let mut known_keys: Vec<Key> = vec![];

  for object in objects {
    // these variable are for better readability in the code
    let object_key = object.0;
    let object_data = object.1;
    let object_assigned_number = object_data.0;
    let object_display = object_data.1;

    let existing_key: Vec<&Key> = known_keys
      .iter()
      .filter(|key| *key == &object_key)
      .collect();

    if !existing_key.is_empty() {
      if let Some(matching_map) = get_map_from_set(&mut data_set, &object_key) {
        matching_map.insert(object_assigned_number, object_display);
      }
    } else {
      known_keys.push(object_key.clone());

      let mut new_assigned_object_map = HashMap::new();
      new_assigned_object_map.insert(object_assigned_number, object_display);

      data_set.push((object_key, new_assigned_object_map));
    }
  }

  data_set
}

/// Gets a mutable reference to the first hashmap from a vector of keys and hashmaps
fn get_map_from_set<'a>(
  data_set: &'a mut Vec<(Key, AssignedObjects)>,
  key: &Key,
) -> Option<&'a mut AssignedObjects> {
  data_set
    .iter_mut()
    .find_map(|data| (&data.0 == key).then(|| &mut data.1))
}
