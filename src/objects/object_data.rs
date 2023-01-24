use crate::general_data::{coordinates::*, hasher};
use crate::objects::errors::*;
pub use crate::objects::traits::*;
use crate::CONFIG;

#[allow(unused)]
use log::debug;

/// This is the data that will be required for the Object derive macro.
///
/// ObjectData contains data such as, the object's unique hash, the position of the
/// defined center point, the strata, and the Sprite.
#[derive(Debug)]
pub struct ObjectData {
  unique_hash: u64,
  /// Based on where the center is.
  /// counts new lines
  object_position: usize,
  /// counts new lines
  top_left_position: usize,
  strata: Strata,
  sprite: Sprite,
}

/// The Strata will be the priority on the screen.
/// That which has a lower Strata, will be above those with higher strata.
///
/// The strata is a range from 0-100, any number outside of that range will
/// not be accepted.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Strata(pub usize);

impl Strata {
  pub fn correct_range(&self) -> bool {
    self.0 <= 100
  }
}

impl ObjectData {
  /// This will create the data for an object.
  /// The data will contain things such as what the object looks like, the hitbox,
  /// what layer the object sits on, the position, and more.
  ///
  /// To create ObjectData you will need the Sprite.
  /// A Sprite contains the data for the object's Skin and Hitbox.
  pub fn new(
    object_position: Coordinates,
    sprite: Sprite,
    strata: Strata,
  ) -> Result<Self, ObjectError> {
    let unique_hash = hasher::get_unique_hash();
    let top_left_position = get_top_left_index_of_skin(
      object_position.coordinates_to_index(CONFIG.grid_width as usize),
      &sprite,
    );

    if !strata.correct_range() {
      return Err(ObjectError::IncorrectStrataRange(strata));
    }

    Ok(Self {
      unique_hash,
      object_position: object_position.coordinates_to_index(CONFIG.grid_width as usize),
      strata,
      sprite,
      top_left_position,
    })
  }

  pub fn get_top_left_index_of_skin(&self) -> usize {
    get_top_left_index_of_skin(self.object_position, &self.sprite)
  }

  pub fn top_left(&self) -> &usize {
    &self.top_left_position
  }

  /// Returns the (width, height) of the current sprite shape.
  pub fn get_sprite_dimensions(&self) -> (usize, usize) {
    let shape = self.sprite.get_shape();
    let rows: Vec<&str> = shape.split('\n').collect();

    let width = rows[0].chars().count();
    let height = rows.len();

    (width, height)
  }

  pub fn change_position(&mut self, new_position: usize) -> Result<(), ObjectError> {
    let (object_width, object_height) = self.get_sprite_dimensions();

    if object_width + (new_position % CONFIG.grid_width as usize) >= CONFIG.grid_width as usize {
      return Err(ObjectError::OutOfBounds(Direction::Right));
    } else if object_height + (new_position / CONFIG.grid_width as usize)
      >= CONFIG.grid_height as usize
    {
      return Err(ObjectError::OutOfBounds(Direction::Down));
    }

    debug!("position: {}", self.object_position);
    let new_top_left = get_top_left_index_of_skin(self.object_position, &self.sprite);

    self.object_position = new_position;
    self.top_left_position = new_top_left;

    Ok(())
  }

  pub fn get_air_char(&self) -> char {
    self.sprite.air_character()
  }

  /// Returns a reference to the unique hash
  pub fn get_unique_hash(&self) -> &u64 {
    &self.unique_hash
  }

  /// Returns a reference to the current position
  pub fn get_object_position(&self) -> &usize {
    &self.object_position
  }

  /// Returns a reference to the String for the object's appearance
  pub fn get_sprite(&self) -> &str {
    self.sprite.get_shape()
  }

  /// Replaces the String for the object's appearance
  pub fn change_sprite(&mut self, new_model: String) {
    *self.sprite.get_mut_shape() = new_model;
  }

  /// Returns a reference to the relative points of the hitbox to
  /// the designated center point of the object's skin.
  pub fn get_hitbox(&self) -> &Vec<(isize, isize)> {
    self.sprite.get_hitbox()
  }

