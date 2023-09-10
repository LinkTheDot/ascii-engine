use crate::errors::*;
use crate::models::animation::ModelAnimationData;
use crate::models::hitboxes::*;
use crate::models::model_file_parser::ModelParser;
use crate::models::sprites::*;
use crate::models::stored_models::*;
use crate::models::strata::Strata;
use crate::CONFIG;
use engine_math::{coordinates::*, hasher, rectangle::*};
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};

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
  sprite: Arc<RwLock<Sprite>>,
  hitbox: Hitbox,
  /// This is created when parsing a model.
  ///
  /// None if there was no `.animate` file in the same path of the model, or there was no alternative path given.
  animation_data: Option<Arc<Mutex<ModelAnimationData>>>,
}

impl ModelData {
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
      return Err(ModelError::NonModelFile);
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

  pub fn from_stored(stored_model: StoredDisplayModel) -> Result<Self, ModelError> {
    stored_model.sprite.validity_check()?;

    let internal_model_data = InternalModelData {
      unique_hash: stored_model.unique_hash,
      assigned_name: stored_model.name,
      position_in_frame: stored_model.position,
      strata: stored_model.strata,
      sprite: Arc::new(RwLock::new(stored_model.sprite)),
      hitbox: stored_model.hitbox,
      animation_data: None,
    };

    let model = Self {
      inner: Arc::new(Mutex::new(internal_model_data)),
    };

    if let Some(animation_data) = stored_model.animation_data {
      let animation_data = ModelAnimationData::new(model.clone(), animation_data);

      model.inner.lock().unwrap().animation_data = Some(Arc::new(Mutex::new(animation_data)));
    }

    Ok(model)
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
    let model_sprite_coordiates = self
      .get_sprite()
      .read()
      .unwrap()
      .get_anchor_as_coordinates();

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
  // this method doesn't communicate with the model storage. Due to that
  // the list will need to be checked every time for strata changes anyways,
  // essentially defeating the entire purpose of this system.
  //
  // It won't matter once strata is removed though, so this stays as it is.
  pub fn change_strata(&mut self, new_strata: Strata) -> Result<(), ModelError> {
    if !new_strata.correct_range() {
      return Err(ModelError::IncorrectStrataRange(new_strata));
    }

    let _ = std::mem::replace(&mut self.inner.lock().unwrap().strata, new_strata);

    Ok(())
  }

  /// Returns a reference to the stored [`Sprite`](crate::models::sprites::Sprite) value on this model.
  pub fn get_sprite(&self) -> Arc<RwLock<Sprite>> {
    self.inner.lock().unwrap().sprite.clone()
  }

  pub fn get_hitbox_dimensions(&self) -> Rectangle {
    *self.inner.lock().unwrap().hitbox.get_hitbox_dimensions()
  }

  pub fn get_hitbox(&self) -> Hitbox {
    self.inner.lock().unwrap().hitbox.clone()
  }

  /// Replaces the currently stored hitbox with the new one.
  pub fn change_hitbox(&mut self, new_hitbox_data: HitboxCreationData) {
    let new_hitbox = Hitbox::from(new_hitbox_data);

    let _ = std::mem::replace(&mut self.inner.lock().unwrap().hitbox, new_hitbox);
  }

  /// Returns a reference to the stored [`ModelAnimationData`](crate::models::animation::ModelAnimationData).
  ///
  /// None is returned if the model isn't currently animated.
  // TODO: mention how to animate a model through the screen or a model_manager.
  pub fn get_animation_data(&mut self) -> Option<Arc<Mutex<ModelAnimationData>>> {
    self.inner.lock().unwrap().animation_data.clone()
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
    // self .inner .lock() .unwrap() .hitbox .sprite_to_hitbox_anchor_difference()
    // let inner = self.inner.lock().unwrap();
    let sprite = self.get_sprite();
    let sprite = sprite.read().unwrap();
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
    let sprite = &self.inner.lock().unwrap().sprite;
    let sprite = sprite.read().unwrap();

    Self::caluculate_top_left_index(&sprite, from_position)
  }

  fn caluculate_top_left_index(sprite: &Sprite, position: (usize, usize)) -> Option<usize> {
    let screen_size = CONFIG.grid_width as usize + 1;
    let sprite_anchor = sprite.get_anchor_as_coordinates();

    let position_in_coordinates = position.subtract(sprite_anchor);
    let position_in_coordinates = Coordinates::from_isize(position_in_coordinates)?;

    // Add 1 to account for new lines.
    Some(position_in_coordinates.coordinates_to_index(screen_size) + 1)
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
    hitbox_data: HitboxCreationData,
    strata: Strata,
    assigned_name: String,
  ) -> Result<Self, ModelError> {
    let Some(position_in_frame) =
      ModelData::caluculate_top_left_index(&sprite, model_world_position)
    else {
      return Err(ModelError::ModelOutOfBounds);
    };

    let hitbox = Hitbox::from(hitbox_data);

    // Will be removed once replaced with a Z axis.
    if !strata.correct_range() {
      return Err(ModelError::IncorrectStrataRange(strata));
    }

    Ok(Self {
      unique_hash: hasher::get_unique_hash(),
      assigned_name,
      strata,
      sprite: Arc::new(RwLock::new(sprite)),
      position_in_frame,
      hitbox,
      animation_data: None,
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
    fn move_in_negative_direction() {}

    #[test]
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
      let model_sprite_anchor_index = model
        .get_sprite()
        .read()
        .unwrap()
        .get_anchor_as_coordinates();
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
