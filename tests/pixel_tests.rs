use ascii_engine::screen::pixel_data_types::*;
use ascii_engine::screen::screen_data::EMPTY_PIXEL;
use ascii_engine::screen::{pixel, pixel::*};
use std::collections::HashMap;

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
    let mut pixel = Pixel::default();
    let [object, _, _] = get_object_list(0);

    let (key, assigned_objects) = convert_object_for_assertion(object.clone());
    let expected_data = (&key, &assigned_objects);

    pixel.insert_object(object.0, object.1, pixel::Reassign::False);

    let data = pixel.get_all_data()[0];

    assert_eq!(data, expected_data);
  }

  #[test]
  fn existing_object_same_name() {
    let mut pixel = Pixel::default();

    let [object0, _, _] = get_object_list(0);
    let [object1, _, _] = get_object_list(1);

    let merged_objects = merge_objects(vec![object0.clone(), object1.clone()]);
    let (key, assigned_objects) = merged_objects[0].to_owned();
    let expected_data = (&key, &assigned_objects);

    pixel.insert_object(object0.0, object0.1, pixel::Reassign::False);
    pixel.insert_object(object1.0, object1.1, pixel::Reassign::False);

    let data = pixel.get_all_data()[0];

    assert_eq!(data, expected_data);
  }

  #[test]
  fn existing_object_different_name() {
    let mut pixel = Pixel::default();

    let [object0, object1, _] = get_object_list(0);

    let (key0, assigned_objects0) = convert_object_for_assertion(object0.clone());
    let (key1, assigned_objects1) = convert_object_for_assertion(object1.clone());
    let object0_data = (&key0, &assigned_objects0);
    let object1_data = (&key1, &assigned_objects1);
    let expected_data = vec![object0_data, object1_data];

    pixel.insert_object(object0.0, object0.1, pixel::Reassign::False);
    pixel.insert_object(object1.0, object1.1, pixel::Reassign::False);

    let data = pixel.get_all_data();

    assert_eq!(expected_data, data);
  }
}

#[cfg(test)]
mod display_tests {
  use super::*;

  #[test]
  fn has_assigned_objects() {
    let mut pixel = Pixel::default();
    let [object, _, _] = get_object_list(0);

    let expected_display = object.1 .1.clone();

    pixel.insert_object(object.0, object.1, pixel::Reassign::True);

    let pixel_display = format!("{}", pixel);

    assert_eq!(pixel_display, expected_display);
  }

  #[test]
  fn has_assigned_objects_and_no_data() {
    let pixel = Pixel::default();

    let expected_display = EMPTY_PIXEL;

    let pixel_display = format!("{}", pixel);

    assert_eq!(pixel_display, expected_display);
  }

  #[test]
  fn has_assigned_objects_but_contains_data() {
    let mut pixel = Pixel::default();
    let [object, _, _] = get_object_list(0);

    let expected_display = EMPTY_PIXEL;

    pixel.insert_object(object.0, object.1, pixel::Reassign::False);

    let pixel_display = format!("{}", pixel);

    assert_eq!(pixel_display, expected_display);

    println!("{:#?}", pixel);
  }
}

#[cfg(test)]
mod change_display_to {
  use super::*;

  #[test]
  fn object_exists() {
    let mut pixel = Pixel::default();
    let [object, _, _] = get_object_list(0);

    let (key, number) = (object.0.clone(), object.1 .0);
    let expected_assignments = (Some(&key), Some(&number));

    // change_display_to() is called when Reassign::True is passed in
    pixel.insert_object(object.0, object.1, pixel::Reassign::True);

    let current_assignments = pixel.get_both_assignments();

    assert_eq!(current_assignments, expected_assignments);
  }

  #[test]
  fn object_does_not_exist() {
    let mut pixel = Pixel::default();
    let [object, _, _] = get_object_list(0);

    let expected_assignments = (None, None);

    let result = pixel.change_display_to(object.0, object.1 .0);

    let current_assignments = pixel.get_both_assignments();

    assert_eq!(current_assignments, expected_assignments);
    assert!(result.is_err());
  }
}

#[test]
fn clear_display_data() {
  let mut pixel = Pixel::default();
  let [object, _, _] = get_object_list(0);

  pixel.insert_object(object.0, object.1, pixel::Reassign::True);
  let (assigned_key, assigned_number) = pixel.get_both_assignments();

  assert!(assigned_key.is_some() && assigned_number.is_some());

  pixel.clear_display_data();
  let (cleared_key, cleared_number) = pixel.get_both_assignments();

  assert!(cleared_key.is_none() && cleared_number.is_none());
}

#[cfg(test)]
mod remove_displayed_object {
  use super::*;

  #[test]
  fn pixel_has_valid_display_data() {
    let mut pixel = Pixel::default();
    let [object, _, _] = get_object_list(0);

    let expected_removed_data = Some(object.clone());
    let expected_assigned_data = (None, None);

    pixel.insert_object(object.0, object.1, pixel::Reassign::True);

    let removed_data = pixel.remove_displayed_object(pixel::Reassign::False);
    let assigned_data = pixel.clone_both_assignments();

    assert_eq!(removed_data, expected_removed_data);
    assert_eq!(assigned_data, expected_assigned_data);
  }

  #[test]
  fn pixel_has_no_display_data() {
    let mut pixel = Pixel::default();

    let expected_removed_data = None;
    let expected_assigned_data = (None, None);

    let removed_data = pixel.remove_displayed_object(pixel::Reassign::False);
    let assigned_data = pixel.clone_both_assignments();

    assert_eq!(removed_data, expected_removed_data);
    assert_eq!(assigned_data, expected_assigned_data);
  }
}

