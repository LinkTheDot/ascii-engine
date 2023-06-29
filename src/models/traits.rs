pub use crate::models::{
  errors::ModelError,
  model_data::{ModelData, Strata},
  sprites::*,
};
pub use model_macros::DisplayModel;
pub use std::sync::{Arc, Mutex, RwLock};

pub trait DisplayModel {
  /// Returns the world placement of the model.
  fn get_position(&self) -> usize;
  /// Returns the very top left position of the model.
  fn get_top_left_position(&self) -> usize;

  /// Returns the dimensions of the model's appearance.
  /// (width, height)
  fn get_sprite_dimensions(&self) -> (usize, usize);

  /// Moves the model to the given position.
  fn absolute_movement(&mut self, new_position: (usize, usize)) -> Vec<ModelData>;
  /// Moves the model a relative amount from it's current position.
  fn relative_movement(&mut self, added_position: (isize, isize)) -> Vec<ModelData>;

  /// Returns the list of collisions the model would have had if it moved to the given location.
  fn absolute_movement_collision_check(&self, new_position: (usize, usize)) -> Vec<ModelData>;
  /// Returns the list of collisions the model would have had if it moved by the given amount.
  fn relative_movement_collision_check(&self, added_position: (isize, isize)) -> Vec<ModelData>;

  /// Returns the value the model uses to classify air in it's appearance.
  fn get_air_char(&self) -> char;

  /// Returns the model's current appearance.
  fn get_sprite(&self) -> String;

  /// Returns a copy of the model's unique hash.
  fn get_unique_hash(&self) -> u64;

  /// Returns a copy of the model's current strata.
  fn get_strata(&self) -> Strata;
  /// Changes the model's current strata.
  ///
  /// # Errors
  ///
  /// - An error is returned when an incorrect strata level is used.
  fn change_strata(&mut self, new_strata: Strata) -> Result<(), ModelError>;

  /// Returns a copy of the given name of the model.
  fn get_name(&self) -> String;
  /// Changes the given name of the model.
  fn change_name(&mut self, new_name: String);

  /// Returns a copy of the ModelData.
  fn get_model_data(&self) -> ModelData;
}
