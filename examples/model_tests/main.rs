use crate::screen_config::*;
use ascii_engine::general_data::coordinates::*;
use ascii_engine::general_data::user_input::spawn_input_thread;
use ascii_engine::prelude::*;
use guard::guard;
use std::collections::VecDeque;
use std::path::Path;

#[allow(unused)]
use log::{debug, error, info, warn};

mod screen_config;

#[derive(Debug, DisplayModel)]
pub struct Square {
  model_data: ModelData,
}

#[derive(Debug, DisplayModel)]
pub struct Wall {
  model_data: ModelData,
}

#[derive(Debug, DisplayModel)]
pub struct TeleportPad {
  model_data: ModelData,
  connected_pad_hash: u64,
}

impl Square {
  fn from_file(path: &Path, world_position: (usize, usize)) -> Self {
    Self {
      model_data: ModelData::from_file(path, world_position).unwrap(),
    }
  }

  fn check_collisions(
    mut initial_square: ModelData,
    mut collision_list: VecDeque<ModelData>,
    move_by: (isize, isize),
    screen_config: &ScreenConfig,
  ) {
    while !collision_list.is_empty() {
      guard!( let Some(mut collided_model) = collision_list.pop_back() else { return; });

      let model_name = collided_model.get_name().to_lowercase();

      match model_name.trim() {
        "square" => {
          let new_collisions = collided_model.move_by(move_by);
          let new_collisions = VecDeque::from(new_collisions);

          let collisions = (move_by, new_collisions);

          Self::check_collisions(collided_model, collisions.1, collisions.0, screen_config);
        }

        "wall" => {
          let move_back = (-move_by.0, -move_by.1);

          let collisions = initial_square.move_by(move_back);

          let collisions = (move_back, VecDeque::from(collisions));

          Self::check_collisions(
            initial_square.clone(),
            collisions.1,
            collisions.0,
            screen_config,
          );
        }

        "teleport pad" => {
          let teleport_pad_position = collided_model.get_model_position();
          let initial_square_position = initial_square.get_model_position();

          if teleport_pad_position == initial_square_position {
            debug!("{teleport_pad_position} == {initial_square_position}");
            let touched_teleport_pad = screen_config
              .get_teleport_pad(&collided_model.get_unique_hash())
              .unwrap();
            let connected_teleport_pad = screen_config
              .get_teleport_pad(&touched_teleport_pad.get_connected_pad_hash())
              .unwrap();

            let connected_pad_position = connected_teleport_pad.get_position();
            let connected_pad_position =
              connected_pad_position.index_to_coordinates((CONFIG.grid_width + 1) as usize);

            let collision_list = initial_square.move_to(connected_pad_position);

            // move back if something else was on the other pad
            if collision_list.len() > 1 {
              let previous_square_position =
                initial_square_position.index_to_coordinates((CONFIG.grid_width + 1) as usize);

              initial_square.move_to(previous_square_position);
            }
          }
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
}

impl TeleportPad {
  fn new(
    pad_1_world_position: (usize, usize),
    pad_2_world_position: (usize, usize),
  ) -> (Self, Self) {
    let model_path = Path::new("examples/models/teleport_pad.model");
    let pad_1_model_data = ModelData::from_file(model_path, pad_1_world_position).unwrap();
    let pad_2_model_data = ModelData::from_file(model_path, pad_2_world_position).unwrap();

    let pad_1_hash = pad_1_model_data.get_unique_hash();
    let pad_2_hash = pad_2_model_data.get_unique_hash();

    let teleport_pad_1 = Self {
      model_data: pad_1_model_data,
      connected_pad_hash: pad_2_hash,
    };
    let teleport_pad_2 = Self {
      model_data: pad_2_model_data,
      connected_pad_hash: pad_1_hash,
    };

    (teleport_pad_1, teleport_pad_2)
  }

  fn is_connected_to(&self, other: &Self) -> bool {
    let other_hash = other.get_unique_hash();

    self.connected_pad_hash == other_hash
  }

  fn get_connected_pad_hash(&self) -> u64 {
    self.connected_pad_hash
  }
}

fn main() {
  let mut screen = ScreenData::new();
  screen.start_printer().unwrap();
  let mut screen_config = ScreenConfig::new(screen);

  let player_square_hash = add_squares(&mut screen_config);
  add_walls(&mut screen_config);
  add_teleport_pads(&mut screen_config);

  screen_config.screen.print_screen().log_if_err();

  user_move(&mut screen_config, player_square_hash);

  warn!("Program closed.");
}

/// Returns the hash for the player's square.
fn add_squares(screen_config: &mut ScreenConfig) -> u64 {
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

  info!("{:#?}", square_list[0]);

  let mut square_hash_list: Vec<u64> = square_list
    .into_iter()
    .map(|square| screen_config.add_square(square).unwrap())
    .collect();

  square_hash_list.remove(0)
}

fn add_walls(screen_config: &mut ScreenConfig) {
  let wall_path = Path::new("examples/models/wall.model");

  // left side
  let wall1 = Wall::from_file(wall_path, (30, 15));
  let wall2 = Wall::from_file(wall_path, (40, 15));

  // right side
  let wall3 = Wall::from_file(wall_path, (80, 15));
  let wall4 = Wall::from_file(wall_path, (90, 15));

  screen_config.add_wall(wall1).log_if_err();
  screen_config.add_wall(wall2).log_if_err();
  screen_config.add_wall(wall3).log_if_err();
  screen_config.add_wall(wall4).log_if_err();
}

fn add_teleport_pads(screen_config: &mut ScreenConfig) {
  let (pad_1, pad_2) = TeleportPad::new((35, 15), (85, 15));

  screen_config.add_teleport_pads(pad_1, pad_2).unwrap();
}

fn user_move(screen_config: &mut ScreenConfig, player_hash: u64) {
  let (user_input, input_kill_sender) = spawn_input_thread(&screen_config.screen).unwrap();
  let mut previous_frame_index = screen_config
    .get_square(&player_hash)
    .unwrap()
    .get_top_left_position();
  let mut player_model_data = screen_config
    .get_square(&player_hash)
    .unwrap()
    .get_model_data();

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

    let player_collisions = player_model_data.move_by(move_by);
    let player_collisions = VecDeque::from(player_collisions);

    Square::check_collisions(
      player_model_data.clone(),
      player_collisions,
      move_by,
      screen_config,
    );

    let new_frame_index = player_model_data.top_left();

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
