// add a new movement function that'll take every item of the object
// and move it to another position
//
// i can already see this having many problems with how
// object data is stored in pixels
// see screen_data for more of an explanation

use crate::general_data::{coordinates::*, user_input};
use crate::objects::{object_data::*, object_movements::*};
use crate::screen::screen_data::*;

/// in ticks aka 'self * 16ms'
const BASIC_MOVEMENT_TIMER: u32 = 6;
const HORIZONTAL_UNIT: u16 = 2;
const VERTICAL_UNIT: u16 = 1;
pub const SQUARE_SHAPE: &str = // /
  "vvxX  Xxcc
xxX    Xxx
xX      Xx
xxX    Xxx
CCxX  XxVV";

pub trait HollowSquare {
  fn create_hollow_square(screen_data: &mut ScreenData, position: Option<Coordinates>) -> Object;

  fn move_object(&mut self, screen_data: &mut ScreenData, move_to: &ObjectMovements);
  fn move_x_times(&mut self, screen_data: &mut ScreenData, move_to: ObjectMovements, x: u16);

  fn spin_cube(&mut self, screen_data: &mut ScreenData, spin_count: usize);
  fn user_move_cube(&mut self, screen_data: &mut ScreenData);
}

impl HollowSquare for Object {
  fn create_hollow_square(screen_data: &mut ScreenData, position: Option<Coordinates>) -> Object {
    let object_information = ObjectInformation::from("square", SQUARE_SHAPE, position, None);

    Object::create(object_information, screen_data)
  }

  fn move_object(&mut self, screen_data: &mut ScreenData, move_to: &ObjectMovements) {
    let new_coords = match self.position.move_coords(move_to) {
      Some(coords) => coords,
      None => return,
    };

    if new_coords
      .get_object_bounds(move_to, self.width, self.height)
      .is_none()
    {
      return;
    }

    let bottom_right_of_square = self.get_bottom_right_of_object();
    let mut coordinate_cube = self
      .position
      .get_coordinates_in_between(&bottom_right_of_square);

    self.position = new_coords;

    if move_to.moves_in_negative_direction() {
      move_cube(screen_data, coordinate_cube, move_to);
    } else {
      coordinate_cube.reverse();

      move_cube(screen_data, coordinate_cube, move_to);
    }
  }

  fn move_x_times(&mut self, screen_data: &mut ScreenData, move_to: ObjectMovements, x: u16) {
    for _ in 0..x {
      self.move_object(screen_data, &move_to);

      screen_data.print_screen();

      screen_data.wait_for_x_ticks(2);
    }

    screen_data.wait_for_x_ticks(BASIC_MOVEMENT_TIMER);
  }

  fn spin_cube(&mut self, screen_data: &mut ScreenData, spin_count: usize) {
    let x = 10;
    let move_x_times_horizontally = HORIZONTAL_UNIT * x;
    let move_x_times_vertically = VERTICAL_UNIT * x;

    for _ in 0..spin_count {
      self.move_x_times(
        screen_data,
        ObjectMovements::Left,
        move_x_times_horizontally,
      );

      self.move_x_times(screen_data, ObjectMovements::Up, move_x_times_vertically);

      self.move_x_times(
        screen_data,
        ObjectMovements::Right,
        move_x_times_horizontally,
      );

      self.move_x_times(screen_data, ObjectMovements::Down, move_x_times_vertically);
    }
  }

  fn user_move_cube(&mut self, screen_data: &mut ScreenData) {
    let (input_receiver, input_end_sender) = user_input::spawn_input_thread();

    loop {
      if let Ok(user_input) = input_receiver.recv() {
        match user_input.to_lowercase().trim() {
          "h" | "a" => self.move_object(screen_data, &ObjectMovements::Left),
          "l" | "d" => self.move_object(screen_data, &ObjectMovements::Right),
          "k" | "w" => self.move_object(screen_data, &ObjectMovements::Up),
          "j" | "s" => self.move_object(screen_data, &ObjectMovements::Down),
          "t" => self.print_square_data(screen_data),
          "e" => break,
          _ => continue,
        }

        screen_data.print_screen();
      }
    }

    let _ = input_end_sender.send(());
  }
}

// the y value refuses to go to 0
// can move off the screen in the positive directions
//   - causes some really weird bugs when going right
//   - crashes when going down
//
//  make this an implementation for objects
//  use self to define the assigned number
fn move_cube(
  screen_data: &mut ScreenData,
  coordinate_cube: Vec<Coordinates>,
  move_to: &ObjectMovements,
) {
  for pixel_coords in coordinate_cube {
    let swap_with = pixel_coords.move_coords(move_to);
    if let Some(swap_with) = swap_with {
      screen_data.transfer_assigned_object_in_pixel_to(&pixel_coords, &swap_with);
      let _ = screen_data.change_pixel_display_at(&swap_with, "square".to_string(), 0);
    } else {
      return;
    }
  }
}
