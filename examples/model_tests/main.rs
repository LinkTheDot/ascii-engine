use crate::screen_config::*;
use ascii_engine::general_data::user_input::spawn_input_thread;
use ascii_engine::prelude::*;
use guard::guard;
use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, RwLock};

#[allow(unused)]
use log::{debug, error, info, warn};

mod screen_config;

#[derive(Debug, DisplayModel)]
// #[derive(Debug)]
pub struct Square {
  model_data: ModelData,
}

#[derive(Debug, DisplayModel)]
pub struct Wall {
  model_data: ModelData,
}

impl Square {
  fn from_file(path: &Path, world_position: (usize, usize)) -> Self {
    Self {
      model_data: ModelData::from_file(path, world_position).unwrap(),
    }
  }

  fn wrap_self(self) -> Arc<RwLock<Self>> {
    Arc::new(RwLock::new(self))
  }

  fn check_collisions(
    initial_square: &Arc<RwLock<Self>>,
    mut collision_list: VecDeque<ModelData>,
    move_by: (isize, isize),
    screen_config: &mut ScreenConfig,
  ) {
    while !collision_list.is_empty() {
      guard!( let Some(collided_model) = collision_list.pop_back() else { return; });

      let model_name = collided_model.get_name().to_lowercase();

      match model_name.trim() {
        "square" => {
          let pushed_model_hash = collided_model.get_unique_hash();
          let pushed_model = screen_config.get_square(&pushed_model_hash);

          let mut pushed_model_guard = pushed_model.write().unwrap();
          let new_model_collisions = VecDeque::from(pushed_model_guard.move_by(move_by));
          drop(pushed_model_guard);

          let collisions = (move_by, new_model_collisions);

          Self::check_collisions(&pushed_model, collisions.1, collisions.0, screen_config);
        }

        "wall" => {
          let move_back = (-move_by.0, -move_by.1);

          let mut square_guard = initial_square.write().unwrap();
          let collisions = square_guard.move_by(move_back);
          drop(square_guard);

          let collisions = (move_back, VecDeque::from(collisions));

          Self::check_collisions(initial_square, collisions.1, collisions.0, screen_config);
        }

        _ => (),
      }
    }
  }
}

impl Wall {
  fn from_file(path: &Path, world_position: (usize, usize)) -> Self {
    Self {
      model_data: ModelData::from_file(path, world_position).unwrap(),
    }
  }

  pub fn wrap_self(self) -> Arc<RwLock<Self>> {
    Arc::new(RwLock::new(self))
  }
}

fn main() {
  let mut screen = ScreenData::new().unwrap();
  screen.start_printer().unwrap();

  let square_world_position_list = vec![(20, 10), (25, 10), (20, 20), (15, 5)];
  let square_list: Vec<Square> = square_world_position_list
    .into_iter()
    .enumerate()
    .map(|(count, world_position)| {
      let square_path = if (count + 1) % 4 == 0 {
        Path::new("examples/models/air_square.model")
      } else {
        Path::new("examples/models/square.model")
      };

      Square::from_file(square_path, world_position)
    })
    .collect();

  let wall_path = Path::new("examples/models/wall.model");
  let wall = Wall::from_file(wall_path, (30, 15));

  info!("{:#?}", square_list[0]);

  let mut screen_config = ScreenConfig::new(screen);
  screen_config.add_wall(wall).log_if_err();
  let mut square_list: Vec<Arc<RwLock<Square>>> = square_list
    .into_iter()
    .map(|square| screen_config.add_square(square).unwrap())
    .collect();

  screen_config.screen.print_screen().log_if_err();

  user_move(&mut screen_config, square_list.remove(0)); // for user movement

  warn!("Program closed.");
}

fn user_move(screen_config: &mut ScreenConfig, square: Arc<RwLock<Square>>) {
  let (user_input, input_kill_sender) = spawn_input_thread(&screen_config.screen).unwrap();
  let square_guard = square.read().unwrap();
  let mut previous_frame_index = square_guard.get_top_left_position();
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

    let mut square_guard = square.write().unwrap();
    let collisions = VecDeque::from(square_guard.move_by(move_by));
    drop(square_guard);

    Square::check_collisions(&square, collisions, move_by, screen_config);

    let square_guard = square.read().unwrap();
    let new_frame_index = square_guard.get_top_left_position();
    drop(square_guard);

    info!(
      "previous_position: {}, new_position: {}",
      previous_frame_index, new_frame_index
    );

    previous_frame_index = new_frame_index;

    screen_config
      .screen
      .print_screen()
      .unwrap_or_else(|error| error!("{error:?}"));

    screen_config.screen.wait_for_x_ticks(1);
  }
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
