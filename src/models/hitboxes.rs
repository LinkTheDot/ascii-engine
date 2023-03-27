use crate::general_data::coordinates::*;
use crate::models::errors::*;
use crate::CONFIG;
use log::error;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct Hitbox {
  relative_position_to_skin: (isize, isize),
  width: isize,
  height: isize,
  empty_hitbox: bool,
}

#[derive(Debug)]
pub struct HitboxCreationData {
  pub shape: String,
  pub anchor_character: char,
}

impl Hitbox {
  pub fn from(
    hitbox_data: HitboxCreationData,
    skin_anchor: (isize, isize),
  ) -> Result<Self, ModelError> {
    hitbox_data.get_hitbox_data(skin_anchor)
  }

  fn create_empty() -> Self {
    Self {
      relative_position_to_skin: (0, 0),
      width: 0,
      height: 0,
      empty_hitbox: true,
    }
  }

  /// Gives the coordinates of the top left of the hitbox.
  /// Returns (x, y).
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

  pub fn is_empty(&self) -> bool {
    self.empty_hitbox
  }
}

impl HitboxCreationData {
  pub fn new(shape: &str, anchor_character: char) -> Self {
    Self {
      shape: shape.to_string(),
      anchor_character,
    }
  }

  /// Converts the given data into a list of relative points from the anchor.
  ///
  /// Returns an error when an invalid hitbox is passed in, or when there's no
  /// valid anchor character in the shape of the hitbox.
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
/// (width, height).
///
/// An error is returned when the hitbox isn't a rectangle.
fn valid_rectangle_check(model: &str) -> Result<(usize, usize), ModelError> {
  if model.chars().count() == 0 {
    return Err(ModelError::EmptyHitboxString);
  }

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

  #[test]
  fn empty_string_passed_in() {
    let empty_string = "";

    let expected_error = Err(ModelError::EmptyHitboxString);

    let returned_data = valid_rectangle_check(empty_string);

    assert_eq!(returned_data, expected_error);
  }
}
