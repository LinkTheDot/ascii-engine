use crate::general_data::{coordinates::*, hasher};
use crate::objects::hitboxes::*;
pub use crate::objects::traits::*;
use crate::screen::objects::Objects;
use crate::CONFIG;
use guard::guard;
use std::sync::{Arc, RwLock};

#[allow(unused)]
use log::debug;

/// This is the data that will be required for the Object derive macro.
///
/// ObjectData contains data such as, the object's unique hash, the position of the
/// defined center point, the strata, and the Sprite.
#[derive(Debug)]
pub struct ObjectData {
  unique_hash: u64,
  assigned_name: String,
  /// Relative position of the center from the top left of the skin
  relative_center: (isize, isize),
  /// counts new lines
  top_left_position: usize,
  strata: Strata,
  sprite: Sprite,
  hitbox: Hitbox,
  /// Exists only when objects are placed on the screen
  // last worked here
  existing_objects: Option<Arc<RwLock<Objects>>>,
}

/// The Strata will be the priority on the screen.
/// That which has a lower Strata, will be behind those with a higher strata.
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
  ///
  /// OBJECT CREATION WILL BE TURNED INTO A FILE FORMAT IN THE FUTURE.
  pub fn new(
    object_position: Coordinates,
    sprite: Sprite,
    hitbox_data: HitboxCreationData,
    strata: Strata,
    assigned_name: String,
  ) -> Result<Self, ObjectError> {
    let unique_hash = hasher::get_unique_hash();
    let position_data = get_position_data(
      object_position.coordinates_to_index(CONFIG.grid_width as usize + 1),
      &sprite,
    );
    let hitbox = Hitbox::from(hitbox_data, position_data.1)?;

    if !strata.correct_range() {
      return Err(ObjectError::IncorrectStrataRange(strata));
    }

    Ok(Self {
      unique_hash,
      assigned_name,
      relative_center: position_data.1,
      strata,
      sprite,
      top_left_position: position_data.0,
      hitbox,
      existing_objects: None,
    })
  }

  /// Returns the index of the object from the top left position.
  pub fn top_left(&self) -> &usize {
    &self.top_left_position
  }

  /// Returns the (width, height) of the current sprite shape.
  pub fn get_sprite_dimensions(&self) -> (usize, usize) {
    let object_skin_shape = self.sprite.get_shape();
    let sprite_skin_rows: Vec<&str> = object_skin_shape.split('\n').collect();

    let sprite_skin_width = sprite_skin_rows[0].chars().count();
    let sprite_skin_height = sprite_skin_rows.len();

    (sprite_skin_width, sprite_skin_height)
  }

  /// Changes the center and top left position of the object.
  ///
  /// Input is based on the top left of the object
  pub fn change_position(&mut self, new_position: usize) {
    let new_center_index = new_position as isize
      + self.relative_center.0
      + (self.relative_center.1 * (CONFIG.grid_width as isize + 1));

    let position_data = get_position_data(new_center_index as usize, &self.sprite);

    self.top_left_position = position_data.0;
    self.relative_center = position_data.1;
  }

  /// Returns what the sprite classifies as air.
  pub fn get_air_char(&self) -> char {
    self.sprite.air_character()
  }

  /// Returns a reference to the unique hash
  pub fn get_unique_hash(&self) -> &u64 {
    &self.unique_hash
  }

  /// Returns a reference to the current position
  pub fn get_object_position(&self) -> usize {
    (self.top_left_position as isize
      + self.relative_center.0
      + (self.relative_center.1 * (CONFIG.grid_width as isize + 1))) as usize
  }

  /// Returns a reference to the String for the object's appearance
  pub fn get_sprite(&self) -> &str {
    self.sprite.get_shape()
  }

  /// Replaces the String for the object's appearance
  pub fn change_sprite(&mut self, new_model: String) {
    *self.sprite.get_mut_shape() = new_model;
  }

  /// Returns a reference to the Strata
  pub fn get_strata(&self) -> &Strata {
    &self.strata
  }

  /// Changes the object's Strata with the given one.
  pub fn change_strata(&mut self, new_strata: Strata) {
    self.strata = new_strata
  }

  pub fn change_name(&mut self, new_name: String) {
    self.assigned_name = new_name
  }

  pub fn get_name(&self) -> &str {
    &self.assigned_name
  }

  pub fn check_collisions_against_all_objects(&self) -> Vec<Arc<Mutex<ObjectData>>> {
    let mut collision_list = vec![];

    if let Some(existing_objects) = &self.existing_objects {
      let existing_objects_read_lock = existing_objects.read().unwrap();

      for (hash, object_data) in existing_objects_read_lock.get_object_list() {
        if hash == &self.unique_hash {
          continue;
        }

        let object_data_guard = object_data.lock().unwrap();

        if self.check_object_collision(&object_data_guard) {
          drop(object_data_guard);

          collision_list.push(Arc::clone(object_data));
        }
      }
    }

    collision_list
  }

  /// Checks if any point in the object collides with another object.
  /// If a collision is detection the point of the collision will be returned.
  /// Otherwise if there was no collision None will be returned.
  pub fn check_object_collision(&self, other_object: &Self) -> bool {
    if self.hitbox.is_empty() || other_object.hitbox.is_empty() {
      return false;
    }

    let (self_hitbox_x, self_hitbox_y) = self.hitbox.get_hitbox_position(self.top_left_position);
    let (other_hitbox_x, other_hitbox_y) = other_object
      .hitbox
      .get_hitbox_position(other_object.top_left_position);

    let (self_hitbox_width, self_hitbox_height) = self.hitbox.get_dimensions();
    let (other_hitbox_width, other_hitbox_height) = other_object.hitbox.get_dimensions();

    // x1 < x2 + w2
    // x2 < x1 + w1
    // y1 < y2 + h2
    // y2 < y1 + h1
    self_hitbox_x < other_hitbox_x + other_hitbox_width
      && other_hitbox_x < self_hitbox_x + self_hitbox_width
      && self_hitbox_y < other_hitbox_y + other_hitbox_height
      && other_hitbox_y < self_hitbox_y + self_hitbox_height
  }

  pub fn assign_object_list(&mut self, object_list: Arc<RwLock<Objects>>) {
    self.existing_objects = Some(object_list);
  }

  pub fn fix_object_strata(&self) -> Result<(), ObjectError> {
    guard!( let Some(object_list) = self.existing_objects.as_ref() else { return Err(ObjectError::ObjectDoesntExist) } );

    let mut object_list_guard = object_list.write().unwrap();

    object_list_guard.fix_strata_list()
  }

  // Not needed for now, if collision checks become an issue for compute times in the future then implement this.
  //
  // pub fn calculate_section(top_left_position: usize) -> u8 {
  //   // section_x_divisor = grid_one_width / grid_two_width
  //   let section_x_divisor = CONFIG.grid_width as f32 / CONFIG.grid_sections_width as f32;
  //   // section_y_divisor = grid_one_height / grid_two_height
  //   let section_y_divisor = CONFIG.grid_height as f32 / CONFIG.grid_sections_height as f32;
  //
  //   let (mut object_x, object_y) =
  //     top_left_position.index_to_coordinates(CONFIG.grid_width as usize + 1);
  //
  //   // accounts for the fact that the position of objects is based off of "grid_width + 1".
  //   // This is needed because the calculation for the section is based off the true grid width.
  //   debug!("{} -= {}", object_x, object_y);
  //   object_x -= object_y;
  //
  //   // x_position_in_grid_two = floor(object_x / section_x_divisor)
  //   let section_x = (object_x as f32 / section_x_divisor).floor();
  //   // y_position_in_grid_two = floor(object_y / section_y_divisor)
  //   let section_y = (object_y as f32 / section_y_divisor).floor();
  //
  //   // convert to an index
  //   (section_x + (CONFIG.grid_sections_width as f32 * section_y)) as u8
  // }
}

