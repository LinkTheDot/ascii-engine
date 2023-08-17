use crate::collision_data::*;
use crate::screen_config::*;
use ascii_engine::general_data::user_input::spawn_input_thread;
use engine_math::coordinates::*;
// use ascii_engine::models::animation::*;
use ascii_engine::prelude::*;
use log::{error, info, warn};
use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, Mutex};

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
    relative_movement: (isize, isize),
    screen_config: &ScreenConfig,
  ) -> CollisionChain {
    let mut collision_chain = CollisionChain::new();

    let square_movement = MovementType::RelativeMovement(relative_movement);
    let initial_square_action = CollisionAction::new(initial_square.clone(), square_movement);
    collision_chain.add_action(initial_square_action);
    while !collision_list.is_empty() {
      let Some(collided_model) = collision_list.pop_back() else { break; };

      let model_name = collided_model.get_name().to_lowercase();

      match model_name.trim() {
        "square" => {
          let new_collisions =
            VecDeque::from(collided_model.relative_movement_collision_check(relative_movement));

          let added_link = Square::collision_checks(
            collided_model,
            new_collisions,
            relative_movement,
            screen_config,
          );

          collision_chain.append(added_link);
        }

        "wall" => collision_chain.cancel_action_chain(),

        "teleport pad" => {
          let teleport_pad_position = collided_model.get_model_position();

          let relative_distance_to_moved_position =
            relative_movement.0 + (relative_movement.1 * (CONFIG.grid_width as isize + 1));
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
              initial_square.absolute_movement_collision_check(connected_pad_position);

            if potential_collisions.len() == 1 {
              let new_movement = MovementType::AbsoluteMovement(connected_pad_position);

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

#[tokio::main]
async fn main() {
  let mut screen_config = ScreenConfig::new(ScreenData::new());

  spawn_printing_task(screen_config.screen.clone());

  let player_square_hash = add_squares(&mut screen_config).await;
  add_walls(&mut screen_config);
  add_teleport_pads(&mut screen_config);

  user_move(&mut screen_config, player_square_hash).await;

  warn!("Program closed.");
  std::process::exit(0);
}

fn spawn_printing_task(screen: Arc<Mutex<ScreenData>>) {
  let _printing_handle = tokio::task::spawn(async move {
    loop {
      std::thread::sleep(std::time::Duration::from_millis(12));

      screen.lock().unwrap().print_screen().log_if_err();
    }
  });
}

/// Gives a list of coordinates to place 50 squares for testing placing a ton of models.
fn get_50_square_coordinates() -> Vec<(usize, usize)> {
  let initial_square = (95, 1);

  (0..50)
    .map(|iteration| {
      let x = ((iteration as f32 / 5.0).floor() * 5.0) as usize + initial_square.0;
      let y = ((iteration % 5) * 3) + initial_square.1;

      (x, y)
    })
    .collect()
}

/// Places 4 squares near each other and returns the hash to one of them.
async fn add_squares(screen_config: &mut ScreenConfig) -> u64 {
  let mut square_world_position_list = vec![(20, 10), (25, 10), (20, 20), (15, 5)];
  square_world_position_list.append(&mut get_50_square_coordinates());

  // let square_list: Vec<Square> = square_world_position_list
  //   .into_iter()
  //   .enumerate()
  //   .map(|(iteration, world_position)| {
  //     let square_path = if (iteration + 1) % 4 == 0 {
  //       Path::new("examples/models/air_square.model")
  //     } else {
  //       Path::new("examples/models/square.model")
  //     };
  //
  //     Square::from_file(square_path, world_position)
  //   })
  //   .collect();
  //
  // let mut square_hash_list: Vec<u64> = square_list
  //   .into_iter()
  //   .flat_map(|square| screen_config.add_square(square))
  //   .collect();
  //
  // square_hash_list.remove(0)

  square_world_position_list
    .into_iter()
    .enumerate()
    .map(|(iteration, world_position)| {
      let square_path = if (iteration + 1) % 4 == 0 {
        Path::new("examples/models/air_square.model")
      } else {
        Path::new("examples/models/square.model")
      };

      screen_config
        .add_square(Square::from_file(square_path, world_position))
        .unwrap()
    })
    // .flat_map(|square| screen_config.add_square(square))
    .next()
    .unwrap()
}

fn add_walls(screen_config: &mut ScreenConfig) {
  let wall_path = Path::new("examples/models/wall.model");

  vec![
    // Left walls.
    (30, 15),
    (40, 15),
    // Right walls.
    (80, 15),
    (90, 15),
  ]
  .into_iter()
  .map(|world_position| Wall::from_file(wall_path, world_position))
  .for_each(|wall| screen_config.add_wall(wall).log_if_err());
}

fn add_teleport_pads(screen_config: &mut ScreenConfig) {
  let (pad_1, pad_2) = TeleportPad::new((35, 15), (85, 15));

  screen_config.add_teleport_pads(pad_1, pad_2).unwrap();
}

async fn user_move(screen_config: &mut ScreenConfig, player_hash: u64) {
  let (user_input, input_kill_sender) = spawn_input_thread().unwrap();
  let player_model_data = screen_config
    .get_square(&player_hash)
    .unwrap()
    .get_model_data();

  let event_sync = screen_config.screen.lock().unwrap().get_event_sync();

  for input in user_input {
    event_sync.wait_for_tick();

    info!("THE INPUT WAS {input}");

    let relative_movement = match input.to_lowercase().trim() {
      "w" => (0, -1),
      "a" => (-1, 0),
      "s" => (0, 1),
      "d" => (1, 0),
      "q" => {
        input_kill_sender.send(()).unwrap();

        break;
      }
      _ => continue,
    };

    // relative_movement/absolute_movement
    let possible_collisions =
      VecDeque::from(player_model_data.relative_movement_collision_check(relative_movement));

    Square::collision_checks(
      player_model_data.clone(),
      possible_collisions,
      relative_movement,
      screen_config,
    )
    .run_action_list();
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
