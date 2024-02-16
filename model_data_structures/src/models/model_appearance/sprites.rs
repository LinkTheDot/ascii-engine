use crate::errors::*;
use engine_math::{prelude::usizeMethods, rectangle::Rectangle};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// The ``Sprite`` contains the data for how a model looks on the screen.
///
/// Also holds the anchor for how the appearance in placed on the screen and where the hitbox is placed
/// relative to the appearance.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Sprite {
  shape: String,
  anchor_character: char,
  anchor_replacement_character: char,
  air_character: char,

  /// Doesn't count new lines
  anchor_character_index: usize,
}

impl Default for Sprite {
  fn default() -> Self {
    Self {
      shape: "".to_string(),
      anchor_character: 'a',
      anchor_replacement_character: ' ',
      air_character: '-',
      anchor_character_index: 0,
    }
  }
}

/// The anchor_character_index needs recalculating after every change of the shape and anchor_character.
impl Sprite {
  /// Returns a default of self.
  // TODO: List the errors. (There's a lot).
  pub fn new(
    shape: impl AsRef<str>,
    anchor_character: char,
    anchor_replacement_character: char,
    air_character: char,
  ) -> Result<Self, ModelError> {
    let shape = shape.as_ref().to_string();
    let mut sprite = Sprite::default();

    sprite.change_shape(
      shape,
      Some(anchor_character),
      Some(anchor_replacement_character),
    )?;
    sprite.change_air_character(air_character)?;

    Ok(sprite)
  }

  /// Changes the internally stored shape and recalculates the anchor's index.
  ///
  /// # Errors
  ///
  /// - The stored shape doesn't have an anchor.
  /// - The stored shape has multiple anchors.
  pub fn change_shape(
    &mut self,
    new_shape: String,
    new_anchor_character: Option<char>,
    new_anchor_replacement_character: Option<char>,
  ) -> Result<(), ModelError> {
    if !Rectangle::string_is_valid_rectangle(&new_shape) {
      log::debug!(
        "shape: {:?}\nanchor_character: {:?}\nanchor_replacement: {:?}",
        new_shape,
        new_anchor_character,
        new_anchor_replacement_character
      );
      return Err(ModelError::NonRectangularShape);
    }

    let anchor_character = new_anchor_character.unwrap_or(self.anchor_character);
    let anchor_replacement_character =
      new_anchor_replacement_character.unwrap_or(self.anchor_replacement_character);
    let new_anchor_character = new_anchor_character.unwrap_or(self.anchor_character);

    if anchor_character == self.air_character {
      return Err(ModelError::SpriteAnchorMatchesAirCharacter);
    }

    let new_index =
      Self::calculate_anchor_index(&new_shape.replace('\n', ""), new_anchor_character)?;

    self.shape = new_shape;
    self.anchor_character_index = new_index;
    self.anchor_character = anchor_character;
    self.anchor_replacement_character = anchor_replacement_character;

    Ok(())
  }

  /// Changes the stored anchor character to the new one, and replaces the previous one in the string
  /// with the new one.
  ///
  /// # Errors
  ///
  /// - If the current shape already contains the new_anchor_character.
  pub fn change_anchor_character(&mut self, new_anchor_character: char) -> Result<(), ModelError> {
    if self.shape.contains(new_anchor_character) {
      return Err(ModelError::ModelSpriteContainsNewAnchorCharacter);
    }

    self.shape = self
      .shape
      .replace(self.anchor_character, &new_anchor_character.to_string());
    self.anchor_character = new_anchor_character;

    Ok(())
  }

  /// Replaces the currently stored air character.
  ///
  /// The air character is basically a transparency layer on the shape of a sprite.
  /// Whenever AsciiEngine is printing to the screen, this character will be ignored.
  ///
  /// # Errors
  ///
  /// - The new_air_character is the same as the currently stored anchor character.
  pub fn change_air_character(&mut self, new_air_character: char) -> Result<(), ModelError> {
    if self.anchor_character == new_air_character {
      return Err(ModelError::SpriteAnchorMatchesAirCharacter);
    }

    self.air_character = new_air_character;

    Ok(())
  }