#[cfg(test)]
mod remove_object {
  use super::*;

  #[test]
  fn pixel_contains_one_of_object() {
    let mut pixel = Pixel::default();
    let [object, _, _] = get_object_list(0);

    let expected_data = Some(object.clone());

    pixel.insert_object(object.0.clone(), object.1.clone(), pixel::Reassign::False);

    let removed_data = pixel.remove_object(&object.0, &object.1 .0, pixel::Reassign::False);

    assert_eq!(expected_data, removed_data);
  }

  #[test]
  fn pixel_contains_multiple_of_object() {
    let mut pixel = Pixel::default();
    let [object1, _, _] = get_object_list(0);
    let [object2, _, _] = get_object_list(1);

    let expected_data = Some(object1.clone());

    pixel.insert_object(object1.0.clone(), object1.1.clone(), pixel::Reassign::False);
    pixel.insert_object(object2.0, object2.1, pixel::Reassign::False);

    let removed_data = pixel.remove_object(&object1.0, &object1.1 .0, pixel::Reassign::False);

    assert_eq!(expected_data, removed_data);
  }

  #[test]
  fn pixel_does_not_contain_object() {
    let mut pixel = Pixel::default();
    let [object, _, _] = get_object_list(0);

    let expected_data = None;

    let removed_data = pixel.remove_object(&object.0, &object.1 .0, pixel::Reassign::False);

    assert_eq!(expected_data, removed_data);
  }

  #[cfg(test)]
  mod reassign_logic {
    use super::*;

    #[test]
    fn reassign_true_data_exists() {
      let mut pixel = Pixel::default();
      let [object1, object2, _] = get_object_list(0);

      let (key, assigned_number) = (object2.0.clone(), object2.1 .0);
      let expected_assignments = (Some(key), Some(assigned_number));

      pixel.insert_object(object1.0.clone(), object1.1.clone(), pixel::Reassign::False);
      pixel.insert_object(object2.0, object2.1, pixel::Reassign::False);

      let _ = pixel.remove_object(&object1.0, &object1.1 .0, pixel::Reassign::True);

      let current_assignments = pixel.take_both_assignments();

      assert_eq!(expected_assignments, current_assignments);
    }

    #[test]
    fn reassign_true_data_doesnt_exist() {
      let mut pixel = Pixel::default();
      let [object1, _, _] = get_object_list(0);

      let expected_assignments = (None, None);

      pixel.insert_object(object1.0.clone(), object1.1.clone(), pixel::Reassign::False);

      let _ = pixel.remove_object(&object1.0, &object1.1 .0, pixel::Reassign::True);

      let current_assignments = pixel.take_both_assignments();

      assert_eq!(expected_assignments, current_assignments);
    }

    #[test]
    fn reassign_false_data_exists() {
      let mut pixel = Pixel::default();
      let [object1, object2, _] = get_object_list(0);

      let expected_assignments = (None, None);

      pixel.insert_object(object1.0.clone(), object1.1.clone(), pixel::Reassign::False);
      pixel.insert_object(object2.0, object2.1, pixel::Reassign::False);

      let _ = pixel.remove_object(&object1.0, &object1.1 .0, pixel::Reassign::False);

      let current_assignments = pixel.take_both_assignments();

      assert_eq!(expected_assignments, current_assignments);
    }

    #[test]
    fn reassign_false_data_doesnt_exist() {
      let mut pixel = Pixel::default();
      let [object1, _, _] = get_object_list(0);

      let expected_assignments = (None, None);

      pixel.insert_object(object1.0.clone(), object1.1.clone(), pixel::Reassign::False);

      let _ = pixel.remove_object(&object1.0, &object1.1 .0, pixel::Reassign::False);

      let current_assignments = pixel.take_both_assignments();

      assert_eq!(expected_assignments, current_assignments);
    }
  }
}

#[cfg(test)]
mod get_current_display_data {
  use super::*;

  #[test]
  #[ignore]
  fn data_exists_has_assignment() {
    let mut pixel = Pixel::default();
    let [object, _, _] = get_object_list(0);

    let (key, object_display) = (object.0.clone(), object.1.clone());
    let expected_display_data = Some((&key, &object_display));

    pixel.insert_object(object.0, object.1, pixel::Reassign::True);

    let current_display_data = pixel.get_current_display_data();

    println!("{:?}\n\n{:?}", expected_display_data, current_display_data);

    // assert_eq!(expected_display_data, current_display_data);
  }

  #[test]
  fn data_doesnt_exist_has_assignment() {}

  #[test]
  fn data_exists_has_no_assignment() {}

  #[test]
  fn data_doesnt_exist_has_no_assignment() {}
}

pub fn get_object_list(number_assignment: u32) -> [KeyAndObjectDisplay; 3] {
  [
    (
      OBJECT_1_NAME.to_string(),
      (number_assignment, OBJECT_1_DISPLAY.to_string()),
    ),
    (
      OBJECT_3_NAME.to_string(),
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

    let has_existing_key = known_keys.iter().any(|key| key == &object_key);

    if has_existing_key {
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
  data_set: &'a mut [(Key, AssignedObjects)],
  key: &Key,
) -> Option<&'a mut AssignedObjects> {
  data_set
    .iter_mut()
    .find_map(|data| (&data.0 == key).then(|| &mut data.1))
}
