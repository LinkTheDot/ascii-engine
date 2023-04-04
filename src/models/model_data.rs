use crate::general_data::{coordinates::*, hasher};
use crate::models::errors::*;
use crate::models::hitboxes::*;
use crate::models::model_file_parser::ModelParser;
pub use crate::models::traits::*;
use crate::screen::models::InternalModels;
use crate::CONFIG;
use guard::guard;
use std::ffi::OsStr;
use std::sync::{Arc, MutexGuard, RwLock};
use std::{fs::File, path::Path};

#[allow(unused)]
use log::debug;

/// ModelData contains everything the screen needs to know for a model to be placed in the world.
///
/// Creating an instance of ModelData for your model requires you to create a model file.
///
/// # Model File Creation Requirements
///
/// To create a model file you'll start by making a file named as such:
/// ```no_run,bash,ignore
///    model_name.model
/// ```
///
/// Once you have your model file, you'll need to feed it the data for your model.
///
/// A model file is formatted as such:
/// ```no_run,bash,ignore
/// - Header
/// - Data
/// - Spacer
/// ```
///
/// The required headers are
/// ```no_run,bash,ignore
/// - Skin
/// - Appearance
/// - Hitbox_Dimensions
/// ```
///
/// The available spacer is
/// ```no_run,bash,ignore
/// -=--=-
/// ```
///
/// The data can differ from header to header.
///
/// ## Skin Data
/// NONE of these fields can be ``=``.
/// The ``name`` field can NOT contain ``'``.
///
/// The required fields under the "Skin" header are as such:
///
/// - anchor (This is the assigned character for a model's hitbox and world placement anchor)
///```no_run,bash,ignore
///   anchor="a"
///```
///
/// - anchor_replacement (This is the character that will replace the anchor character. This can be thought of as the "fix" for when you're building out the appearance of a model)
///```no_run,bash,ignore
///   anchor_replacement="-"
///```
///
/// - air (This is the character that will be designated as the model's air. Air will be transparent on the screen)
///```no_run,bash,ignore
///   air="-"
///```
///
/// - name (This is the assigned name for the model. The name can be used to identify collisions and what you want to do depending on the collided model)
///```no_run,bash,ignore
///   name="Square"
///```
///
/// - strata (The strata is what layer the model is on the screen. Refer to [`Strata`](Strata)) for more information)
///```no_run,bash,ignore
///   strata="95"
///```
///
///
/// ## Appearance
///
/// This will be how your model looks on the screen.
/// The appearance must be rectangular in shape.
///
/// To build a non-rectangular shape, you can use the air character defined under the "Skin" header to have a transparent pixel.
///
/// Your model's appearance requires you to have an anchor character.
/// The anchor character will be used to dictate where the model is placed on the screen.
/// The anchor also dictates where the anchor for Hitbox_Dimensions will be placed relative to your model's appearance.
///
/// The appearance field will look something like this:
///```no_run,bash,ignore
///=====
///|-a-|
///=====
///```
///
/// ## Hitbox_Dimensions
///
/// The Hitbox_Dimensions field is very similar to the Appearance.
///
/// Just like the appearance, the Hitbox_Dimensions requires one anchor character to be assigned within it.
/// The anchor character will dictate where the hitbox is placed relative to the anchor in the appearance.
///
/// This will dictate the size of your model's hitbox, and it must be a rectangular shape.
/// Any character that isn't the anchor will be accepted for dictating the dimensions of the hitbox.
///
/// The Hitbox_Dimensions field will look something like this:
///```no_run,bash,ignore
///=====
///==a==
///=====
///```
///
/// # Creating a file
///
/// Now that all of the information required to make a model has been defined, we can get to actually making one.
///
/// Here's a mock model file of a simple square model.
///
/// ```no_run,bash,ignore
/// Skin
/// anchor="a"
/// anchor_replacement="-"
/// air="-"
/// name="Square"
/// strata="95"
/// -=--=-
/// Appearance
/// =====
/// |-a-|
/// =====
/// -=--=-
/// Hitbox_Dimensions
/// =====
/// ==a==
/// =====
/// -=--=-
/// ```
///
/// First we define the ``Skin`` header.
///
/// Next we assign our character for the anchor.
///
/// With our anchor assigned, we now assign a character to replace it so we don't have a random ``a`` on our model.
/// Here we go with the air character, because I want the square to have a hole in the center.
///
/// From there we assign air to ``-``,
///
/// We give our model a name, in this case ``Square``.
///
/// Lastly we give it a strata of ``95``, meaning anything that overlaps with our square, and that has a strata < 95, will be under the square.
///
/// # Comments
///
/// It should be known that the character sequence ``+- `` anywhere in a line is reserved for comments.
/// This means if you put ``+- `` anywhere on a line in your model file, that line will be ignored by the parser.
#[derive(Debug)]
pub struct ModelData {
  internal_data: Arc<Mutex<InternalModelData>>,
}

