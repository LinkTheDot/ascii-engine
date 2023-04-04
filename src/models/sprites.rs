use crate::models::errors::*;

#[allow(unused)]
use log::debug;

//
// Remove the Skin and just have the sprite contain all the internal data.
// There's no use for the skin anymore now that hitboxes are stored somewhere else.
//

/// The ``Sprite`` contains the data for how a model looks on the screen.
#[derive(Debug, PartialEq, Eq)]
pub struct Sprite {
  pub shape: String,
  pub anchor_character: char,
  pub anchor_replacement_character: char,
  pub air_character: char,
  /// Doesn't count new lines
  anchor_character_index: usize,
}

impl Sprite {
  /// Converts a Skin into a Sprite
  pub fn new(
    shape: &str,
    anchor_character: char,
    anchor_replacement_character: char,
    air_character: char,
  ) -> Result<Self, ModelError> {
    let mut shape = shape.to_string();

    let anchor_character_index = Self::get_anchor_index(&shape, anchor_character)?;
    Self::fix_shape(&mut shape, anchor_character, anchor_replacement_character);

    Ok(Self {
      shape,
      anchor_character,
      anchor_replacement_character,
      air_character,
      anchor_character_index,
    })
  }

  /// Returns the index of the anchor character in the sprite's appearance.
  pub fn get_anchor_character_index(&self) -> usize {
    self.anchor_character_index
  }

  /// Returns a reference to the skin's appearance
  pub fn get_shape(&self) -> &str {
    &self.shape
  }

  /// Returns a mutable reference to the skin's appearance
  // This should be removed
  pub fn get_mut_shape(&mut self) -> &mut String {
    &mut self.shape
  }

  /// Returns the character labeled as air in the sprite's appearance.
  pub fn air_character(&self) -> char {
    self.air_character
  }

  fn fix_shape(shape: &mut String, anchor_character: char, anchor_replacement_character: char) {
    *shape = shape.replace(
      &anchor_character.to_string(),
      &anchor_replacement_character.to_string(),
    )
  }

  fn get_anchor_index(shape: &str, anchor_character: char) -> Result<usize, ModelError> {
    shape
      .replace('\n', "")
      .chars()
      .position(|pixel| pixel == anchor_character)
      .ok_or(ModelError::NoAnchor)
  }
}
