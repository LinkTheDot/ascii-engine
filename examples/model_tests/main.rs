use ascii_engine::prelude::*;
use std::path::{Path, PathBuf};

mod collision_data;
mod screen_config;

#[derive(DisplayModel)]
struct Player {}

impl Player {
  fn get_model_data() -> ModelData {
    ModelData::from_file(Path::new("examples/models/square.model"), (10, 10)).unwrap()
  }

  fn get_animation_data() -> ModelAnimationData {
    let animation_name = "test".to_string();
    let animation_frames = vec![
      Sprite::new("xxxxx\nxxaxx\nxxxxx", 'a', 'x', '-').unwrap(),
      Sprite::new("/xxxx\nxxaxx\nxxxxx", 'a', 'x', '-').unwrap(),
      Sprite::new("x/xxx\n/xaxx\nxxxxx", 'a', 'x', '-').unwrap(),
      Sprite::new("xx/xx\nx/axx\n/xxxx", 'a', 'x', '-').unwrap(),
      Sprite::new("xxx/x\nxxaxx\nx/xxx", 'a', '/', '-').unwrap(),
      Sprite::new("xxxx/\nxxa/x\nxx/xx", 'a', 'x', '-').unwrap(),
      Sprite::new("xxxxx\nxxax/\nxxx/x", 'a', 'x', '-').unwrap(),
      Sprite::new("xxxxx\nxxaxx\nxxxx/", 'a', 'x', '-').unwrap(),
    ]
    .into_iter()
    .map(|s| AnimationFrame::new(s, 2))
    .collect();

    let animation = AnimationFrames::new(animation_frames, AnimationLoopCount::Forever, None);
    ModelAnimationData::new(vec![(animation_name, animation)])
  }
}

fn main() {
  // // create_large_test_world();
  // let path = PathBuf::from("examples/worlds/test_world.wrld");
  // // let path = PathBuf::from("examples/worlds/large_test_world.wrld");
  // let stored_world = StoredWorld::load(&path).unwrap();
  // let screen_data = ScreenData::from_world(stored_world);
  //
  // let stored_world = StoredWorld::load(path).unwrap();
  // log::info!("{:#?}", stored_world);
  //
  // let (_printing_thread_handle, printing_thread_kill_sender) =
  //   screen_data.spawn_printing_thread(60, None);
  //
  // let mut model_manager = screen_data.get_model_manager();
  //
  // loop {
  //   let input = get_user_input();
  //   let movement: (isize, isize) = match input.trim() {
  //     "w" => (0, -1),
  //     "s" => (0, 1),
  //     "a" => (-1, 0),
  //     "d" => (1, 0),
  //     "q" => break,
  //     "z" => {
  //       model_manager
  //         .queue_model_animation(&Player::HASH, "test")
  //         .log_if_err();
  //       continue;
  //     }
  //     "x" => {
  //       model_manager
  //         .remove_current_model_animation(&Player::HASH)
  //         .log_if_err();
  //       continue;
  //     }
  //     _ => continue,
  //   };
  //
  //   let movement = ModelMovement::Relative(movement);
  //
  //   // Don't care about the collisions for now.
  //   let _ = model_manager
  //     .move_model(&Player::HASH, movement)
  //     .log_if_err();
  // }
  //
  // let _ = printing_thread_kill_sender.send(());
}

#[allow(dead_code)]
fn create_large_test_world() {
  let model_count = 100;
  let mut model_list: Vec<ModelData> = Vec::with_capacity(model_count);
  let animation_data = Player::get_animation_data();

  let mut player_model = Player::get_model_data();
  let _ = player_model
    .get_appearance_data()
    .lock()
    .unwrap()
    .add_animation_data(animation_data.clone());
  println!("player_hash: {}", player_model.get_hash());

  model_list.push(player_model);

  for _ in 0..(model_count - 1) {
    let mut model = Player::get_model_data();
    let _ = model
      .get_appearance_data()
      .lock()
      .unwrap()
      .add_animation_data(animation_data.clone());

    model_list.push(model)
  }

  let world = StoredWorld::new(model_list);
  world
    .save(PathBuf::from("examples/worlds/large_test_world.wrld"))
    .unwrap();
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
