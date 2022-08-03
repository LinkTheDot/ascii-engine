use crate::objects::{hollow_square::*, object_data::*};
use crate::screen::screen_data::*;
use std::error::Error;

#[allow(unused_mut)]
pub fn run_screen(mut screen_data: ScreenData) -> Result<(), Box<dyn Error>> {
  // possibly just use a channel to store all the updates
  // that would happen in a given 'pass' and go through them all
  let new_square = Object::create_hollow_square(Some((30, 15)));

  new_square.place_object(&mut screen_data);

  println!("{}", screen_data.display());

  Ok(())
}
