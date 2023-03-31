use crate::models::errors::*;

#[allow(unused)]
use log::debug;

/// The Sprite is the display data of a model.
#[derive(Debug, PartialEq, Eq)]
pub struct Sprite {
  skin: Skin,
}

/// The Skin is how a model will appear on the screen.
///
/// When creating a skin's shape, anchor and air characters will need to be designated.
/// The anchor character will be replaced with the 'anchor_replacement_character' field when
/// building the shape of the Skin.
#[derive(Debug, PartialEq, Eq)]
pub struct Skin {
  pub shape: String,
  pub anchor_character: char,
  pub anchor_replacement_character: char,
  pub air_character: char,
  /// Doesn't count new lines
  anchor_character_index: usize,
}

impl Sprite {
  pub fn new(mut skin: Skin) -> Result<Self, ModelError> {
    skin.fix_skin();

    Ok(Self { skin })
  }

  pub fn get_anchor_character_index(&self) -> usize {
    self.skin.anchor_character_index
  }

  /// Returns a reference to the skin's shape
  pub fn get_shape(&self) -> &str {
    &self.skin.shape
  }

  /// Returns a mutable reference to the skin's shape
  pub fn get_mut_shape(&mut self) -> &mut String {
    &mut self.skin.shape
  }

  pub fn air_character(&self) -> char {
    self.skin.air_character
  }
}

impl Skin {
  pub fn new(
    shape: &str,
    anchor_character: char,
    anchor_replacement_character: char,
    air_character: char,
  ) -> Result<Self, ModelError> {
    let cleaned_shape = shape.replace('\n', "");
    let anchor_character_index = cleaned_shape
      .chars()
      .position(|pixel| pixel == anchor_character);

    match anchor_character_index {
      None => Err(ModelError::NoAnchor),
      Some(anchor_character_index) => Ok(Self {
        shape: shape.to_string(),
        anchor_character,
        anchor_replacement_character,
        air_character,
        anchor_character_index,
      }),
    }
  }

  /// Replaces the anchor character in the skin's shape with the given
  /// replacement character.
  fn fix_skin(&mut self) {
    self.shape = self.shape.replace(
      &self.anchor_character.to_string(),
      &self.anchor_replacement_character.to_string(),
    );
  }

  pub fn get_anchor_character_index(&self) -> usize {
    self.anchor_character_index
  }
}

#[cfg(test)]
mod skin_logic {
  use super::*;

  #[test]
  fn fix_skin_logic() {
    let mut skin = Skin::new("x-x\nxcx\nx-x", 'c', '-', '-').unwrap();

    let expected_shape = "x-x\nx-x\nx-x";

    skin.fix_skin();

    assert_eq!(skin.shape, expected_shape)
  }
}
