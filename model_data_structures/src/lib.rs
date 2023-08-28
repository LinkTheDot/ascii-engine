//! This is the crate in which all data structures used in the graphics engine live.

use lazy_static::lazy_static;

lazy_static! {
  // Only way this can cause an error is if the code for the config builder was done wrong.
  pub static ref CONFIG: config_builder::ConfigData = config_builder::get_config().unwrap();
}

pub mod models {
  pub mod animation;
  pub mod errors;
  pub mod hitboxes;
  pub mod model_data;
  pub mod model_file_parser;
  pub mod model_movements;
  pub mod sprites;
  pub mod strata;
}

pub mod screen {
  pub mod errors;
  pub mod model_storage;
}

pub mod config_builder;
pub mod errors;
pub mod prelude;
