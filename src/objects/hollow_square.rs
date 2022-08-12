use crate::general_data::coordinates::*;
use crate::objects::object_data::*;
use crate::screen::screen_data::*;
use std::io;

pub const SQUARE_SHAPE: &str = // /
  "vvxX  Xxcc
xxX    Xxx
xX      Xx
xxX    Xxx
CCxX  XxVV";

pub trait HollowSquare {
  fn create_hollow_square(position: Option<Coordinates>) -> Object;

  fn move_object(&mut self, screen_data: &mut ScreenData, move_to: ObjectMovements);

  fn spin_cube(&mut self, screen_data: &mut ScreenData, spin_count: usize);
  fn user_move_cube(&mut self, screen_data: &mut ScreenData);
}

impl HollowSquare for Object {
  fn create_hollow_square(position: Option<Coordinates>) -> Object {
    Object::create("square", SQUARE_SHAPE, position)
  }

  fn move_object(&mut self, screen_data: &mut ScreenData, move_to: ObjectMovements) {
    let new_coords = match self.position.move_coords(&move_to) {
      Some(coords) => coords,
      None => return,
    };

    if new_coords
      .get_object_bounds(&move_to, self.width, self.height)
      .is_none()
    {
      return;
    }

    let bottom_right_of_square = self.get_bottom_right_of_object();
    let mut coordinate_cube = self
      .position
      .get_coordinates_in_between(&bottom_right_of_square);

    self.position = new_coords;

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

  fn spin_cube(&mut self, screen_data: &mut ScreenData, spin_count: usize) {
    let ticks_between_moves = 6;

    for _ in 0..spin_count {
      self.move_object(screen_data, ObjectMovements::Left);
      self.move_object(screen_data, ObjectMovements::Left);
      screen_data
        .screen_clock
        .wait_for_x_ticks(ticks_between_moves);

      println!("{GRID_SPACER}");
      println!("{}", screen_data.display());

      self.move_object(screen_data, ObjectMovements::Up);
      screen_data
        .screen_clock
        .wait_for_x_ticks(ticks_between_moves);

      println!("{GRID_SPACER}");
      println!("{}", screen_data.display());

      self.move_object(screen_data, ObjectMovements::Right);
      self.move_object(screen_data, ObjectMovements::Right);
      screen_data
        .screen_clock
        .wait_for_x_ticks(ticks_between_moves);

      println!("{GRID_SPACER}");
      println!("{}", screen_data.display());

      self.move_object(screen_data, ObjectMovements::Down);
      screen_data
        .screen_clock
        .wait_for_x_ticks(ticks_between_moves);

      println!("{GRID_SPACER}");
      println!("{}", screen_data.display());
    }
  }

  fn user_move_cube(&mut self, screen_data: &mut ScreenData) {
    loop {
      let mut user_input = String::new();

      io::stdin().read_line(&mut user_input).unwrap();

      match user_input.to_lowercase().trim() {
        "left" | "l" => self.move_object(screen_data, ObjectMovements::Left),
        "right" | "r" => self.move_object(screen_data, ObjectMovements::Right),
        "up" | "u" => self.move_object(screen_data, ObjectMovements::Up),
        "down" | "d" => self.move_object(screen_data, ObjectMovements::Down),
        "exit" | "e" => break,
        _ => continue,
      }

      println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
      println!("{}", screen_data.display());
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
    //
    // crashes whenever any coordinate is 0 and of course when going
    // out of bounds period
    if let Some(swap_with) = swap_with {
      screen_data.transfer_latest_object_in_pixel_to(&pixel_coords, &swap_with);
      screen_data.change_pixel_display_at(&swap_with, "square".to_string());
    } else {
      return;
    }
  }
}
