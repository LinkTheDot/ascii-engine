use interactable_screen::screen::screen_data::*;

#[test]
fn generate_pixel_grid_logic() {
  let pixel_grid = generate_pixel_grid();
  let expected_grid_length = GRID_WIDTH * GRID_HEIGHT;

  assert_eq!(pixel_grid.len(), expected_grid_length);
}

#[test]
fn display_works() {
  let screen = ScreenData::new()
    .unwrap_or_else(|error| panic!("An error has occured while grabbing ScreenData: '{error}'"));
  let expected_screen_size = GRID_WIDTH * GRID_HEIGHT + GRID_HEIGHT;
  let display = screen.display();

  println!("{}", &display);

  assert_eq!(display.len(), expected_screen_size);
}

#[cfg(test)]
mod pixel_data_transfer_logic {
  use super::*;

  const OBJECT_1: &str = "Square";
  const OBJECT_DISPLAY_1: &str = "x";

  const OBJECT_2: &str = "Circle";
  const OBJECT_DISPLAY_2: &str = "X";

  const OBJECT_3: &str = "Line";
  const OBJECT_DISPLAY_3: &str = "*";

  #[test]
  fn move_oldest_object_in_pixel_logic() {
    let mut screen = ScreenData::default();
    let (object_1_data, object_2_data, _x) = generate_all_objects();

    let first_pixel = (0, 0);
    let second_pixel = (1, 0);

    let expected_first_pixel_data = vec![];
    let expected_second_pixel_data = vec![object_1_data.clone()];
    let expected_pixel_2_data = object_2_data.clone();

    screen.insert_object_at(&first_pixel, &object_1_data);
    screen.insert_object_at(&second_pixel, &object_2_data);

    let pixel_2_data = screen.replace_latest_object_in_pixel(&first_pixel, &second_pixel);

    assert_eq!(
      (
        &screen.get_pixel_at(&first_pixel).objects_within,
        &screen.get_pixel_at(&second_pixel).objects_within,
        pixel_2_data
      ),
      (
        &expected_first_pixel_data,
        &expected_second_pixel_data,
        Some(expected_pixel_2_data)
      )
    )
  }

  #[test]
  fn move_all_objects_in_pixel_logic() {
    let mut screen = ScreenData::default();
    let (object_1_data, object_2_data, object_3_data) = generate_all_objects();

    let first_pixel = (0, 0);
    let second_pixel = (1, 0);

    let expected_first_pixel_data = vec![];
    let expected_second_pixel_data = vec![object_1_data.clone(), object_2_data.clone()];
    let expected_pixel_2_data = vec![object_3_data.clone()];

    screen.insert_object_at(&first_pixel, &object_1_data);
    screen.insert_object_at(&first_pixel, &object_2_data);
    screen.insert_object_at(&second_pixel, &object_3_data);

    let pixel_2_data = screen.replace_all_objects_in_pixel(&first_pixel, &second_pixel);

    assert_eq!(
      (
        &screen.get_pixel_at(&first_pixel).objects_within,
        &screen.get_pixel_at(&second_pixel).objects_within,
        pixel_2_data
      ),
      (
        &expected_first_pixel_data,
        &expected_second_pixel_data,
        expected_pixel_2_data
      )
    )
  }

  fn generate_all_objects() -> (ObjectAndDisplay, ObjectAndDisplay, ObjectAndDisplay) {
    (
      (OBJECT_1.to_string(), OBJECT_DISPLAY_1.to_string()),
      (OBJECT_2.to_string(), OBJECT_DISPLAY_2.to_string()),
      (OBJECT_3.to_string(), OBJECT_DISPLAY_3.to_string()),
    )
  }
}
