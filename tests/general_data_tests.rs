use interactable_screen::general_data::{coordinates::*, map_methods::*};
use std::collections::{BTreeMap, HashMap};

#[test]
fn get_coordinates_in_between_logic() {
  let top_left = (0, 0);
  let bottom_right = (9, 9);
  let expected_amount_of_coordinates = 100;

  let in_between = &top_left.get_coordinates_in_between(&bottom_right);

  assert_eq!(in_between.len(), expected_amount_of_coordinates);
}

#[cfg(test)]
mod get_lowest_key_logic {
  use super::*;

  #[test]
  fn has_keys() {
    let mut map = HashMap::new();
    let expected_lowest = Some(&0);

    map.insert(2, "two");
    map.insert(0, "zero");
    map.insert(1, "one");

    let lowest_number = map.get_lowest_key();

    assert_eq!(lowest_number, expected_lowest);
  }

  #[test]
  fn has_no_keys() {
    let map: HashMap<u32, &str> = HashMap::new();
    let expected_lowest = None;

    let lowest_number = map.get_lowest_key();

    assert_eq!(lowest_number, expected_lowest);
  }
}
