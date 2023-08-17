//! This is the crate in which all data structures used in the graphics engine live.

pub mod models {
  pub mod animation;
  pub mod errors;
  pub mod hitboxes;
  pub mod model_data;
  pub mod sprites;
  pub mod strata;
}

pub mod screen {
  pub mod errors;
  pub mod model_storage;
}

pub mod errors;
pub mod prelude;
