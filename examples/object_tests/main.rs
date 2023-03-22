// use ascii_engine::general_data::coordinates::*;
use ascii_engine::general_data::user_input::spawn_input_thread;
use ascii_engine::prelude::*;
// use ascii_engine::screen::objects::Objects;
use crate::screen_config::*;
use std::sync::{Arc, Mutex, RwLock, RwLockWriteGuard};

#[allow(unused)]
use log::{debug, error, info, warn};

mod screen_config;

#[derive(Debug, Object)]
pub struct Square {
  object_data: Arc<Mutex<ObjectData>>,
}

#[derive(Debug, Object)]
pub struct Wall {
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

  fn wrap_self(self) -> Arc<RwLock<Self>> {
    Arc::new(RwLock::new(self))
  }

  fn create_empty_square(position: (usize, usize)) -> Self {
    let skin = Skin::new("%%%%%\n%-c-%\n%%%%%", 'c', '-', '-').unwrap();
    let sprite = Sprite::new(skin).unwrap();
    let hitbox = HitboxCreationData::new("", '-');
    let name = String::from("Square");
    let square_object_data = ObjectData::new(position, sprite, hitbox, Strata(5), name).unwrap();

    Square {
      object_data: Arc::new(Mutex::new(square_object_data)),
    }
  }

  fn pushed_square(
    screen_config: &mut ScreenConfig,
    square: Arc<Mutex<ObjectData>>,
    move_by: (isize, isize),
  ) {
    let pushed_object_guard = square.lock().unwrap();
    let pushed_object_hash = *pushed_object_guard.get_unique_hash();
    drop(pushed_object_guard);

    let pushed_object = screen_config.get_square(&pushed_object_hash);

    let mut pushed_object_guard = pushed_object.write().unwrap();
    let collisions = pushed_object_guard.move_by(move_by);
    drop(pushed_object_guard);

    Square::check_collisions(collisions, screen_config, move_by, &pushed_object);
  }

  fn check_collisions(
    collision_list: Vec<Arc<Mutex<ObjectData>>>,
    screen_config: &mut ScreenConfig,
    move_by: (isize, isize),
    square: &Arc<RwLock<Self>>,
  ) {
    for collision in collision_list {
      let collision_guard = collision.lock().unwrap();
      let object_name = collision_guard.get_name().to_lowercase();
      drop(collision_guard);

      match object_name.trim() {
        "square" => Square::pushed_square(screen_config, collision.clone(), move_by),
        "wall" => {
          let move_back = (-move_by.0, -move_by.1);

          let mut square_guard = square.write().unwrap();
          let new_collisions = square_guard.move_by(move_back);
          drop(square_guard);

          Square::check_collisions(new_collisions, screen_config, move_back, square);
        }
        _ => continue,
      }
    }
  }
}

impl Wall {
  fn new(position: (usize, usize)) -> Self {
    let wall_string =
      "|||||\n|||||\n|||||\n|||||\n|||||\n||c||\n|||||\n|||||\n|||||\n|||||\n|||||".to_string();
    let skin = Skin::new(&wall_string, 'c', '|', '-').unwrap();
    let sprite = Sprite::new(skin).unwrap();
    let name = String::from("Wall");
    let hitbox_data = HitboxCreationData::new(&wall_string, 'c');

    let object_data = ObjectData::new(position, sprite, hitbox_data, Strata(100), name).unwrap();
    Self {
      object_data: Arc::new(Mutex::new(object_data)),
    }
  }

  pub fn wrap_self(self) -> Arc<RwLock<Self>> {
    Arc::new(RwLock::new(self))
  }
}

fn main() {
  let screen = ScreenData::new().unwrap();

  let square_position_list = vec![(5, 5), (10, 10), (20, 20), (15, 5)];
  let square_list: Vec<Square> = square_position_list
    .into_iter()
    .enumerate()
    .map(|(count, position)| {
      if (count + 1) % 4 == 0 {
        Square::create_empty_square(position)
      } else {
        Square::new(position)
      }
    })
    .collect();

  let wall = Wall::new((30, 15));

  info!("{:#?}", square_list[0]);

  let mut screen_config = ScreenConfig::new(screen);

  screen_config.add_wall(wall).log_if_err();
  let mut square_list: Vec<Arc<RwLock<Square>>> = square_list
    .into_iter()
    .map(|square| screen_config.add_square(square).unwrap())
    .collect();

  screen_config.screen.print_screen().log_if_err();

  // spin_object(&mut screen, square); // for automatic movement
  user_move(&mut screen_config, square_list.remove(0)); // for user movement

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
fn user_move(screen_config: &mut ScreenConfig, square: Arc<RwLock<Square>>) {
  let (user_input, input_kill_sender) = spawn_input_thread();
  let square_guard = square.read().unwrap();
  let mut previous_position = square_guard.get_top_left_position();
  drop(square_guard);

  for input in user_input {
    screen_config.screen.wait_for_x_ticks(1);

    info!("THE INPUT WAS {input}");
    screen_config
      .screen
      .print_screen()
      .unwrap_or_else(|error| error!("{error:?}"));

    let move_by = match input.to_lowercase().trim() {
      "w" => {
        screen_config.screen.wait_for_x_ticks(1);

        (0, -1)
      }
      "a" => (-1, 0),
      "s" => {
        screen_config.screen.wait_for_x_ticks(1);

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
    let mut square_guard = square.write().unwrap();
    let collisions = square_guard.move_by(move_by);
    drop(square_guard);

    Square::check_collisions(collisions, screen_config, move_by, &square);

    let square_guard = square.read().unwrap();
    let new_position = square_guard.get_top_left_position();
    drop(square_guard);

    info!("previous_position: {previous_position}, new_position: {new_position}",);

    previous_position = new_position;

    screen_config
      .screen
      .print_screen()
      .unwrap_or_else(|error| error!("{error:?}"));

    // info!("current_object_data: \n{:#?}", object);
    screen_config.screen.wait_for_x_ticks(1);
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
