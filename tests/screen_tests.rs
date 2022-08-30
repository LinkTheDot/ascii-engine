use interactable_screen::screen::screen_data::*;

const OBJECT_1: &str = "Square";
const OBJECT_DISPLAY_1: &str = "x";

const OBJECT_2: &str = "Circle";
const OBJECT_DISPLAY_2: &str = "X";

const OBJECT_3: &str = "Line";
const OBJECT_DISPLAY_3: &str = "*";

fn generate_all_objects() -> [KeyAndObjectDisplay; 3] {
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
  let [data_one, data_two, _] = generate_all_objects();
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
