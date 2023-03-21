use crate::general_data::coordinates::*;
use crate::objects::errors::*;
use crate::CONFIG;
use guard::guard;

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
  pub center_character: char,
}

impl Hitbox {
  pub fn from(
    hitbox_data: HitboxCreationData,
    skin_relative_center: (isize, isize),
  ) -> Result<Self, ObjectError> {
    hitbox_data.get_hitbox_data(skin_relative_center)
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
  pub fn get_hitbox_position(&self, object_position: usize) -> (isize, isize) {
    let (object_x, object_y) = object_position.index_to_coordinates(CONFIG.grid_width as usize + 1);

    (
      object_x as isize + self.relative_position_to_skin.0,
      object_y as isize + self.relative_position_to_skin.1,
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
  // OBJECT CREATION WILL CHANGE TO A FILE FORMAT
  pub fn new(shape: &str, center_character: char) -> Self {
    Self {
      shape: shape.to_string(),
      center_character,
    }
  }

  /// Converts the given data into a list of relative points from the center.
  ///
  /// Returns an error when an invalid hitbox is passed in, or when there's no
  /// valid center character in the shape of the hitbox.
  fn get_hitbox_data(self, skin_relative_center: (isize, isize)) -> Result<Hitbox, ObjectError> {
    if self.shape.trim() == "" {
      return Ok(Hitbox::create_empty());
    }

    let (hitbox_width, hitbox_height) = valid_rectangle_check(&self.shape)?;
    let hitbox = &self.shape.split('\n').collect::<String>();
    let hitbox_center_index = hitbox
      .chars()
      .position(|pixel| pixel == self.center_character);

    guard!( let Some(hitbox_center_index) = hitbox_center_index else { return Err(ObjectError::NoCenter) });

    let hitbox_center_coordinates = (
      (hitbox_center_index % hitbox_width) as isize,
      (hitbox_center_index / hitbox_width) as isize,
    );

    let x_difference = skin_relative_center.0 - hitbox_center_coordinates.0;
    let y_difference = skin_relative_center.1 - hitbox_center_coordinates.1;

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
fn valid_rectangle_check(object: &str) -> Result<(usize, usize), ObjectError> {
  if object.chars().count() == 0 {
    return Err(ObjectError::EmptyHitboxString);
  }

  let rows: Vec<&str> = object.split('\n').collect();
  let object_width = rows[0].chars().count();

  let rows_have_same_lengths = rows.iter().all(|row| row.chars().count() == object_width);

  if rows_have_same_lengths {
    Ok((object_width, rows.len()))
  } else {
    Err(ObjectError::NonRectangularShape)
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

    let expected_error = Err(ObjectError::NonRectangularShape);

    let returned_data = valid_rectangle_check(shape);

    assert_eq!(returned_data, expected_error);
  }

  #[test]
  fn empty_string_passed_in() {
    let empty_string = "";

    let expected_error = Err(ObjectError::EmptyHitboxString);

    let returned_data = valid_rectangle_check(empty_string);

    assert_eq!(returned_data, expected_error);
  }
}
