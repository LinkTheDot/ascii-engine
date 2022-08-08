use crate::general_data::coordinates::*;
use crate::objects::object_data::*;
use crate::screen::screen_data::*;

pub const SQUARE_SHAPE: &str = // /
  "vvxX  Xxcc
xxX    Xxx
xX      Xx
xxX    Xxx
CCxX  XxVV";

pub trait HollowSquare {
  fn create_hollow_square(position: Option<Coordinates>) -> Object;

  fn move_object(&mut self, screen_data: &mut ScreenData, move_to: ObjectMovements);
}

impl HollowSquare for Object {
  fn create_hollow_square(position: Option<Coordinates>) -> Object {
    Object::create("square", SQUARE_SHAPE, position)
  }

  // latest thing being worked on
  //
  // doesn't actually take the data
  fn move_object(&mut self, screen_data: &mut ScreenData, move_to: ObjectMovements) {
    let bottom_right_of_square = self.get_bottom_right_of_object();
    let mut coordinate_cube = self
      .position
      .get_coordinates_in_between(&bottom_right_of_square);

    self.position = self.position.move_coords(&move_to).unwrap();

    match move_to {
      ObjectMovements::Up | ObjectMovements::Left => {
        move_cube(screen_data, coordinate_cube, move_to);
      }
      ObjectMovements::Down | ObjectMovements::Right => {
        coordinate_cube.reverse();

        move_cube(screen_data, coordinate_cube, move_to);
      }
    }
  }
}

fn move_cube(
  screen_data: &mut ScreenData,
  coordinate_cube: Vec<Coordinates>,
  move_to: ObjectMovements,
) {
  for pixel_coords in coordinate_cube {
    let swap_with = pixel_coords.move_coords(&move_to);

    // this is a temporary slapped on fix but in the future just using
    // if let will cause bugs, aka distorting the object in certain
    // directions and then just suddenly stopping the movement
    if let Some(swap_with) = swap_with {
      screen_data.transfer_latest_object_in_pixel_to(&pixel_coords, &swap_with);
      screen_data.change_pixel_display_at(&swap_with, "square".to_string());
    } else {
      return;
    }
  }
}
