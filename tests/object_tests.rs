// make a way to iterate through all of the cube and compare
// the data with what should be there

use interactable_screen::objects::{hollow_square::*, object_data::*, object_movements::*};
use interactable_screen::screen::screen_data::*;

#[test]
fn create_an_object() {
  let mut screen = ScreenData::default();
  let square = Object::create_hollow_square(&mut screen, None);

  assert_eq!(square.object_shape, SQUARE_SHAPE)
}

#[test]
fn place_an_object() {
  let mut screen = ScreenData::default();
  let hollow_square = Object::create_hollow_square(&mut screen, Some((30, 15)));
  let hollow_square_0_0 = Object::create_hollow_square(&mut screen, None);
  let hollow_square_near_0_0 = Object::create_hollow_square(&mut screen, Some((5, 1)));

  hollow_square.place_object(&mut screen);
  hollow_square_0_0.place_object(&mut screen);
  hollow_square_near_0_0.place_object(&mut screen);

  println!("{}", screen.display());
}

#[cfg(test)]
mod movements {
  use super::*;

  #[test]
  #[ignore]
  fn move_a_placed_object() {
    let mut screen = ScreenData::default();
    let mut hollow_square = Object::create_hollow_square(&mut screen, Some((2, 2)));

    hollow_square.place_object(&mut screen);

    hollow_square.move_object(&mut screen, &ObjectMovements::Up);
    hollow_square.move_object(&mut screen, &ObjectMovements::Down);
    hollow_square.move_object(&mut screen, &ObjectMovements::Left);
    hollow_square.move_object(&mut screen, &ObjectMovements::Right);
  }

  #[cfg(test)]
  mod move_object_out_of_bounds {
    use super::*;

    #[test]
    fn move_up() {
      let mut screen = ScreenData::default();
      let mut hollow_square = Object::create_hollow_square(&mut screen, Some((0, 0)));
      let expected_position = (0, 0);

      hollow_square.move_object(&mut screen, &ObjectMovements::Up);

      assert_eq!(hollow_square.position, expected_position);
    }

    // doesn't work
    #[test]
    #[ignore]
    fn move_down() {
      let mut screen = ScreenData::default();
      // for now just manually put the number in
      // later once the function is made place it in
      // the expected position then move it
      let mut hollow_square = Object::create_hollow_square(&mut screen, Some((165, 35)));
      let expected_position = (
        GRID_WIDTH - hollow_square.width,
        GRID_HEIGHT - hollow_square.height,
      );

      hollow_square.move_object(&mut screen, &ObjectMovements::Down);

      assert_eq!(hollow_square.position, expected_position);
    }

    #[test]
    fn move_left() {
      let mut screen = ScreenData::default();
      let mut hollow_square = Object::create_hollow_square(&mut screen, Some((0, 0)));
      let expected_position = (0, 0);

      hollow_square.move_object(&mut screen, &ObjectMovements::Left);

      assert_eq!(hollow_square.position, expected_position);
    }

    // doesn't work
    #[test]
    #[ignore]
    fn move_right() {
      let mut screen = ScreenData::default();
      // for now just manually put the number in
      // later once the function is made place it in
      // the expected position then move it
      let mut hollow_square = Object::create_hollow_square(&mut screen, Some((165, 35)));
      let expected_position = (
        GRID_WIDTH - hollow_square.width,
        GRID_HEIGHT - hollow_square.height,
      );

      hollow_square.move_object(&mut screen, &ObjectMovements::Down);

      assert_eq!(hollow_square.position, expected_position);
    }
  }
}

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

#[test]
fn get_bottom_right_of_object_logic() {
  let mut screen = ScreenData::default();

  let x_coordinate = 5;
  let y_coordinate = 5;

  let hollow_square = Object::create_hollow_square(&mut screen, Some((x_coordinate, y_coordinate)));
  let bottom_right = hollow_square.get_bottom_right_of_object();
  let expected_coords = (
    x_coordinate + hollow_square.width - 1,
    y_coordinate + hollow_square.height - 1,
  );

  assert_eq!(bottom_right, expected_coords)
}
