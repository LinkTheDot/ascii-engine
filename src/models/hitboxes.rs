use crate::general_data::coordinates::*;
use crate::models::errors::*;
use crate::CONFIG;
use log::error;
use std::cmp::Ordering;

/// The hitbox will be how objects know the space they take up in the world.
///
/// You will not need to manually create a hitbox, rather, you will add a field called "Hitbox_Dimensions"
/// to your model file.
///
/// # Example
///
/// The "a" character represents the assigned "anchor_character" under the "Skin" Header.
/// ```no_run,bash,ignore
/// * other data above *
/// -=--=-
/// HitboxDimensions
/// xxxxx
/// xxaxx
/// xxxxx
/// ```
///
/// Refer to [`ModelData`](crate::models::model_data::ModelData) for more information on model creation.
///
/// # Manual Creation
///
/// If for some reason you still want to manually create a hitbox through code (which is not recommended and you should make your own model file).
///
/// First you much create [`HitboxCreationData`](HitboxCreationData).
/// From there, you can create a hitbox with that and the relative anchor to the skin using the [`Hitbox::from()`](Hitbox::from) method.
#[derive(Debug, Eq, PartialEq)]
pub struct Hitbox {
  relative_position_to_skin: (isize, isize),
  width: isize,
  height: isize,
  empty_hitbox: bool,
}

/// The required data to create a hitbox.
///
/// Takes the shape of the hitbox and the anchor.
///
/// The shape must be a rectangular shape, nothing else will be accepted.
///
/// # Example
/// ```no_run,bash,ignore
/// xxxxx
/// xxaxx
/// xxxxx
/// ```
///
/// The anchor will be the relative placement of the hitbox to the appearance of a model.
/// When creating a model, both the appearance and hitbox are required to have anchors.
///
/// When placed in the world, a hitbox will be placed on it's anchor, and the hitbox's anchor
/// will be placed over that.
#[derive(Debug)]
pub struct HitboxCreationData {
  pub shape: String,
  pub anchor_character: char,
}

impl Hitbox {
  /// Creates a new hitbox from the passed in data and anchor to the skin.
  ///
  /// # Errors
  ///
  /// - Returns an error when no anchor was found on the shape of the hitbox.
  /// - Returns an error if multiple anchors were found on the shape of the hitbox.
  pub fn from(
    hitbox_data: HitboxCreationData,
    skin_anchor: (isize, isize),
  ) -> Result<Self, ModelError> {
    hitbox_data.get_hitbox_data(skin_anchor)
  }

  /// Returns an empty hitbox.
  ///
  /// An empty hitbox will have the 'empty_hitbox' field labeled as true.
  /// This will stop any checks from being run on this hitbox instance.
  ///
  /// This means an object with an "empty hitbox" will never interact with the world.
  fn create_empty() -> Self {
    Self {
      relative_position_to_skin: (0, 0),
      width: 0,
      height: 0,
      empty_hitbox: true,
    }
  }

  /// Takes the frame position of the model and returns the hitbox's frame position.
  ///
  /// Returned as (x, y)
  pub fn get_hitbox_position(&self, model_position: usize) -> (isize, isize) {
    let (model_x, model_y) = model_position.index_to_coordinates(CONFIG.grid_width as usize + 1);

    (
      model_x as isize + self.relative_position_to_skin.0,
      model_y as isize + self.relative_position_to_skin.1,
    )
  }

  /// Returns the (width, height) of the hitbox.
  pub fn get_dimensions(&self) -> (isize, isize) {
    (self.width, self.height)
  }

  /// Returns true if the hitbox is labeled as empty.
  pub fn is_empty(&self) -> bool {
    self.empty_hitbox
  }

  /// Returns the (x, y) of the hitbox based on if the model was in the position of the new passed in value.
  ///
  /// The passed in value is based on frame index.
  pub fn get_position_based_on(&self, new_position: usize) -> (isize, isize) {
    let (model_x, model_y) = new_position.index_to_coordinates(CONFIG.grid_width as usize + 1);

    (
      model_x as isize + self.relative_position_to_skin.0,
      model_y as isize + self.relative_position_to_skin.1,
    )
  }
}

impl HitboxCreationData {
  /// Creates a new instance of HitboxCreationData.
  ///
  /// This should not be used over model files.
  /// Refer to [`ModelData`](crate::models::model_data::ModelData) for information of creating a model file.
  pub fn new(shape: &str, anchor_character: char) -> Self {
    Self {
      shape: shape.to_string(),
      anchor_character,
    }
  }