/// An object's position is an index of a frame.
/// This index will account for any newlines.
///
/// Returns (top_left_position, relative_position_of_object_center)
fn get_position_data(object_position: usize, sprite: &Sprite) -> (usize, (isize, isize)) {
  let relative_coordinates = get_relative_position_of_center_to_top_left(sprite);

  let true_width = CONFIG.grid_width as isize + 1;

  let top_left_position = (object_position as isize
    + relative_coordinates.0
    + (true_width * relative_coordinates.1)) as usize;

  (
    top_left_position,
    (-relative_coordinates.0, -relative_coordinates.1),
  )
}

fn get_relative_position_of_center_to_top_left(sprite: &Sprite) -> (isize, isize) {
  let sprite_rows: Vec<&str> = sprite.get_shape().split('\n').collect();
  let sprite_width = sprite_rows[0].chars().count() as isize;

  let skin_center_index = sprite.get_center_character_index() as isize;
  let skin_center_coordinates = (
    skin_center_index % sprite_width,
    skin_center_index / sprite_width,
  );

  (-skin_center_coordinates.0, -skin_center_coordinates.1)
}

impl PartialEq for ObjectData {
  fn eq(&self, other: &Self) -> bool {
    self.relative_center == other.relative_center
      && self.top_left_position == other.top_left_position
      && self.strata == other.strata
      && self.sprite == other.sprite
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const SHAPE: &str = "x-x\nxcx\nx-x";
  const CENTER_CHAR: char = 'c';
  const CENTER_REPLACEMENT_CHAR: char = '-';
  const AIR_CHAR: char = '-';

  #[test]
  #[ignore]
  fn get_top_left_coordinates_of_skin_logic() {
    // let (x, y) = (10, 10);
    // let object_index = (x, y).coordinates_to_index(CONFIG.grid_width as usize + 1);
    // let sprite = get_sprite(true);
    //
    // let expected_index = ((CONFIG.grid_width + 1) as usize * (y - 1)) + (x - 1);
    //
    // let top_left_index = get_position_data(object_index, &sprite);
    //
    // assert_eq!(top_left_index, expected_index);
  }

  #[test]
  fn get_0_0_relative_to_center_logic() {
    let sprite = get_sprite();

    let expected_position = (-1, -1);

    let relative_position = get_relative_position_of_center_to_top_left(&sprite);

    assert_eq!(relative_position, expected_position);
  }

  //
  // Functions used for tests

  fn get_sprite() -> Sprite {
    let skin = get_skin();

    Sprite::new(skin).unwrap()
  }

  fn get_skin() -> Skin {
    Skin::new(SHAPE, CENTER_CHAR, CENTER_REPLACEMENT_CHAR, AIR_CHAR).unwrap()
  }
}
