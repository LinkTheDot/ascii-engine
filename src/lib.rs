use crate::general_data::config_builder;
use lazy_static::lazy_static;

lazy_static! {
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

pub mod objects {
  pub mod errors;
  pub mod object_data;
  pub mod sprites;
  pub mod traits;
}

pub mod screen {
  pub mod errors;
  pub mod objects;
  pub mod screen_data;
}
