// use ascii_engine::general_data::coordinates::*;
use ascii_engine::general_data::user_input::spawn_input_thread;
use ascii_engine::prelude::*;
// use ascii_engine::screen::objects::Objects;
use std::sync::{Arc, Mutex};

#[allow(unused)]
use log::{debug, error, info, warn};

//skin
//xxx
//xcx
//
//hitbox
//xxx
//-x-

#[derive(Debug, Object)]
struct Square {
  object_data: Arc<Mutex<ObjectData>>,
}

impl Square {
  fn new(position: (usize, usize)) -> Self {
    let sprite = get_square_sprite();
    let hitbox = get_square_hitbox();
    let name = String::from("Square");
    let square_object_data = ObjectData::new(position, sprite, hitbox, Strata(0), name).unwrap();

    Square {
      object_data: Arc::new(Mutex::new(square_object_data)),
    }
  }

  fn create_empty_square(position: (usize, usize)) -> Self {
    let skin = Skin::new("%%%%%\n%-c-%\n%%%%%", 'c', '-', '-').unwrap();
    let sprite = Sprite::new(skin).unwrap();
    let hitbox = HitboxCreationData::new("", '-');
    let name = String::from("Square");
    let square_object_data = ObjectData::new(position, sprite, hitbox, Strata(0), name).unwrap();

    Square {
      object_data: Arc::new(Mutex::new(square_object_data)),
    }
  }
}

fn main() {
  let mut screen = ScreenData::new().unwrap();

  let mut square = Square::new((5, 5));
  let mut square2 = Square::new((10, 10));
  let mut empty_square = Square::create_empty_square((15, 5));
  debug!("empty_square_is \n{:#?}", empty_square);

  info!("{:#?}", square);

  screen.add_object(&mut square).log_if_err();
  screen.add_object(&mut square2).log_if_err();
  screen.add_object(&mut empty_square).log_if_err();

  screen.print_screen().log_if_err();

  // spin_object(&mut screen, square); // for automatic movement
  user_move(&mut screen, square); // for user movement

  warn!("Program closed.");
}

#[allow(dead_code)]
fn spin_object<O: Object>(screen: &mut ScreenData, mut object: O) {
  for _ in 0..100 {
    for _ in 0..26 {
      screen.print_screen().log_if_err();

      object.move_by((2, 0));

      screen.wait_for_x_ticks(1);
    }

    for _ in 0..13 {
      screen.print_screen().log_if_err();

      object.move_by((0, 1));

      screen.wait_for_x_ticks(2);
    }

    for _ in 0..26 {
      screen.print_screen().log_if_err();

      object.move_by((-1, 0));

      screen.wait_for_x_ticks(1);
    }

    for _ in 0..13 {
      screen.print_screen().log_if_err();

      object.move_by((0, -1));

      screen.wait_for_x_ticks(2);
    }
  }
}

#[allow(dead_code)]
// fn user_move<O: Object + std::fmt::Debug>(screen: &mut ScreenData, mut object: O) {
fn user_move(screen: &mut ScreenData, mut object: Square) {
  let (user_input, input_kill_sender) = spawn_input_thread();
  let mut previous_position = object.get_top_left_position();

  for input in user_input {
    screen.wait_for_x_ticks(1);

    info!("THE INPUT WAS {input}");
    screen
      .print_screen()
      .unwrap_or_else(|error| error!("{error:?}"));

    let move_by = match input.to_lowercase().trim() {
      "w" => {
        screen.wait_for_x_ticks(1);

        (0, -1)
      }
      "a" => (-1, 0),
      "s" => {
        screen.wait_for_x_ticks(1);

        (0, 1)
      }
      "d" => (1, 0),
      "q" => {
        input_kill_sender.send(()).unwrap();

        break;
      }
      _ => continue,
    };

    // info!("Position: ({}, {})", current_x, current_y);
    // square.move_to((current_x, current_y)).unwrap();
    let collisions = object.move_by(move_by);

    if !collisions.is_empty() {
      // info!("collision: {collisions:?}");

      object.move_by((-move_by.0, -move_by.1));
    }

    let new_position = object.get_top_left_position();

    info!("previous_position: {previous_position}, new_position: {new_position}",);

    previous_position = new_position;

    screen
      .print_screen()
      .unwrap_or_else(|error| error!("{error:?}"));

    // info!("current_object_data: \n{:#?}", object);
    screen.wait_for_x_ticks(1);
  }
}

fn get_square_sprite() -> Sprite {
  let skin = Skin::new("xxxxx\nx-c-x\nxxxxx", 'c', '-', '-').unwrap();
  // "--x--\n-xxx-\nxxxxx\nx-c-x"

  Sprite::new(skin).unwrap()
}

fn get_square_hitbox() -> HitboxCreationData {
  HitboxCreationData::new("xxxxx\nxxcxx\nxxxxx", 'c')
}

trait ResultTraits {
  /// Logs the result if it's an error.
  /// The message will be under 'Error' when logged.
  fn log_if_err(&self);
}

impl<T, E> ResultTraits for Result<T, E>
where
  E: std::fmt::Debug,
{
  fn log_if_err(&self) {
    if let Err(error) = self {
      error!("{error:?}")
    }
  }
}