  /// Converts a [`HitboxCreationData`](HitboxCreationData) into a [`Hitbox`](Hitbox).
  ///
  /// NOTE
  /// This method takes the distance of the model's anchor TO it's top left.
  /// What this means is if you have some model:
  /// ```no_run,bash,ignore
  /// xxx
  /// xax
  /// xxx
  /// ```
  /// Here you would pass in (-1, -1). This is because the top left if (-1, -1) away from the anchor.
  ///
  /// If the skin string is empty, returns an [`empty hitbox`](Hitbox::create_empty).
  ///
  /// # Errors
  ///
  /// - Returns an error when no anchor was found on the shape of the hitbox.
  /// - Returns an error if multiple anchors were found on the shape of the hitbox.
  fn get_hitbox_data(self, skin_relative_anchor: (isize, isize)) -> Result<Hitbox, ModelError> {
    if self.shape.trim() == "" {
      return Ok(Hitbox::create_empty());
    }

    let (hitbox_width, hitbox_height) = valid_rectangle_check(&self.shape)?;
    let hitbox = &self.shape.split('\n').collect::<String>();
    let hitbox_anchor_indices: Vec<usize> = hitbox
      .chars()
      .enumerate()
      .filter(|(_, character)| character == &self.anchor_character)
      .map(|(index, _)| index)
      .collect();

    let hitbox_anchor_index = match hitbox_anchor_indices.len().cmp(&1) {
      Ordering::Equal => hitbox_anchor_indices[0],
      Ordering::Greater => {
        error!("Multiple anchors were found when attempting to make a hitbox.");

        return Err(ModelError::MultipleAnchorsFound(hitbox_anchor_indices));
      }
      Ordering::Less => {
        error!("No anchors were found when attempting to make a hitbox.");

        return Err(ModelError::NoAnchor);
      }
    };

    let hitbox_anchor_coordinates = (
      (hitbox_anchor_index as f32 % hitbox_width as f32).ceil() as isize,
      (hitbox_anchor_index as f32 / hitbox_width as f32).ceil() as isize,
    );

    let x_difference = skin_relative_anchor.0 - hitbox_anchor_coordinates.0;
    let y_difference = skin_relative_anchor.1 - hitbox_anchor_coordinates.1;

    Ok(Hitbox {
      relative_position_to_skin: (x_difference, y_difference),
      width: hitbox_width as isize,
      height: hitbox_height as isize,
      empty_hitbox: false,
    })
  }
}

/// Returns
/// Result<(width, height)>.
///
/// # Errors
///
/// - An error is returned when the hitbox isn't a rectangle.
// It shouldn't be possible to pass in nothing where this is called right now.
// If that changes then change this to account for that.
fn valid_rectangle_check(model: &str) -> Result<(usize, usize), ModelError> {
  let rows: Vec<&str> = model.split('\n').collect();
  let model_width = rows[0].chars().count();

  let rows_have_same_lengths = rows.iter().all(|row| row.chars().count() == model_width);

  if rows_have_same_lengths {
    Ok((model_width, rows.len()))
  } else {
    Err(ModelError::NonRectangularShape)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  // use crate::prelude::*;

  #[cfg(test)]
  mod valid_rectangle_check_logic {
    use super::*;

    #[test]
    fn valid_rectangle() {
      let dimensions = "xxx\nxxx\nxxx";

      let expected_dimensions = Ok((3, 3));

      let rectangle_dimensions = valid_rectangle_check(dimensions);

      assert_eq!(rectangle_dimensions, expected_dimensions);
    }

    #[test]
    fn invalid_rectangle() {
      let dimensions = "xx\nxxx\nx\nxxxxxx";

      let expected_error = Err(ModelError::NonRectangularShape);

      let returned_data = valid_rectangle_check(dimensions);

      assert_eq!(returned_data, expected_error);
    }
  }

  #[test]
  fn empty_hitbox_logic() {
    let hitbox_creation_data = HitboxCreationData::new("", 'a');
    let hitbox = Hitbox::from(hitbox_creation_data, (0, 0)).unwrap();

    assert!(hitbox.is_empty());
  }

  #[test]
  fn get_hitbox_position_logic() {
    let hitbox_creation_data = HitboxCreationData::new("xxxxx\nxxaxx\nxxxxx", 'a');
    let hitbox = Hitbox::from(hitbox_creation_data, (1, 1)).unwrap();
    let model_frame_position = 10 + (10 * (CONFIG.grid_width as usize + 1));

    let expected_position = (9, 9);

    let position = hitbox.get_hitbox_position(model_frame_position);

    assert_eq!(position, expected_position);
  }

  #[test]
  fn get_dimensions_logic() {
    let hitbox_creation_data = HitboxCreationData::new("xxxxx\nxxaxx\nxxxxx", 'a');
    let hitbox = Hitbox::from(hitbox_creation_data, (1, 1)).unwrap();

    let expected_dimensions: (isize, isize) = (5, 3);

    let hitbox_dimensions = hitbox.get_dimensions();

    assert_eq!(hitbox_dimensions, expected_dimensions);
  }

  #[test]
  fn hitbox_data_no_anchor() {
    let hitbox_creation_data = HitboxCreationData::new("x", 'a');

    let expected_result = Err(ModelError::NoAnchor);

    let result = Hitbox::from(hitbox_creation_data, (0, 0));

    assert_eq!(result, expected_result);
  }

  #[test]
  fn hitbox_data_multiple_anchors() {
    let hitbox_creation_data = HitboxCreationData::new("xaax", 'a');

    let expected_result = Err(ModelError::MultipleAnchorsFound(vec![1, 2]));

    let result = Hitbox::from(hitbox_creation_data, (0, 0));

    assert_eq!(result, expected_result);
  }

  // uncomment if needed
  //
  // -- Data for tests below --
  //
  //
  // #[derive(DisplayModel)]
  // struct TestModel {
  //   model_data: ModelData,
  // }
  //
  // impl TestModel {
  //   fn new() -> Self {
  //     let test_model_path = std::path::Path::new("tests/models/test_square.model");
  //     let model_data = ModelData::from_file(test_model_path, WORLD_POSITION).unwrap();
  //
  //     Self { model_data }
  //   }
  // }
}
