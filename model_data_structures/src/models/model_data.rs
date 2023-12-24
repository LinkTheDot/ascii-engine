use crate::errors::*;
use crate::models::hitboxes::*;
use crate::models::model_appearance::sprites::*;
use crate::models::model_file_parser::ModelParser;
use crate::models::stored_models::*;
use crate::models::strata::Strata;
use crate::prelude::ModelAppearance;
use crate::CONFIG;
use engine_math::{coordinates::*, hasher, rectangle::*};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct ModelData {
  inner: Arc<Mutex<InternalModelData>>,
}

#[derive(Debug, Clone)]
pub struct InternalModelData {
  unique_hash: u64,
  assigned_name: String,
  /// counts new lines
  // Will be replaced with coordinates once cameras are implemented
  position_in_frame: usize,
  strata: Strata,
  appearance: Arc<Mutex<ModelAppearance>>,
  hitbox: Hitbox,
  tags: HashSet<String>,
}

impl ModelData {
  /// This will create the data for a model.
  /// The data will contain things such as what the model looks like, the hitbox,
  /// what strata or layer the model sits on, the position, and more.
  ///
  /// For information on creating a model, refer to [`ModelData`](ModelData).
  // TODO: List the errors.
  pub fn new(
    model_position: Coordinates,
    base_appearance: Sprite,
    hitbox_data: Hitbox,
    strata: Strata,
    assigned_name: String,
  ) -> Result<Self, ModelError> {
    let internal_data = InternalModelData::new(
      model_position,
      base_appearance,
      hitbox_data,
      strata,
      assigned_name,
    )?;

    Ok(Self {
      inner: Arc::new(Mutex::new(internal_data)),
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
  /// - Returns an error when the file has the wrong extension.
  /// - Returns an error when the file didn't exist.
  /// - Returns an error when the model file was build incorrectly. [`Errors when parsing model files`](crate::models::errors::ModelCreationError).
  pub fn from_file(
    model_file_path: &Path,
    frame_position: (usize, usize),
  ) -> Result<Self, ModelError> {
    if model_file_path.extension() != Some(OsStr::new("model")) {
      return Err(ModelCreationError::NonModelFile.into());
    }
    let model_file = File::open(model_file_path);

    match model_file {
      Ok(file) => ModelParser::parse(file, frame_position),
      Err(_) => {
        let file_path = model_file_path
          .file_name()
          // Unwrap and convert the OsStr to an OsString.
          .map(|path_string| path_string.to_owned());

        let error = ModelCreationError::ModelFileDoesntExist(file_path);

        Err(ModelError::ModelCreationError(error))
      }
    }
  }

  // TODO: List the errors.
  pub fn from_stored(mut stored_model: StoredDisplayModel) -> Result<Self, ModelError> {
    if !stored_model.repair_missing_fields() {
      return Err(ModelError::MissingCrutialFieldsInStoredDisplayModel);
    }

    if let Err(ModelError::AnimationError(AnimationError::AnimationValidityCheckFailed(
      error_list,
    ))) = stored_model
      .appearance_data
      .as_mut()
      .unwrap()
      .full_validity_check()
    {
      for animation_error_data in error_list {
        log::error!(
          "Failed to load animation for model {:?}. Reasons: {:?}",
          stored_model.name,
          animation_error_data
        );

        let _ = stored_model
          .appearance_data
          .as_mut()
          .unwrap()
          .remove_animation_from_list(&animation_error_data.animation_name);
      }
    }

    let internal_model_data = InternalModelData {
      unique_hash: engine_math::hasher::get_unique_hash(),
      assigned_name: stored_model.name.unwrap_or("".to_string()),
      position_in_frame: stored_model.position.unwrap_or(0),
      strata: stored_model.strata.unwrap_or(Strata(0)),
      appearance: Arc::new(Mutex::new(stored_model.appearance_data.unwrap())),
      hitbox: stored_model.hitbox.unwrap(),
      tags: stored_model.tags.unwrap(),
    };

    Ok(Self {
      inner: Arc::new(Mutex::new(internal_model_data)),
    })
  }

  pub fn to_stored(self) -> StoredDisplayModel {
    StoredDisplayModel::new(self)
  }

  /// Returns a copy of the model's stored unique hash.
  pub fn get_hash(&self) -> u64 {
    self.inner.lock().unwrap().unique_hash
  }

  /// Returns a copy of the model's assigned name.
  pub fn get_name(&self) -> String {
    self.inner.lock().unwrap().assigned_name.clone()
  }

  /// Changes the assigned name for the model
  pub fn change_name(&self, new_name: String) {
    self.inner.lock().unwrap().assigned_name = new_name
  }

  /// Returns a copy of the current top left position of the model in the world.
  pub fn get_frame_position(&self) -> usize {
    self.inner.lock().unwrap().position_in_frame
  }

  /// Returns the world position of the model, this is where the model's sprite anchor is located.
  pub fn get_world_position(&self) -> (isize, isize) {
    let frame_position = self.get_frame_position();
    let screen_width = CONFIG.grid_width as usize + 1;
    let model_sprite_coordiates = self.get_sprite().get_anchor_as_coordinates();

    frame_position
      .index_to_coordinates(screen_width)
      .add(model_sprite_coordiates)
      .subtract((1, 0)) // Remove 1 from the x-axis to stop accounting for new lines.
  }

  /// Returns a copy of the currently stored strata for the model.
  pub fn get_strata(&self) -> Strata {
    self.inner.lock().unwrap().strata
  }

  /// Replaces the strata with the new one passed in.
  ///
  /// # Errors
  ///
  /// - When the new strata passed in was in an impossible range
  // This method is fun.
  // Because models are stored by strata for easier frame building, and
  // models don't communicate with the model storage. Due to that the list
  // will need to be checked every time for strata changes anyways,
  // essentially defeating the entire purpose of this system when changed.
  //
  // It won't matter once strata is removed though, so this stays as it is.
  pub fn change_strata(&mut self, new_strata: Strata) -> Result<(), ModelError> {
    if !new_strata.correct_range() {
      return Err(ModelError::IncorrectStrataRange(new_strata));
    }

    let _ = std::mem::replace(&mut self.inner.lock().unwrap().strata, new_strata);

    Ok(())
  }

  /// Returns a copy of the current [`Sprite`](crate::models::model_appearance::sprites::Sprite) on this model.
  ///
  /// Preferably you get a copy of the appearance through [`get_appearance_data`](ModelData::get_appearance_data), and obtain
  /// a reference to the Sprite instead.
  pub fn get_sprite(&self) -> Sprite {
    self
      .get_appearance_immutably()
      .lock()
      .unwrap()
      .get_appearance()
      .clone()
  }

  /// Returns the current dimensions of the hitbox.
  pub fn get_hitbox_dimensions(&self) -> Rectangle {
    *self.inner.lock().unwrap().hitbox.get_hitbox_dimensions()
  }

  /// Returns a copy of the current hitbox.
  pub fn get_hitbox(&self) -> Hitbox {
    self.inner.lock().unwrap().hitbox.clone()
  }

  /// Replaces the currently stored hitbox with a new one, returing the previously stored hitbox.
  pub fn change_hitbox(&mut self, new_hitbox: Hitbox) -> Hitbox {
    std::mem::replace(&mut self.inner.lock().unwrap().hitbox, new_hitbox)
  }

  /// Returns a reference to the [`model's appearance]`(crate::model_data::model_appearance::ModelAppearance).
  // TODO: mention how to animate a model through the screen or a model_manager.
  pub fn get_appearance_data(&mut self) -> Arc<Mutex<ModelAppearance>> {
    self.inner.lock().unwrap().appearance.clone()
  }

  /// Changes the placement_anchor and top left position of the model.
  ///
  /// Input is based on the frame_position aka top left position of the model.
  pub fn change_position(&mut self, new_position: usize) {
    let mut internal_data = self.inner.lock().unwrap();

    internal_data.position_in_frame = new_position;
  }

  /// Returns true if the area of the model's hitbox is 0;
  pub fn hitbox_is_empty(&self) -> bool {
    let internal_data = self.inner.lock().unwrap();

    internal_data.hitbox.get_hitbox_dimensions().area() == 0
  }

  pub fn sprite_to_hitbox_anchor_difference(&self) -> (isize, isize) {
    let sprite = self.get_sprite();
    let sprite_anchor = sprite.get_anchor_as_coordinates();
    drop(sprite);
    let hitbox_anchor = self
      .inner
      .lock()
      .unwrap()
      .hitbox
      .get_anchor_as_coordinates();

    hitbox_anchor.subtract(sprite_anchor)
  }

  /// Returns the top left of the model in the frame based on the given position.
  ///
  /// This does not use the current position for the model. Rather, it takes a hypothetical
  /// world position for the model, returning where the model's top left position would be
  /// if it were in this position.
  ///
  /// None is returned if the position was OutOfBounds in the negative direction.
  pub fn calculate_top_left_index_from(&self, from_position: (usize, usize)) -> Option<usize> {
    let appearance = self.get_appearance_immutably();
    let appearance = appearance.lock().unwrap();
    let sprite = appearance.get_appearance();

    Self::caluculate_top_left_index(sprite, from_position)
  }

  fn caluculate_top_left_index(sprite: &Sprite, position: (usize, usize)) -> Option<usize> {
    let screen_size = CONFIG.grid_width as usize + 1;
    let sprite_anchor = sprite.get_anchor_as_coordinates();

    let position_in_coordinates = position.subtract(sprite_anchor);
    let position_in_coordinates = Coordinates::from_isize(position_in_coordinates)?;

    // Add 1 to account for new lines.
    Some(position_in_coordinates.coordinates_to_index(screen_size) + 1)
  }

  fn get_appearance_immutably(&self) -> Arc<Mutex<ModelAppearance>> {
    self.inner.lock().unwrap().appearance.clone()
  }

  /// Returns true of the model contains the given tag.
  pub fn contains_tag<S: AsRef<str>>(&self, tag: S) -> bool {
    let inner = self.inner.lock().unwrap();

    inner.tags.contains(tag.as_ref())
  }

  /// Returns true of the model contains the given tag.
  pub fn contains_tags<S: AsRef<str>>(&self, tags: &[S]) -> bool {
    let inner = self.inner.lock().unwrap();

    tags
      .iter()
      .map(AsRef::as_ref)
      .all(|tag| inner.tags.contains(tag))
  }

  pub fn add_tags(&mut self, tags: Vec<String>) {
    tags.into_iter().for_each(|tag| {
      self.inner.lock().unwrap().tags.insert(tag);
    });
  }

  /// Returns a copy of the tags for this model.
  pub fn get_tags(&self) -> HashSet<String> {
    self.inner.lock().unwrap().tags.clone()
  }
}

impl InternalModelData {
  /// # Errors
  ///
  /// - Returns an error when no anchor was found on the shape of the hitbox.
  /// - Returns an error if multiple anchors were found on the shape of the hitbox.
  /// - Returns an error when an impossible strata is passed in.
  /// - Returns an error if the model was placed out of bounds.
  fn new(
    model_world_position: Coordinates,
    sprite: Sprite,
    hitbox: Hitbox,
    strata: Strata,
    assigned_name: String,
  ) -> Result<Self, ModelError> {
    let Some(position_in_frame) =
      ModelData::caluculate_top_left_index(&sprite, model_world_position)
    else {
      return Err(ModelError::ModelOutOfBounds);
    };

    // Will be removed once replaced with a Z axis.
    if !strata.correct_range() {
      return Err(ModelError::IncorrectStrataRange(strata));
    }

    let model_appearance = Arc::new(Mutex::new(ModelAppearance::new(sprite, None)));

    Ok(Self {
      unique_hash: hasher::get_unique_hash(),
      assigned_name,
      strata,
      appearance: model_appearance,
      position_in_frame,
      hitbox,
      tags: HashSet::new(),
    })
  }
}

impl PartialEq for ModelData {
  fn eq(&self, other: &Self) -> bool {
    self.get_hash() == other.get_hash()
  }
}

impl PartialEq for InternalModelData {
  fn eq(&self, other: &Self) -> bool {
    self.unique_hash == other.unique_hash
  }
}

impl Eq for ModelData {}

impl PartialOrd for ModelData {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for ModelData {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.get_hash().cmp(&other.get_hash())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::testing_data::*;

  const WORLD_POSITION: (usize, usize) = (10, 10);

  #[cfg(test)]
  mod change_position_logic {
    use super::*;

    #[test]
    fn move_in_positive_direction() {
      let mut model = TestingData::new_test_model(WORLD_POSITION);
      let new_position = Coordinates::from_isize(WORLD_POSITION.subtract((1, 1)))
        .unwrap()
        .coordinates_to_index(CONFIG.grid_width as usize + 1);

      model.change_position(new_position);

      assert_eq!(model.get_frame_position(), new_position);
    }

    #[test]
    #[ignore]
    fn move_in_negative_direction() {}

    #[test]
    #[ignore]
    fn move_out_of_bounds() {}
  }

  #[test]
  fn model_position_is_correct() {
    let model = TestingData::new_test_model(WORLD_POSITION);
    let expected_position = model.calculate_top_left_index_from(WORLD_POSITION).unwrap();

    assert_eq!(model.get_frame_position(), expected_position);
    assert_eq!(model.get_world_position(), WORLD_POSITION.to_isize());
  }

  #[cfg(test)]
  mod calculate_top_left_index_from_logic {
    use super::*;

    #[test]
    fn valid_position() {
      let model = TestingData::new_test_model(WORLD_POSITION);
      let position = (5, 5);

      let screen_size = CONFIG.grid_width as usize + 1;
      let model_sprite_anchor_index = model.get_sprite().get_anchor_as_coordinates();
      // Add 1 to account for new lines
      let expected_index = 1
        + (Coordinates::from_isize(position.subtract(model_sprite_anchor_index))
          .unwrap()
          .coordinates_to_index(screen_size));

      let index = model.calculate_top_left_index_from(position).unwrap();

      assert_eq!(index, expected_index);
    }

    #[test]
    #[should_panic]
    fn position_out_of_bounds() {
      let model = TestingData::new_test_model(WORLD_POSITION);
      let position = (0, 0);

      model.calculate_top_left_index_from(position).unwrap();
    }
  }
}
