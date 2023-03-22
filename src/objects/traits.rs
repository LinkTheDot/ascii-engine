pub use crate::objects::{
  errors::ObjectError,
  object_data::{ObjectData, Strata},
  sprites::*,
};
pub use crate::screen::objects::Objects;
pub use object_macros::Object;
pub use std::sync::{Arc, Mutex, RwLock};

pub trait Object {
  /// Returns the center position of the object.
  fn get_position(&self) -> usize;
  /// Returns the very top left position of the object.
  fn get_top_left_position(&self) -> usize;

  /// Returns the dimensions of the object's appearance.
  /// (width, height)
  fn get_sprite_dimensions(&self) -> (usize, usize);

  /// Moves the object to the given position.
  fn move_to(&mut self, new_position: (usize, usize)) -> Vec<Arc<Mutex<ObjectData>>>;
  /// Moves the object a relative amount from it's current position.
  fn move_by(&mut self, added_position: (isize, isize)) -> Vec<Arc<Mutex<ObjectData>>>;

  /// Returns the value the object uses to classify air in it's appearance.
  fn get_air_char(&self) -> char;

  /// Returns the object's current appearance.
  fn get_sprite(&self) -> String;
  /// Changes the appearance of the object.
  fn change_sprite(&mut self, new_model: String);

  // /// Returns the list of relative points around the object's center that act
  // /// as the object's hitbox.
  // fn get_hitbox(&self) -> Vec<(isize, isize)>;
  // ///
  // fn change_hitbox(&mut self, new_hitbox_model: HitboxCreationData) -> Result<(), ObjectError>;

  /// Returns a copy of the object's unique hash.
  fn get_unique_hash(&self) -> u64;

  /// Returns a copy of the object's current strata.
  fn get_strata(&self) -> Strata;
  /// Changes the object's current strata.
  ///
  /// # Errors
  ///
  /// An error is returned when an incorrect strata level is used.
  fn change_strata(&mut self, new_strata: Strata) -> Result<(), ObjectError>;

  /// Returns a copy of the given name of the object.
  fn get_name(&self) -> String;
  /// Changes the given name of the object.
  fn change_name(&mut self, new_name: String);

  /// Returns a copy of the ObjectData.
  fn get_object_data(&self) -> Arc<Mutex<ObjectData>>;

  fn assign_object_list(&mut self, object_list: Arc<RwLock<Objects>>);
}
