use interactable_screen::screen::pixel::*;
use interactable_screen::screen::screen_data::*;

const OBJECT_1: &str = "Square";
const OBJECT_DISPLAY_1: &str = "x";

const OBJECT_2: &str = "Circle";
const OBJECT_DISPLAY_2: &str = "X";

const OBJECT_3: &str = "Line";
const OBJECT_DISPLAY_3: &str = "*";

const REASSIGN_DISPLAY: bool = true;

#[test]
fn generate_pixel_grid_logic() {
  let pixel_grid = generate_pixel_grid();
  let expected_grid_length = GRID_WIDTH * GRID_HEIGHT;

  assert_eq!(pixel_grid.len(), expected_grid_length);
}

#[cfg(test)]
mod remove_displayed_object_logic {
  use super::*;

  #[test]
  fn remove_with_one_object() {
    let mut pixel_1 = Pixel::new();
    let [object_data, _, _] = generate_placeholder_objects(None);

    let expected_object_data = Some(object_data.clone());

    pixel_1.insert_object(object_data.0, object_data.1, REASSIGN_DISPLAY);

    let removed_displayed_object = pixel_1.remove_displayed_object();

    assert_eq!(removed_displayed_object, expected_object_data);
  }

  #[test]
  fn remove_with_two_objects() {
    let mut pixel_1 = Pixel::new();
    let [object_1, object_2, _] = generate_placeholder_objects(None);

    let expected_removed_data = Some(object_1.clone());
    let mut expected_pixel_1_data = Pixel::new();
    expected_pixel_1_data.insert_object(object_2.0.clone(), object_2.1.clone(), true);

    pixel_1.insert_object(object_1.0, object_1.1, REASSIGN_DISPLAY);
    pixel_1.insert_object(object_2.0, object_2.1, !REASSIGN_DISPLAY);

    let removed_display_data = pixel_1.remove_displayed_object();

    assert_eq!(pixel_1, expected_pixel_1_data);
    assert_eq!(removed_display_data, expected_removed_data);
  }

  #[test]
  fn remove_with_no_objects() {
    let mut pixel_1 = Pixel::new();
    let expected_display_object = None;

    let removed_displayed_object = pixel_1.remove_displayed_object();

    assert_eq!(removed_displayed_object, expected_display_object);
  }
}

#[cfg(test)]
mod reassign_display_data_logic {
  use super::*;

  #[test]
  fn available_object() {
    let mut pixel = Pixel::new();
    let mut expected_pixel_data = Pixel::new();
    let [object, _, _] = generate_placeholder_objects(None);

    expected_pixel_data.insert_object(object.0.clone(), object.1.clone(), REASSIGN_DISPLAY);
    pixel.insert_object(object.0, object.1, !REASSIGN_DISPLAY);

    pixel.reassign_display_data();

    assert_eq!(pixel, expected_pixel_data)
  }

  #[test]
  fn multiple_available_objects() {
    let mut pixel = Pixel::new();
    let mut expected_pixel_data = Pixel::new();
    let [object_1, object_2, _] = generate_placeholder_objects(None);

    expected_pixel_data.insert_object(object_2.0.clone(), object_2.1.clone(), REASSIGN_DISPLAY);
    expected_pixel_data.insert_object(object_1.0.clone(), object_1.1.clone(), !REASSIGN_DISPLAY);
    pixel.insert_object(object_1.0, object_1.1, !REASSIGN_DISPLAY);
    pixel.insert_object(object_2.0, object_2.1, !REASSIGN_DISPLAY);

    pixel.reassign_display_data();

    assert_eq!(pixel, expected_pixel_data)
  }

  #[test]
  fn no_available_object() {
    let mut pixel = Pixel::new();
    let expected_pixel_data = Pixel::new();

    pixel.reassign_display_data();

    assert_eq!(pixel, expected_pixel_data)
  }
}

#[cfg(test)]
mod change_display_to_logic {
  use super::*;

  #[test]
  fn expected_data() {
    let mut pixel = Pixel::new();
    let [object_1, _, _] = generate_placeholder_objects(None);

    let expected_object = object_1.clone();
    let expected_assignment = Some((&expected_object.0, &expected_object.1 .0));

    // REASSIGN_DISPLAY will run 'change_display_to'
    pixel.insert_object(object_1.0, object_1.1, REASSIGN_DISPLAY);

    assert_eq!(pixel.get_both_assignments(), expected_assignment)
  }

  #[test]
  fn object_not_in_pixel() {
    let mut pixel = Pixel::new();
    let [object_1, _, _] = generate_placeholder_objects(None);

    let expected_assignment = None;

    pixel.change_display_to(Some(object_1.0), Some(object_1.1 .0));

    assert_eq!(pixel.get_both_assignments(), expected_assignment);
  }

  #[test]
  fn input_none() {
    let mut pixel = Pixel::new();

    let expected_assignment = None;

    pixel.change_display_to(None, None);

    assert_eq!(pixel.get_both_assignments(), expected_assignment);
  }
}

#[cfg(test)]
mod assigned_key_has_multiple_objects_logic {
  use super::*;

  #[test]
  fn does_have_multiple_objects() {
    let mut pixel = Pixel::new();
    let [object_1_0, _, _] = generate_placeholder_objects(None);
    let [object_1_1, _, _] = generate_placeholder_objects(Some(1));

    pixel.insert_object(object_1_0.0, object_1_0.1, true);
    pixel.insert_object(object_1_1.0, object_1_1.1, false);

    let outcome = pixel.assigned_key_has_multiple_objects();

    assert!(outcome);
  }

  #[test]
  fn doesnt_have_multiple_objects() {
    let mut pixel = Pixel::new();
    let [object_1, _, _] = generate_placeholder_objects(None);

    pixel.insert_object(object_1.0, object_1.1, false);

    let outcome = !pixel.assigned_key_has_multiple_objects();

    assert!(outcome);
  }
}

#[cfg(test)]
mod remove_object_assigned_number {
  use super::*;

  #[test]
  fn has_multiple_objects() {
    let mut pixel = Pixel::new();
    let [object_1_0, _, _] = generate_placeholder_objects(None);
    let [object_1_1, _, _] = generate_placeholder_objects(Some(1));

    let expected_removed_data = Some(object_1_1.1.clone());
    let mut expected_pixel_data = Pixel::new();
    expected_pixel_data.insert_object(object_1_0.0.clone(), object_1_0.1.clone(), true);

    pixel.insert_object(object_1_0.0, object_1_0.1, false);
    pixel.insert_object(object_1_1.0, object_1_1.1, true);

    let removed_data = pixel.remove_object_assigned_number(REASSIGN_DISPLAY);

    assert_eq!(removed_data, expected_removed_data);
    assert_eq!(pixel, expected_pixel_data);
  }

  #[test]
  fn has_one_object() {}
}

fn generate_placeholder_objects(assigned_number: Option<u32>) -> [KeyAndObjectDisplay; 3] {
  let assigned_number = match assigned_number {
    Some(num) => num,
    None => 0,
  };

  [
    (
      OBJECT_1.to_string(),
      (assigned_number, OBJECT_DISPLAY_1.to_string()),
    ),
    (
      OBJECT_2.to_string(),
      (assigned_number, OBJECT_DISPLAY_2.to_string()),
    ),
    (
      OBJECT_3.to_string(),
      (assigned_number, OBJECT_DISPLAY_3.to_string()),
    ),
  ]
}
