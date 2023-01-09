pub use crate::objects::{errors::ObjectError, object_data::Strata, sprites::*};
pub use object_macros::Object;

pub trait Object {
  fn get_position(&self) -> &usize;
  fn get_top_left_position(&self) -> &usize;

  fn get_sprite_dimensions(&self) -> (usize, usize);

  fn move_to(&mut self, new_position: (usize, usize)) -> Result<(), ObjectError>;
  fn move_by(&mut self, added_position: (isize, isize));

  fn get_air_char(&self) -> char;

  fn get_sprite(&self) -> &str;
  fn change_sprite(&mut self, new_model: String);

  fn get_hitbox(&self) -> &Vec<(isize, isize)>;
  fn change_hitbox(&mut self, new_hitbox_model: Hitbox) -> Result<(), ObjectError>;

  fn get_unique_hash(&self) -> &u64;

  fn get_strata(&self) -> &Strata;
  fn change_strata(&mut self, new_strata: Strata) -> Result<(), ObjectError>;
}
