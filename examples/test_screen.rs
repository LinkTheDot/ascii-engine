use ascii_engine::objects::{hollow_square::*, object_data::*};
use ascii_engine::screen::screen_data::*;
use std::io;
use std::thread;
use std::time::Duration;

pub fn main() {
  let mut screen_data = ScreenData::default();
  let main_square = Object::create_hollow_square(&mut screen_data, Some((100, 6)));
  let secondary_square = Object::create_hollow_square(&mut screen_data, Some((82, 5)));

  main_square.place_object(&mut screen_data);
  secondary_square.place_object(&mut screen_data);

  screen_data.print_screen();

  parse_user_input(screen_data, main_square);
}

fn parse_user_input(mut screen_data: ScreenData, mut main_square: Object) {
  for error_count in (0..5).rev() {
    let mut user_input = String::new();

    screen_data.print_text("choose a mode, 'manual' | 'spin'");
    io::stdin().read_line(&mut user_input).unwrap();

    match user_input.to_lowercase().trim() {
      "manual" => main_square.user_move_cube(&mut screen_data),
      "spin" => main_square.spin_cube(&mut screen_data, 100),
      "e" | "exit" => break,
      // add a clock test that'll synchronize 2 objects
      _ => {
        screen_data.print_screen();

        screen_data.print_text(format!(
          "\nIncorrect Input, {} attempts remaining",
          error_count
        ));
      }
    }
  }

  exit_countdown(&mut screen_data);
}

fn exit_countdown(screen: &mut ScreenData) {
  for exit_counter in (1..6).rev() {
    screen.clear_screen().unwrap();

    println!("\nNow closing in {exit_counter}");

    thread::sleep(Duration::from_millis(500));
  }
}
