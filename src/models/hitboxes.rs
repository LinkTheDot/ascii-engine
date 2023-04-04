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
#[derive(Debug)]
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

  /// Returns the top left position of the hitbox in a frame.
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
  /// Takes the relative distance of the top left of the hitbox to the skin when their anchors are aligned.
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
      (hitbox_anchor_index % hitbox_width) as isize,
      (hitbox_anchor_index / hitbox_width) as isize,
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
mod valid_rectangle_check_logic {
  use super::*;

  #[test]
  fn valid_rectangle() {
    let rectangle = "xxx\nxxx\nxxx";

    let expected_dimensions = Ok((3, 3));

    let rectangle_dimensions = valid_rectangle_check(rectangle);

    assert_eq!(rectangle_dimensions, expected_dimensions);
  }

  #[test]
  fn invalid_rectangle() {
    let shape = "xx\nxxx\nx\nxxxxxx";

    let expected_error = Err(ModelError::NonRectangularShape);

    let returned_data = valid_rectangle_check(shape);

    assert_eq!(returned_data, expected_error);
  }
}
