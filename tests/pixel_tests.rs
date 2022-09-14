use interactable_screen::screen::pixel::*;
use interactable_screen::screen::screen_data::*;

const OBJECT_1: &str = "Square";
const OBJECT_DISPLAY_1: &str = "x";

const OBJECT_2: &str = "Circle";
const OBJECT_DISPLAY_2: &str = "X";

const OBJECT_3: &str = "Line";
const OBJECT_DISPLAY_3: &str = "*";

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

    let [object_data, _, _] = generate_placeholder_objects();
    let expected_object_data = Some(object_data.clone());

    pixel_1.insert_object(object_data.0, object_data.1, true);

    let removed_displayed_object = pixel_1.remove_displayed_object();

    assert_eq!(removed_displayed_object, expected_object_data);
  }

  #[test]
  fn remove_with_two_objects() {
    let mut pixel_1 = Pixel::new();
    let [object_1, object_2, _] = generate_placeholder_objects();

    let expected_removed_data = Some(object_2.clone());
    let mut expected_pixel_1_data = Pixel::new();
    expected_pixel_1_data.insert_object(object_2.0.clone(), object_2.1.clone(), true);

    pixel_1.insert_object(object_1.0, object_1.1, true);
    pixel_1.insert_object(object_2.0, object_2.1, false);

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
    let [object, _, _] = generate_placeholder_objects();

    expected_pixel_data.insert_object(object.0.clone(), object.1.clone(), true);
    pixel.insert_object(object.0, object.1, false);

    pixel.reassign_display_data();

    assert_eq!(pixel, expected_pixel_data)
  }

  #[test]
  fn multiple_available_objects() {
    let mut pixel = Pixel::new();
    let mut expected_pixel_data = Pixel::new();
    let [object_1, object_2, _] = generate_placeholder_objects();

    expected_pixel_data.insert_object(object_2.0.clone(), object_2.1.clone(), true);
    expected_pixel_data.insert_object(object_1.0.clone(), object_1.1.clone(), false);
    pixel.insert_object(object_1.0, object_1.1, false);
    pixel.insert_object(object_2.0, object_2.1, false);

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

fn generate_placeholder_objects() -> [KeyAndObjectDisplay; 3] {
  [
    (OBJECT_1.to_string(), (0, OBJECT_DISPLAY_1.to_string())),
    (OBJECT_2.to_string(), (0, OBJECT_DISPLAY_2.to_string())),
    (OBJECT_3.to_string(), (0, OBJECT_DISPLAY_3.to_string())),
  ]
}
