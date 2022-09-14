use interactable_screen::screen::{object_screen_data::*, screen_data::*};
use std::collections::HashMap;

const OBJECT_1: &str = "Square";
const OBJECT_DISPLAY_1: &str = "x";

const OBJECT_2: &str = "Circle";
const OBJECT_DISPLAY_2: &str = "X";

const OBJECT_3: &str = "Line";
const OBJECT_DISPLAY_3: &str = "*";

fn generate_placeholder_objects() -> [KeyAndObjectDisplay; 3] {
  [
    (OBJECT_1.to_string(), (0, OBJECT_DISPLAY_1.to_string())),
    (OBJECT_2.to_string(), (0, OBJECT_DISPLAY_2.to_string())),
    (OBJECT_3.to_string(), (0, OBJECT_DISPLAY_3.to_string())),
  ]
}

#[test]
fn display_works() {
  let screen = ScreenData::default();
  let expected_screen_size = GRID_WIDTH * GRID_HEIGHT + GRID_HEIGHT;
  let display = screen.display();

  println!("{}", &display);

  assert_eq!(display.len(), expected_screen_size);
}

#[test]
fn adding_multiple_items_then_moving_one() {
  let mut screen = ScreenData::default();
  let pixel_one = (0, 0);
  let pixel_two = (1, 0);
  let [data_one, data_two, _] = generate_placeholder_objects();
  let expected_origin_pixel_data = data_one.1 .1.clone();
  let expected_new_pixel_data = data_two.1 .1.clone();

  screen.insert_object_at(&pixel_one, data_one, true);
  screen.insert_object_at(&pixel_one, data_two, true);

  println!("{:?} - {:?}", pixel_one, screen.get_pixel_at(&pixel_one));

  screen.transfer_assigned_object_in_pixel_to(&pixel_one, &pixel_two);

  println!("{:?} - {:?}", pixel_one, screen.get_pixel_at(&pixel_one));
  println!("{:?} - {:?}", pixel_two, screen.get_pixel_at(&pixel_two));

  let pixel_one_data = screen
    .get_pixel_at(&pixel_one)
    .get_all_current_display_data()
    .unwrap()
    .get(&0)
    .unwrap();

  let pixel_two_data = screen
    .get_pixel_at(&pixel_two)
    .get_all_current_display_data()
    .unwrap()
    .get(&0)
    .unwrap();

  assert_eq!(pixel_one_data, &expected_origin_pixel_data);
  assert_eq!(pixel_two_data, &expected_new_pixel_data);
}

#[test]
fn transferring_same_object_names() {
  let mut screen = ScreenData::default();
  let pixel_one = (0, 0);
  let pixel_two = (1, 0);
  let data_one = (OBJECT_1.to_string(), (0, OBJECT_DISPLAY_1.to_string()));
  let data_two = (OBJECT_1.to_string(), (1, OBJECT_DISPLAY_2.to_string()));
  let expected_origin_pixel_data = data_one.1 .1.clone();
  let expected_new_pixel_data = data_two.1 .1.clone();

  screen.insert_object_at(&pixel_one, data_one, true);
  screen.insert_object_at(&pixel_one, data_two, true);

  println!("{:?} - {:?}", pixel_one, screen.get_pixel_at(&pixel_one));

  screen.transfer_assigned_object_in_pixel_to(&pixel_one, &pixel_two);

  println!("{:?} - {:?}", pixel_one, screen.get_pixel_at(&pixel_one));
  println!("{:?} - {:?}", pixel_two, screen.get_pixel_at(&pixel_two));

  let pixel_one_data = screen
    .get_pixel_at(&pixel_one)
    .get_all_current_display_data()
    .unwrap()
    .get(&0)
    .unwrap();

  let pixel_two_data = screen
    .get_pixel_at(&pixel_two)
    .get_all_current_display_data()
    .unwrap()
    .get(&1)
    .unwrap();

  assert_eq!(pixel_one_data, &expected_origin_pixel_data);
  assert_eq!(pixel_two_data, &expected_new_pixel_data);
}

#[cfg(test)]
mod pixel_data_transfer_logic {
  use super::*;

  #[test]
  fn move_oldest_object_in_pixel_logic() {
    let mut screen = ScreenData::default();
    let [object_1_data, object_2_data, _] = generate_placeholder_objects();

    let first_pixel = (0, 0);
    let second_pixel = (1, 0);

    let expected_first_pixel_data: Option<AssignedObjects> = None;
    let mut expected_second_pixel_data = HashMap::new();
    let expected_pixel_2_data = Some(object_2_data.clone());

    expected_second_pixel_data.insert(object_1_data.1 .0, object_1_data.1 .1.clone());

    screen.insert_object_at(&first_pixel, object_1_data, true);
    screen.insert_object_at(&second_pixel, object_2_data, true);

    let pixel_2_data = screen.replace_latest_object_in_pixel(&first_pixel, &second_pixel);

    // pixel one check
    assert_eq!(
      expected_first_pixel_data.as_ref(),
      screen
        .get_pixel_at(&first_pixel)
        .get_all_current_display_data()
    );

    // pixel two check
    assert_eq!(
      Some(&expected_second_pixel_data),
      screen
        .get_pixel_at(&second_pixel)
        .get_all_current_display_data(),
    );

    // removed pixel 2 data check
    assert_eq!(&expected_pixel_2_data, &pixel_2_data)
  }
}
