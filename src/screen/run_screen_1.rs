use crate::objects::{hollow_square::*, object_data::*};
use crate::screen::screen_data::*;
use std::error::Error;
use std::io;

pub fn run_screen(mut screen_data: ScreenData) -> Result<(), Box<dyn Error>> {
  // possibly just use a channel to store all the updates
  // that would happen in a given 'pass' and go through them all
  let new_square = Object::create_hollow_square(Some((30, 15)));

  new_square.place_object(&mut screen_data);

  println!("{}", screen_data.display());

  Ok(())
}

pub fn run_test_screen(mut screen_data: ScreenData) -> Result<(), Box<dyn Error>> {
  let mut new_square = Object::create_hollow_square(Some((100, 5)));
  let mut user_input = String::new();

  new_square.place_object(&mut screen_data);

  println!("{GRID_SPACER}");
  println!("{}", screen_data.display());

  println!("choose a mode, 'manual' | 'spin'");
  io::stdin().read_line(&mut user_input).unwrap();

  match user_input.to_lowercase().trim() {
    "manual" => new_square.user_move_cube(&mut screen_data),
    "spin" => new_square.spin_cube(&mut screen_data, 100),
    _ => (),
  }

  Ok(())
}
