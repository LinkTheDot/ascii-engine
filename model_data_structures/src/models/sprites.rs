use crate::errors::*;
use engine_math::{prelude::UsizeMethods, rectangle::Rectangle};
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
    new_anchor_replacement_character: Option<char>,
  ) -> Result<(), ModelError> {
    if !Rectangle::string_is_valid_rectangle(&new_shape) {
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
  /// Returns the list of error(s) that may have been detected with the sprite's data.
  ///
  /// # Errors
  ///
  /// - The stored shape isn't rectangular.
  /// - The stored shape doesn't have an anchor.
  /// - The stored shape has multiple anchors.
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
}

// impl From<(String, char, char, char)> for Sprite {
//   fn from(item: (String, char, char, char)) -> Result<Self, ModelError> {
//     todo!()
//   }
// }
