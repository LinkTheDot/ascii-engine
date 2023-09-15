#![doc = include_str!("../README.md")]

// use crate::general_data::config_builder;
use lazy_static::lazy_static;
use model_data_structures::config_builder;

lazy_static! {
  // Only way this can cause an error is if the code for the config builder was done wrong.
  pub static ref CONFIG: config_builder::ConfigData = config_builder::get_config().unwrap();
}

pub mod defaults;
pub mod errors;
pub mod prelude;

pub mod general_data {
  pub mod file_logger;
  pub mod user_input;
}

pub mod models {
  pub mod animation_file_parser;
  pub mod animation_thread;
  pub mod traits;
}

pub mod screen {
  pub mod model_manager;
  pub mod model_storage;
  pub mod printer;
  pub mod screen_data;
  pub mod stored_worlds;
}

#[test]
fn temp_create_file() {
  use crate::screen::screen_data::*;
  use model_data_structures::models::testing_data::*;
  use model_data_structures::prelude::*;

  let animation_frames = AnimationFrames::new(
    TestingData::get_test_frames(vec![
      ("xxxxx\nxxaxx\nxxxxx".to_string(), 2, 'x'),
      ("/xxxx\nxxaxx\nxxxxx".to_string(), 2, 'x'),
      ("x/xxx\n/xaxx\nxxxxx".to_string(), 2, 'x'),
      ("xx/xx\nx/axx\n/xxxx".to_string(), 2, 'x'),
      ("xxx/x\nxxaxx\nx/xxx".to_string(), 2, '/'),
      ("xxxx/\nxxa/x\nxx/xx".to_string(), 2, 'x'),
      ("xxxxx\nxxax/\nxxx/x".to_string(), 2, 'x'),
      ("xxxxx\nxxaxx\nxxxx/".to_string(), 2, 'x'),
    ]),
    AnimationLoopCount::Forever,
    None,
  );
  let model = TestingData::new_test_model_with_animation(
    (10, 10),
    vec![("test".to_string(), animation_frames)],
  );
  println!("hash: {}", model.get_hash());

  let mut screen = ScreenData::new();
  screen.add_model(model).unwrap();

  let world = screen.reset_world();
  let world_path = std::path::PathBuf::from("examples/worlds/test_world.wrld");

  world.save(world_path).unwrap();
}
