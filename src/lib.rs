use crate::general_data::config_builder;
use lazy_static::lazy_static;

lazy_static! {
  // Only way this can cause an error is if the code for the config builder was done wrong.
  pub static ref CONFIG: config_builder::ConfigData = config_builder::get_config().unwrap();
}

pub mod defaults;
pub mod errors;
pub mod prelude;

pub mod general_data {
  pub mod config_builder;
  pub mod coordinates;
  pub mod file_logger;
  pub mod hasher;
  pub mod map_methods;
  pub mod user_input;
}

pub mod models {
  pub mod errors;
  pub mod hitboxes;
  pub mod model_data;
  pub mod model_file_parser;
  pub mod sprites;
  pub mod traits;
}

pub mod screen {
  pub mod errors;
  pub mod models;
  pub mod screen_data;
}
