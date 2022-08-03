use interactable_screen::objects::{hollow_square::*, object_data::*};
use interactable_screen::screen::screen_data::*;

#[test]
fn create_an_object() {
  let square = Object::create_hollow_square(None);

  assert_eq!(square.object_shape, SQUARE_SHAPE)
}

#[test]
fn place_an_object() {
  let mut screen = ScreenData::new()
    .unwrap_or_else(|error| panic!("An error has occured while getting ScreenData: '{error}'"));
  let hollow_square = Object::create_hollow_square(Some((30, 15)));
  let hollow_square_0_0 = Object::create_hollow_square(None);
  let hollow_square_near_0_0 = Object::create_hollow_square(Some((5, 1)));

  hollow_square.place_object(&mut screen);
  hollow_square_0_0.place_object(&mut screen);
  hollow_square_near_0_0.place_object(&mut screen);

  println!("{}", screen.display());
}

#[test]
#[ignore]
fn move_a_placed_object() {}

#[cfg(test)]
mod get_object_sizes_logic {
  use super::*;

  #[test]
  fn get_object_width_logic() {
    let object_1d = "1234";
    let object_2d = "1234\n1234";
    let object_width_1d = get_object_width(object_1d);
    let object_width_2d = get_object_width(object_2d);

    assert_eq!(object_width_2d, object_width_1d);
  }

  #[test]
  fn get_object_height_logic() {
    let object_1d = "1234";
    let object_2d = "1234\n1234";
    let object_height_1d = get_object_height(object_1d);
    let object_height_2d = get_object_height(object_2d);

    let expected_1d_height = 1;
    let expected_2d_height = 2;

    assert_eq!(object_height_1d, expected_1d_height);
    assert_eq!(object_height_2d, expected_2d_height);
  }
}
