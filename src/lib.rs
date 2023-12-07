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
  pub mod traits;
}

pub mod screen {
  pub mod model_manager;
  pub mod model_storage;
  pub mod printer;
  pub mod screen_data;
  pub mod stored_worlds;
}