  /// Replaces the currently stored anchor_replacement character.
  ///
  /// This is the character that the anchor will be replaced with.
  /// Obviously you don't want some random irrelevant character to be slapped on your sprite.
  /// This is to mask over that pesky anchor character when AsciiEngine is printing the sprite.
  pub fn change_anchor_replacement_character(&mut self, new_anchor_replacement: char) {
    self.anchor_replacement_character = new_anchor_replacement;
  }

  /// Returns the index of the anchor character in the sprite's current appearance.
  pub fn get_anchor_index(&self) -> usize {
    self.anchor_character_index
  }

  /// Returns the anchor as if it were coordinates.
  ///
  /// This is not based on World coordinates, but rather it's coordinates internal to the sprite's appearance.
  /// That means if the model is 3x3 in size, and the index is 4, this method will return (1, 1).
  pub fn get_anchor_as_coordinates(&self) -> (usize, usize) {
    let sprite_width = self.get_dimensions().x;

    self
      .anchor_character_index
      .index_to_coordinates(sprite_width)
  }

  /// Returns the actual appearance of the sprite.
  ///
  /// This will be the appearance of the model without the replacement anchor character.
  pub fn get_appearance(&self) -> String {
    self.shape.clone().replace(
      self.anchor_character,
      self.anchor_replacement_character.to_string().as_str(),
    )
  }

  /// Calculates the index for the anchor_character in the shape and returns it.
  /// Does NOT account for new lines in the shape string.
  ///
  /// # Errors
  ///
  /// - The stored shape doesn't have an anchor.
  /// - The stored shape has multiple anchors.
  pub fn calculate_anchor_index(shape: &str, anchor_character: char) -> Result<usize, ModelError> {
    let shape_anchor_indices: Vec<usize> = shape
      .replace('\n', "")
      .chars()
      .enumerate()
      .filter(|(_, character)| character == &anchor_character)
      .map(|(index, _)| index)
      .collect();

    match shape_anchor_indices.len().cmp(&1) {
      Ordering::Equal => Ok(shape_anchor_indices[0]),
      Ordering::Greater => {
        log::error!("Multiple anchors were found when calculating a sprite's anchor index.");

        Err(ModelError::MultipleAnchorsFound(shape_anchor_indices))
      }
      Ordering::Less => {
        log::error!("No anchors were found when calculating a sprite's anchor index.");

        Err(ModelError::NoAnchor)
      }
    }
  }

  /// Checks if the sprite is valid or not.
  /// Returns a wrapper of errors that may have been detected with the sprite's data.
  ///
  /// # Errors
  ///
  /// - The stored shape isn't rectangular.
  /// - The stored shape doesn't have an anchor.
  /// - The stored shape has multiple anchors.
  /// - The anchor and air characters are the same.
  pub fn validity_check(&self) -> Result<(), ModelError> {
    let mut error_list = vec![];

    if !Rectangle::string_is_valid_rectangle(&self.shape) {
      error_list.push(ModelError::NonRectangularShape);
    }

    if let Err(anchor_error) = Self::calculate_anchor_index(&self.shape, self.anchor_character) {
      error_list.push(anchor_error);
    }

    if self.anchor_character == self.air_character {
      error_list.push(ModelError::SpriteAnchorMatchesAirCharacter);
    }

    if !error_list.is_empty() {
      Err(ModelError::SpriteValidityChecks(error_list))
    } else {
      Ok(())
    }
  }

  /// Returns the dimensions for the string of the sprite's shape.
  ///
  /// Does NOT include new lines.
  pub fn get_dimensions(&self) -> Rectangle {
    // The shape should always be valid
    Rectangle::get_string_dimensions(&self.shape).unwrap()
  }

  /// Returns a copy of the current air character.
  pub fn air_character(&self) -> char {
    self.air_character
  }