  /// Replaces the object's hitbox with a new one
  pub fn change_hitbox(&mut self, new_hitbox: Hitbox) -> Result<(), ObjectError> {
    self.sprite.change_hitbox(new_hitbox)
  }

  /// Returns a reference to the Strata
  pub fn get_strata(&self) -> &Strata {
    &self.strata
  }

  /// Changes the object's Strata with the given one.
  pub fn change_strata(&mut self, new_strata: Strata) {
    self.strata = new_strata
  }
}

/// Object_position is an index of a frame.
/// This index will account for any newlines.
fn get_top_left_index_of_skin(object_position: usize, sprite: &Sprite) -> usize {
  let relative_coordinates = get_0_0_relative_to_center(sprite);

  // get coordinates of object
  let object_coordinates = object_position.index_to_coordinates(CONFIG.grid_width as usize + 1);

  // get the coordinates for the top left of the object
  let true_top_left_coordinates = (
    object_coordinates.0 as isize + relative_coordinates.0,
    object_coordinates.1 as isize + relative_coordinates.1,
  );

  // convert coordinates to index
  (true_top_left_coordinates.0 + ((CONFIG.grid_width as isize + 1) * true_top_left_coordinates.1))
    as usize
}

fn get_0_0_relative_to_center(sprite: &Sprite) -> (isize, isize) {
  let sprite_rows: Vec<&str> = sprite.get_shape().split('\n').collect();
  let sprite_width = sprite_rows[0].chars().count() as isize;

  let skin_center_index = sprite.get_center_character_index() as isize;
  let skin_center_coordinates = (
    skin_center_index % sprite_width,
    skin_center_index / sprite_width,
  );

  (-skin_center_coordinates.0, -skin_center_coordinates.1)
}

#[cfg(test)]
mod tests {
  use super::*;

  const SHAPE: &str = "x-x\nxcx\nx-x";
  const CENTER_CHAR: char = 'c';
  const CENTER_REPLACEMENT_CHAR: char = '-';
  const AIR_CHAR: char = '-';

  #[test]
  fn get_top_left_coordinates_of_skin_logic() {
    let object_coordinates = (10, 10);
    let object_index = object_coordinates.coordinates_to_index(CONFIG.grid_width as usize + 1);
    let sprite = get_sprite(true);

    let expected_index = 1593;

    let top_left_index = get_top_left_index_of_skin(object_index, &sprite);

    assert_eq!(top_left_index, expected_index);
  }

  #[test]
  fn get_0_0_relative_to_center_logic() {
    let sprite = get_sprite(true);

    let expected_position = (-1, -1);

    let relative_position = get_0_0_relative_to_center(&sprite);

    assert_eq!(relative_position, expected_position);
  }

  fn get_object_data(object_position: (usize, usize), center_is_hitbox: bool) -> ObjectData {
    let sprite = get_sprite(center_is_hitbox);
    let strata = Strata(0);

    match ObjectData::new(object_position, sprite, strata) {
      Ok(object_data) => object_data,
      Err(error) => panic!("An error has occurred while getting the object data: {error:?}"),
    }
  }

  fn get_sprite(center_is_hitbox: bool) -> Sprite {
    let skin = get_skin();
    let hitbox = get_hitbox(center_is_hitbox);

    match Sprite::new(skin, hitbox) {
      Ok(sprite) => sprite,
      Err(error) => panic!("An error has occurred while getting the sprite: '{error:?}"),
    }
  }

  fn get_skin() -> Skin {
    match Skin::new(SHAPE, CENTER_CHAR, CENTER_REPLACEMENT_CHAR, AIR_CHAR) {
      Ok(skin) => skin,
      Err(error) => panic!("An error has occurred while getting the skin: '{error:?}'"),
    }
  }

  fn get_hitbox(center_is_hitbox: bool) -> Hitbox {
    let shape = "xxx\n-c-";

    Hitbox::new(shape, 'c', '-', center_is_hitbox)
  }
}
