use crate::general_data::config_builder;
use lazy_static::lazy_static;

lazy_static! {
  pub static ref CONFIG: config_builder::ConfigData = config_builder::get_config()
    .unwrap_or_else(|err| panic!("an error has occurred getting the config: '{err}'"));
}

pub mod defaults;

pub mod general_data {
  pub mod config_builder;
  pub mod coordinates;
  pub mod file_logger;
  pub mod hasher;
  pub mod map_methods;
  pub mod user_input;
}

pub mod objects {
  pub mod hollow_square;
  pub mod object_data;
  pub mod object_movements;
}

pub mod screen {
  pub mod object_screen_data;
  pub mod pixel;
  pub mod pixel_data_types;
  pub mod screen_data;
}
