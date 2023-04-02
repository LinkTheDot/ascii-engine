use crate::general_data::{coordinates::*, hasher};
use crate::models::errors::*;
use crate::models::hitboxes::*;
use crate::models::model_file_parser::ModelParser;
pub use crate::models::traits::*;
use crate::screen::models::Models;
use crate::CONFIG;
use guard::guard;
use std::sync::{Arc, MutexGuard, RwLock};
use std::{fs::File, path::Path};

#[allow(unused)]
use log::debug;

/// This is the data that will be required for the Model derive macro.
///
/// ModelData is a collection of all data required for the screen to display a model.
#[derive(Debug)]
pub struct ModelData {
  internal_data: Arc<Mutex<InternalModelData>>,
}

#[derive(Debug)]
struct InternalModelData {
  unique_hash: u64,
  assigned_name: String,
  /// Relative position of the top left to the model's world placement
  placement_anchor: (isize, isize),
  /// counts new lines
  position_in_frame: usize,
  strata: Strata,
  sprite: Sprite,
  hitbox: Hitbox,
  /// Exists only when models are placed on the screen
  existing_models: Option<Arc<RwLock<Models>>>,
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

impl Clone for ModelData {
  fn clone(&self) -> Self {
    Self {
      internal_data: Arc::clone(&self.internal_data),
    }
  }
}

impl ModelData {
  fn get_internal_data(&self) -> MutexGuard<InternalModelData> {
    self.internal_data.lock().unwrap()
  }

  /// This will create the data for a model.
  /// The data will contain things such as what the model looks like, the hitbox,
  /// what layer the model sits on, the position, and more.
  ///
  /// To create ModelData you will need the Sprite.
  /// A Sprite contains the data for the model's Skin and Hitbox.
  pub fn new(
    model_position: Coordinates,
    sprite: Sprite,
    hitbox_data: HitboxCreationData,
    strata: Strata,
    assigned_name: String,
  ) -> Result<Self, ModelError> {
    let internal_data =
      InternalModelData::new(model_position, sprite, hitbox_data, strata, assigned_name)?;

    Ok(Self {
      internal_data: Arc::new(Mutex::new(internal_data)),
    })
  }

  pub fn from_file(
    model_file_path: &Path,
    frame_position: (usize, usize),
  ) -> Result<Self, ModelError> {
    let model_file = File::open(model_file_path);

    match model_file {
      Ok(file) => ModelParser::parse(file, frame_position),
      Err(_) => {
        let file_path = model_file_path
          .file_name()
          .map(|path_string| path_string.to_owned());

        let error = ModelCreationError::ModelFileDoesntExist(file_path);

        Err(ModelError::ModelCreationError(error))
      }
    }
  }

  pub fn top_left(&self) -> usize {
    let internal_data = self.get_internal_data();

    internal_data.position_in_frame
  }

  /// Returns the (width, height) of the current sprite shape.
  pub fn get_sprite_dimensions(&self) -> (usize, usize) {
    let internal_data = self.get_internal_data();

    let model_skin_shape = internal_data.sprite.get_shape();
    let sprite_skin_rows: Vec<&str> = model_skin_shape.split('\n').collect();

    let sprite_skin_width = sprite_skin_rows[0].chars().count();
    let sprite_skin_height = sprite_skin_rows.len();

    (sprite_skin_width, sprite_skin_height)
  }

  /// Changes the placement_anchor and top left position of the model.
  ///
  /// Input is based on the frame_position aka top left position of the model.
  pub fn change_position(&mut self, new_position: usize) {
    let mut internal_data = self.get_internal_data();

    let new_frame_anchor_index = new_position as isize
      + internal_data.placement_anchor.0
      + (internal_data.placement_anchor.1 * (CONFIG.grid_width as isize + 1));

    let (frame_index, new_anchor) =
      get_position_data(new_frame_anchor_index as usize, &internal_data.sprite);

    internal_data.position_in_frame = frame_index;
    internal_data.placement_anchor = new_anchor;
  }

