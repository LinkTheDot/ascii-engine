use crate::collision_data::*;
use crate::screen_config::*;
use ascii_engine::general_data::coordinates::*;
use ascii_engine::general_data::user_input::spawn_input_thread;
use ascii_engine::prelude::*;
use guard::guard;
use std::collections::VecDeque;
use std::path::Path;

#[allow(unused)]
use log::{debug, error, info, warn};

mod collision_data;
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

#[derive(Debug)]
pub struct ConnectedTeleportPads<'a> {
  pub teleport_pad_1: &'a TeleportPad,
  pub teleport_pad_2: &'a TeleportPad,
}

impl Square {
  fn from_file(path: &Path, world_position: (usize, usize)) -> Self {
    Self {
      model_data: ModelData::from_file(path, world_position).unwrap(),
    }
  }

  fn collision_checks(
    initial_square: ModelData,
    mut collision_list: VecDeque<ModelData>,
    move_by: (isize, isize),
    screen_config: &ScreenConfig,
  ) -> CollisionChain {
    let mut collision_chain = CollisionChain::new();

    let square_movement = MovementType::MoveBy(move_by);
    let initial_square_action = CollisionAction::new(initial_square.clone(), square_movement);
    collision_chain.add_action(initial_square_action);

    while !collision_list.is_empty() {
      guard!( let Some(mut collided_model) = collision_list.pop_back() else { break; });

      let model_name = collided_model.get_name().to_lowercase();

      match model_name.trim() {
        "square" => {
          let new_collisions = VecDeque::from(collided_model.move_by_collision_check(move_by));

          let added_link =
            Square::collision_checks(collided_model, new_collisions, move_by, screen_config);

          collision_chain.append(added_link);
        }

        "wall" => collision_chain.cancel_action_chain(),

        "teleport pad" => {
          let teleport_pad_position = collided_model.get_model_position();

          let relative_distance_to_moved_position =
            move_by.0 + (move_by.1 * (CONFIG.grid_width as isize + 1));
          let moved_square_position = (initial_square.get_model_position() as isize
            + relative_distance_to_moved_position) as usize;

          if moved_square_position == teleport_pad_position {
            let connected_teleport_pads = screen_config
              .get_connected_teleport_pads(&collided_model.get_unique_hash())
              .unwrap();

            let connected_pad_position = connected_teleport_pads
              .teleport_pad_2
              .get_position()
              .index_to_coordinates((CONFIG.grid_width + 1) as usize);

            let potential_collisions =
              initial_square.move_to_collision_check(connected_pad_position);

            if potential_collisions.len() == 1 {
              let new_movement = MovementType::MoveTo(connected_pad_position);

              let initial_square_hash = initial_square.get_unique_hash();
              collision_chain.change_movement_of(&initial_square_hash, new_movement);
            }
          }
        }
        _ => (),
      }
    }

    collision_chain
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

  debug!("Player square hash is: {}", square_hash_list[0]);
  debug!("Moved square hash is {}", square_hash_list[1]);

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
  let player_model_data = screen_config
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

    let possible_collisions = VecDeque::from(player_model_data.move_by_collision_check(move_by));

    Square::collision_checks(
      player_model_data.clone(),
      possible_collisions,
      move_by,
      screen_config,
    )
    .run_link();

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
