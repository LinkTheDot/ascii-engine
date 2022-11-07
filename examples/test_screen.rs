use ascii_engine::objects::{hollow_square::*, object_data::*};
use ascii_engine::screen::screen_data::*;
use std::io;
use std::thread;
use std::time::Duration;

pub fn main() {
  let mut screen_data = ScreenData::default();
  let mut new_square = Object::create_hollow_square(&mut screen_data, Some((100, 6)));
  let second_square = Object::create_hollow_square(&mut screen_data, Some((82, 5)));

  let mut user_input = String::new();

  new_square.place_object(&mut screen_data);
  second_square.place_object(&mut screen_data);

  println!("{GRID_SPACER}");
  println!("{}", screen_data.display());

  for error_count in (0..5).rev() {
    println!("choose a mode, 'manual' | 'spin'");
    io::stdin().read_line(&mut user_input).unwrap();

    match user_input.to_lowercase().trim() {
      "manual" => new_square.user_move_cube(&mut screen_data),
      "spin" => new_square.spin_cube(&mut screen_data, 100),
      // add a clock test that'll synchronize 2 objects
      _ => {
        println!("{GRID_SPACER}");
        println!("{}", screen_data.display());

        println!("Incorrect Input, {} attempts remaining", error_count);
      }
    }
  }

  for exit_counter in (1..6).rev() {
    println!("{GRID_SPACER}");
    println!("{GRID_SPACER}");
    println!("{GRID_SPACER}");

    println!("Now closing in {exit_counter}");

    println!("{GRID_SPACER}");

    thread::sleep(Duration::from_millis(500));
  }
}
