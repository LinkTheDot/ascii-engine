use crate::errors::*;
use engine_math::rectangle::Rectangle;
use std::cmp::Ordering;

/// The ``Sprite`` contains the data for how a model looks on the screen.
#[derive(Debug, PartialEq, Eq)]
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
  pub fn new() -> Self {
    Self::default()
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
  ) -> Result<(), ModelError> {
    if !Rectangle::string_is_valid_rectangle(&new_shape) {
      return Err(ModelError::NonRectangularShape);
    }

    let anchor_character = new_anchor_character.unwrap_or(self.anchor_character);

    if anchor_character == self.air_character {
      return Err(ModelError::SpriteAnchorMatchesAirCharacter);
    }

    let new_index = Self::calculate_anchor_index(&new_shape, self.anchor_character)?;

    self.shape = new_shape;
    self.anchor_character_index = new_index;
    self.anchor_character = anchor_character;

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

    let _ = self
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

  /// Calculates the index for the anchor_character in the shape and returns it.
  /// Does NOT account for new lines in the shape string.
  ///
  /// # Errors
  ///
  /// - The stored shape doesn't have an anchor.
  /// - The stored shape has multiple anchors.
  fn calculate_anchor_index(shape: &str, anchor_character: char) -> Result<usize, ModelError> {
    let shape_anchor_indices: Vec<usize> = shape
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
}