  /// Creates a new Sprite with the data as is. Does not check for any errors that may make the Sprite invalid.
  /// When passing in the index, it is exclusive to any newlines, meaning an appearance of "xxx\nxcx" would have
  /// an anchor index of 4, because the newline is ignored.
  ///
  /// This is for creating Sprites for any validity tests that require an invalid Sprite.
  #[cfg(test)]
  pub fn new_unchecked(
    shape: impl AsRef<str>,
    anchor_character: char,
    anchor_replacement_character: char,
    air_character: char,
    anchor_character_index: usize,
  ) -> Self {
    Self {
      shape: shape.as_ref().to_string(),
      anchor_character,
      anchor_replacement_character,
      air_character,
      anchor_character_index,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn change_shape_invalid_rectangle() {
    let mut sprite = Sprite::default();
    let invalid_shape = "a\n-sd".to_string();

    let expected_error = ModelError::NonRectangularShape;

    let result = sprite.change_shape(invalid_shape, None, None).unwrap_err();

    assert_eq!(result, expected_error);
  }

  #[test]
  fn change_shape_anchor_equals_air_character() {
    let mut sprite = Sprite::default();

    let expected_error = ModelError::SpriteAnchorMatchesAirCharacter;

    let result = sprite
      .change_shape("-a-".to_string(), Some('-'), None)
      .unwrap_err();

    assert_eq!(result, expected_error);
  }

  #[test]
  fn change_anchor_character_sprite_contains_new_anchor() {
    let mut sprite = Sprite::new("-x-\n-a-", 'a', '-', '-').unwrap();

    let expected_error = ModelError::ModelSpriteContainsNewAnchorCharacter;

    let result = sprite.change_anchor_character('x').unwrap_err();

    assert_eq!(result, expected_error);
  }

  #[test]
  fn change_anchor_character_logic() {
    let appearance = "|-|\n|a|".to_string();
    let mut sprite = Sprite::new(appearance.clone(), 'a', '-', '-').unwrap();

    let expected_appearance = appearance.replace('a', "x");

    sprite.change_anchor_character('x').unwrap();

    assert_eq!(sprite.shape, expected_appearance);
  }

  #[test]
  fn change_air_character_matching_anchor() {
    let mut sprite = Sprite::new("-x-\n-a-", 'a', '-', '-').unwrap();

    let expected_error = ModelError::SpriteAnchorMatchesAirCharacter;

    let result = sprite.change_air_character('a').unwrap_err();

    assert_eq!(result, expected_error);
  }

  #[test]
  fn change_anchor_replacement_character_logic() {
    let appearance = "-x-\n-a-".to_string();
    let mut sprite = Sprite::new(appearance.clone(), 'a', '-', '-').unwrap();

    let expected_before = appearance.replace('a', "-");
    let expected_after = appearance.replace('a', "x");

    let before = sprite.get_appearance();
    sprite.change_anchor_replacement_character('x');
    let after = sprite.get_appearance();

    assert_eq!(before, expected_before);
    assert_eq!(after, expected_after);
  }

  #[test]
  fn get_anchor_index_logic() {
    let sprite = Sprite::new("-x-\n-a-", 'a', '-', '-').unwrap();

    let expected_index = 4;
    let expected_coordinates = (1, 1);

    let index = sprite.get_anchor_index();
    let coordinates = sprite.get_anchor_as_coordinates();

    assert_eq!(coordinates, expected_coordinates);
    assert_eq!(index, expected_index);
  }

  #[test]
  fn calculate_anchor_index_error_logic() {
    let anchor_character = 'x';
    let no_anchor = "---";
    let multuiple_anchors = "x-x";

    let multiple_anchor_error = ModelError::MultipleAnchorsFound(vec![0, 2]);
    let no_anchor_error = ModelError::NoAnchor;

    let no_anchor_result = Sprite::calculate_anchor_index(no_anchor, anchor_character).unwrap_err();
    let multiple_anchor_result =
      Sprite::calculate_anchor_index(multuiple_anchors, anchor_character).unwrap_err();

    assert_eq!(no_anchor_result, no_anchor_error);
    assert_eq!(multiple_anchor_result, multiple_anchor_error);
  }

  #[test]
  fn calculate_anchor_index_logic() {
    let anchor_character = 'x';
    let shape = "---\n-x-";

    let expected_position = 4;

    let position = Sprite::calculate_anchor_index(shape, anchor_character).unwrap();

    assert_eq!(position, expected_position);
  }

  #[test]
  fn validity_check_all_errors() {
    let junk_sprite = Sprite::new_unchecked("x-x\naa", 'a', 'a', 'a', 100);

    let expected_error_list = Err(ModelError::SpriteValidityChecks(vec![
      ModelError::NonRectangularShape,
      ModelError::MultipleAnchorsFound(vec![3, 4]),
      ModelError::SpriteAnchorMatchesAirCharacter,
    ]));

    let result = junk_sprite.validity_check();

    assert_eq!(result, expected_error_list);
  }
}