  /// Returns what the sprite classifies as air.
  pub fn get_air_char(&self) -> char {
    let internal_data = self.get_internal_data();

    internal_data.sprite.air_character()
  }

  /// Returns a reference to the unique hash
  pub fn get_unique_hash(&self) -> u64 {
    let internal_data = self.get_internal_data();

    internal_data.unique_hash
  }

  /// Returns a reference to the current position
  pub fn get_model_position(&self) -> usize {
    let internal_data = self.get_internal_data();

    (internal_data.position_in_frame as isize
      + internal_data.placement_anchor.0
      + (internal_data.placement_anchor.1 * (CONFIG.grid_width as isize + 1))) as usize
  }

  /// Returns a reference to the String for the model's appearance
  pub fn get_sprite(&self) -> String {
    let internal_data = self.get_internal_data();

    internal_data.sprite.get_shape().to_string()
  }

  /// Replaces the String for the model's appearance
  pub fn change_sprite(&mut self, new_model: String) {
    let mut internal_data = self.get_internal_data();

    *internal_data.sprite.get_mut_shape() = new_model;
  }

  /// Returns a reference to the Strata
  pub fn get_strata(&self) -> Strata {
    let internal_data = self.get_internal_data();

    internal_data.strata
  }

  /// Changes the model's Strata with the given one.
  pub fn change_strata(&mut self, new_strata: Strata) {
    let mut internal_data = self.get_internal_data();

    internal_data.strata = new_strata
  }

  pub fn change_name(&mut self, new_name: String) {
    let mut internal_data = self.get_internal_data();

    internal_data.assigned_name = new_name
  }

  pub fn get_name(&self) -> String {
    let internal_data = self.get_internal_data();

    internal_data.assigned_name.clone()
  }

  pub fn check_collisions_against_all_models(&self) -> Vec<ModelData> {
    let internal_data = self.get_internal_data();

    let mut collision_list = vec![];

    if let Some(existing_models) = &internal_data.existing_models {
      let existing_models_read_lock = existing_models.read().unwrap();

      for (hash, model_data) in existing_models_read_lock.get_model_list() {
        if hash == &internal_data.unique_hash {
          continue;
        }

        let other_model_internal_data = model_data.get_internal_data();

        if internal_data.check_model_collision(&other_model_internal_data) {
          drop(other_model_internal_data);

          collision_list.push(model_data.clone());
        }
      }
    }

    collision_list
  }

  pub fn assign_model_list(&mut self, model_list: Arc<RwLock<Models>>) {
    let mut internal_data = self.get_internal_data();

    internal_data.existing_models = Some(model_list);
  }

  pub fn fix_model_strata(&self) -> Result<(), ModelError> {
    let internal_data = self.get_internal_data();
    guard!( let Some(model_list) = internal_data.existing_models.as_ref() else { return Err(ModelError::ModelDoesntExist) } );

    let mut model_list_guard = model_list.write().unwrap();

    model_list_guard.fix_strata_list()
  }
}

impl InternalModelData {
  fn new(
    model_position: Coordinates,
    sprite: Sprite,
    hitbox_data: HitboxCreationData,
    strata: Strata,
    assigned_name: String,
  ) -> Result<Self, ModelError> {
    let unique_hash = hasher::get_unique_hash();
    let position_data = get_position_data(
      model_position.coordinates_to_index(CONFIG.grid_width as usize + 1),
      &sprite,
    );
    let hitbox = Hitbox::from(hitbox_data, position_data.1)?;

    if !strata.correct_range() {
      return Err(ModelError::IncorrectStrataRange(strata));
    }

    Ok(Self {
      unique_hash,
      assigned_name,
      placement_anchor: position_data.1,
      strata,
      sprite,
      position_in_frame: position_data.0,
      hitbox,
      existing_models: None,
    })
  }

