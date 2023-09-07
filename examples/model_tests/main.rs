use ascii_engine::prelude::*;
use std::path::Path;

mod collision_data;
mod screen_config;

#[derive(DisplayModel)]
struct Player {
  #[allow(unused)]
  model_hash: u64,
}

impl Player {
  fn new() -> (Self, ModelData) {
    let path = Path::new("examples/models/square.model");
    let model = ModelData::from_file(path, (10, 10)).unwrap();
    let model_hash = model.get_hash();

    (Self { model_hash }, model)
  }
}

fn main() {
  let mut screen_data = ScreenData::new();
  let (player, player_model) = Player::new();

  screen_data.add_model(player_model).unwrap();

  let (_printing_thread_handle, printing_thread_kill_sender) =
    screen_data.spawn_printing_thread(60, None);

  let mut model_manager = screen_data.get_model_manager();

  loop {
    let input = get_user_input();
    let movement: (isize, isize) = match input.trim() {
      "w" => (0, -1),
      "s" => (0, 1),
      "a" => (-1, 0),
      "d" => (1, 0),
      "q" => break,
      _ => continue,
    };

    let movement = ModelMovement::Relative(movement);

    // Don't care about the collisions for now.
    let _ = model_manager
      .move_model(player.model_hash, movement)
      .log_if_err();
  }

  let _ = printing_thread_kill_sender.send(());
}

trait ResultTraits<T> {
  /// Logs the result if it's an error.
  /// The message will be under 'Error' when logged.
  fn log_if_err(self) -> Option<T>;
}

impl<T, E> ResultTraits<T> for Result<T, E>
where
  E: std::fmt::Debug,
{
  fn log_if_err(self) -> Option<T> {
    if let Err(error) = self {
      log::error!("{error:?}");

      return None;
    }

    self.ok()
  }
}