/// This is the internal storage of ModelData.
///
/// Everything from the model's unique hash to it's hitbox data is stored here.
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
  existing_models: Option<Arc<RwLock<InternalModels>>>,
}

/// The Strata will be the priority on the screen.
/// That which has a lower Strata, will be behind those with a higher strata.
///
/// The strata is a range from 0-100, any number outside of that range will
/// not be accepted.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Strata(pub usize);

impl Strata {
  /// Returns true if the given strata is withing 0-100.
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
  /// Returns a MutexGuard of the [`InternalModelData`](InternalModelData).
  fn get_internal_data(&self) -> MutexGuard<InternalModelData> {
    self.internal_data.lock().unwrap()
  }

  /// This will create the data for a model.
  /// The data will contain things such as what the model looks like, the hitbox,
  /// what strata or layer the model sits on, the position, and more.
  ///
  /// For information on creating a model, refer to [`ModelData`](ModelData).
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

  /// This is the main way you'll be creating an instance of ModelData.
  ///
  /// For creating your own model file refer to [`ModelData`](ModelData).
  ///
  /// Takes the Path for the model file, and the position you wish to place the model in the world.
  ///
  /// # Errors
  ///
  /// Returns an error when the file has the wrong extension.
  /// Returns an error when the file didn't exist.
  /// Returns an error when the model file was build incorrectly. [`Errors when parsing model files`](crate::models::errors::ModelCreationError).
  pub fn from_file(
    model_file_path: &Path,
    frame_position: (usize, usize),
  ) -> Result<Self, ModelError> {
    if model_file_path.extension() != Some(OsStr::new("model")) {
      return Err(ModelError::NonModelFile);
    }

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

  /// Returns the frame position of a model.
  pub fn top_left(&self) -> usize {
    let internal_data = self.get_internal_data();

    internal_data.position_in_frame
  }

  /// Returns the (width, height) of the current sprite's shape.
  pub fn get_sprite_dimensions(&self) -> (usize, usize) {
    let internal_data = self.get_internal_data();

    let model_skin_shape = internal_data.sprite.get_shape();
    let sprite_skin_rows: Vec<&str> = model_skin_shape.split('\n').collect();

    let sprite_skin_width = sprite_skin_rows[0].chars().count();
    let sprite_skin_height = sprite_skin_rows.len();

    (sprite_skin_width, sprite_skin_height)
  }

  /// Moves a model to the given world position.
  ///
  /// Returns a list of references to model collisions that occurred in the new location.
  pub fn move_to(&mut self, new_position: (usize, usize)) -> Vec<ModelData> {
    let internal_data = self.get_internal_data();

    let anchored_placement = new_position.0 + ((CONFIG.grid_width as usize + 1) * new_position.1);
    let top_left_difference = (internal_data.placement_anchor.0
      + (internal_data.placement_anchor.1 * (CONFIG.grid_width as isize + 1)))
      as usize;

    drop(internal_data);

    let new_index = anchored_placement - top_left_difference;

    self.change_position(new_index);

    self.check_collisions_against_all_models()
  }

  /// Moves a relative amount based on the values passed in.
  ///
  /// Returns a list of references to model collisions that occurred in the new location.
  pub fn move_by(&mut self, added_position: (isize, isize)) -> Vec<ModelData> {
    let true_width = CONFIG.grid_width as isize + 1;

    let new_index = added_position.0 + (true_width * added_position.1) + self.top_left() as isize;

    self.change_position(new_index as usize);

    self.check_collisions_against_all_models()
  }

  /// Changes the placement_anchor and top left position of the model.
  ///
  /// Input is based on the frame_position aka top left position of the model.
  fn change_position(&mut self, new_position: usize) {
    let mut internal_data = self.get_internal_data();

    let new_frame_anchor_index = new_position as isize
      + internal_data.placement_anchor.0
      + (internal_data.placement_anchor.1 * (CONFIG.grid_width as isize + 1));

    let (frame_index, new_anchor) =
      get_position_data(new_frame_anchor_index as usize, &internal_data.sprite);

    internal_data.position_in_frame = frame_index;
    internal_data.placement_anchor = new_anchor;
  }

  /// Returns the character the model uses for air.
  pub fn get_air_char(&self) -> char {
    let internal_data = self.get_internal_data();

    internal_data.sprite.air_character()
  }

  /// Returns a copy of the model's unique hash.
  pub fn get_unique_hash(&self) -> u64 {
    let internal_data = self.get_internal_data();

    internal_data.unique_hash
  }

  /// Returns a copy of the model's current world position.
  pub fn get_model_position(&self) -> usize {
    let internal_data = self.get_internal_data();

    (internal_data.position_in_frame as isize
      + internal_data.placement_anchor.0
      + (internal_data.placement_anchor.1 * (CONFIG.grid_width as isize + 1))) as usize
  }

  /// Returns a copy of the model's current appearance.
  pub fn get_sprite(&self) -> String {
    let internal_data = self.get_internal_data();

    internal_data.sprite.get_shape().to_string()
  }

  /// Replaces the String for the model's appearance
  // This needs changing
  pub fn change_sprite(&mut self, new_model: String) {
    let mut internal_data = self.get_internal_data();

    *internal_data.sprite.get_mut_shape() = new_model;
  }

  /// Returns the model's currently assigned strata.
  pub fn get_strata(&self) -> Strata {
    let internal_data = self.get_internal_data();

    internal_data.strata
  }

  /// Changes the model's Strata with the given one.
  ///
  /// # Errors
  ///
  /// Returns an error when the new given strata is beyond the possible range.
  /// Returns an error when a model's currently assigned strata is also impossible.
  pub fn change_strata(&mut self, new_strata: Strata) -> Result<(), ModelError> {
    let mut internal_data = self.get_internal_data();

    if new_strata.correct_range() {
      internal_data.strata = new_strata;
      drop(internal_data);

      self.fix_model_strata()?;
    } else {
      return Err(ModelError::IncorrectStrataRange(new_strata));
    }

    Ok(())
  }

  /// Changes the name of the model to the one passed in.
  pub fn change_name(&mut self, new_name: String) {
    let mut internal_data = self.get_internal_data();

    internal_data.assigned_name = new_name
  }

  /// Returns a copy of the model's name.
  pub fn get_name(&self) -> String {
    let internal_data = self.get_internal_data();

    internal_data.assigned_name.clone()
  }

  /// Checks the hitbox of every model that exists against the model's own hitbox.
  ///
  /// Returns the list of hitboxes that are colliding with the model's hitbox.
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

  /// Assigns the list of existing models.
  pub fn assign_model_list(&mut self, model_list: Arc<RwLock<InternalModels>>) {
    let mut internal_data = self.get_internal_data();

    internal_data.existing_models = Some(model_list);
  }

  /// Fixes the strata for every model that exists in the InternalModels list.
  ///
  /// If a model somehow has an assigned strata that is different from where it's internall stored.
  /// This method will fix that.
  pub fn fix_model_strata(&self) -> Result<(), ModelError> {
    let internal_data = self.get_internal_data();

    guard!( let Some(model_list) = internal_data.existing_models.as_ref() else { return Err(ModelError::ModelDoesntExist) } );

    let mut model_list_guard = model_list.write().unwrap();

    model_list_guard.fix_strata_list()
  }
}

impl InternalModelData {
  /// # Errors
  ///
  /// - Returns an error when no anchor was found on the shape of the hitbox.
  /// - Returns an error if multiple anchors were found on the shape of the hitbox.
  /// - Returns an error when an impossible strata is passed in.
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

  /// Returns true if the model's hitbox is overlapping with the hitbox of the model passed in.
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
/// Takes the world placement of a model and returns it's index in a frame, and
/// the relative distance of the FrameIndex to the WorldPlacement.
///
/// Returns (FrameIndex, WorldPlacementAnchor)
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

/// Gets the relative distance from the sprite's frame index to it's world placement.
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