  /// Checks if any point in the model collides with another model.
  /// If a collision is detection the point of the collision will be returned.
  /// Otherwise if there was no collision None will be returned.
  pub fn check_model_collision(&self, other_model: &Self) -> bool {
    if self.hitbox.is_empty() || other_model.hitbox.is_empty() {
      return false;
    }

    let (self_hitbox_x, self_hitbox_y) = self.hitbox.get_hitbox_position(self.position_in_frame);
    let (other_hitbox_x, other_hitbox_y) = other_model
      .hitbox
      .get_hitbox_position(other_model.position_in_frame);

    let (self_hitbox_width, self_hitbox_height) = self.hitbox.get_dimensions();
    let (other_hitbox_width, other_hitbox_height) = other_model.hitbox.get_dimensions();

    // x1 < x2 + w2
    // x2 < x1 + w1
    // y1 < y2 + h2
    // y2 < y1 + h1
    self_hitbox_x < other_hitbox_x + other_hitbox_width
      && other_hitbox_x < self_hitbox_x + self_hitbox_width
      && self_hitbox_y < other_hitbox_y + other_hitbox_height
      && other_hitbox_y < self_hitbox_y + self_hitbox_height
  }
}

/// A model's position is an index of a frame.
/// This index will account for any newlines.
///
/// Takes the world placement of a model and returns it's index in a frame and
/// the relative distance of the FrameIndex to the WorldPlacement.
///
/// Returns (TopLeftFrameIndex, WorldPlacementAnchor)
fn get_position_data(model_position: usize, sprite: &Sprite) -> (usize, (isize, isize)) {
  let relative_coordinates = get_frame_index_to_world_placement_anchor(sprite);

  let true_width = CONFIG.grid_width as isize + 1;

  let top_left_position = (model_position as isize
    + relative_coordinates.0
    + (true_width * relative_coordinates.1)) as usize;

  (
    top_left_position,
    (-relative_coordinates.0, -relative_coordinates.1),
  )
}

///
fn get_frame_index_to_world_placement_anchor(sprite: &Sprite) -> (isize, isize) {
  let sprite_rows: Vec<&str> = sprite.get_shape().split('\n').collect();
  let sprite_width = sprite_rows[0].chars().count() as isize;

  let skin_anchor_index = sprite.get_anchor_character_index() as isize;
  let skin_anchor_coordinates = (
    skin_anchor_index % sprite_width,
    skin_anchor_index / sprite_width,
  );

  (-skin_anchor_coordinates.0, -skin_anchor_coordinates.1)
}

impl PartialEq for InternalModelData {
  fn eq(&self, other: &Self) -> bool {
    self.placement_anchor == other.placement_anchor
      && self.position_in_frame == other.position_in_frame
      && self.strata == other.strata
      && self.sprite == other.sprite
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const SHAPE: &str = "x-x\nxcx\nx-x";
  const ANCHOR_CHAR: char = 'c';
  const ANCHOR_REPLACEMENT_CHAR: char = '-';
  const AIR_CHAR: char = '-';

  #[test]
  #[ignore]
  fn get_top_left_coordinates_of_skin_logic() {
    // let (x, y) = (10, 10);
    // let model_index = (x, y).coordinates_to_index(CONFIG.grid_width as usize + 1);
    // let sprite = get_sprite(true);
    //
    // let expected_index = ((CONFIG.grid_width + 1) as usize * (y - 1)) + (x - 1);
    //
    // let top_left_index = get_position_data(model_index, &sprite);
    //
    // assert_eq!(top_left_index, expected_index);
  }

  #[test]
  fn get_frame_index_to_world_placement_anchor_logic() {
    let sprite = get_sprite();

    let expected_position = (-1, -1);

    let relative_position = get_frame_index_to_world_placement_anchor(&sprite);

    assert_eq!(relative_position, expected_position);
  }

  //
  // Functions used for tests

  fn get_sprite() -> Sprite {
    let skin = get_skin();

    Sprite::new(skin).unwrap()
  }

  fn get_skin() -> Skin {
    Skin::new(SHAPE, ANCHOR_CHAR, ANCHOR_REPLACEMENT_CHAR, AIR_CHAR).unwrap()
  }
}
