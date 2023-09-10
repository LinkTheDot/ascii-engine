use crate::errors::*;
use engine_math::{prelude::UsizeMethods, rectangle::*};
use serde::{Deserialize, Serialize};

/// The hitbox will be how objects know the space they take up in the world.
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct Hitbox {
  hitbox_anchor_index: usize,
  dimensions: Rectangle,
  empty_hitbox: bool,
}

impl Hitbox {
  /// Creates a new hitbox from the passed in data and anchor to the skin.
  ///
  /// NOTE
  /// "skin_anchor_coordinates" is the internal coordinates of the anchor within the model's current appearance.
  ///
  /// That would mean if you had a skin like such:
  /// ```no_run,bash,ignore
  /// xxx
  /// xax
  /// xxx
  /// ```
  /// you would pass in (1, 1).
  pub fn new(dimensions: Rectangle, hitbox_anchor_index: usize) -> Self {
    Self {
      hitbox_anchor_index,
      dimensions,
      empty_hitbox: dimensions.area() == 0,
    }
  }

  pub fn assign_anchor_index(&mut self, new_index: usize) -> Result<(), ModelError> {
    if !Rectangle::index_is_valid(&self.dimensions, new_index) {
      return Err(ModelError::IndexLargerThanHitboxArea);
    }

    self.hitbox_anchor_index = new_index;

    Ok(())
  }

  pub fn assign_dimensions(&mut self, new_dimensions: Rectangle) -> Result<(), ModelError> {
    if !Rectangle::index_is_valid(&new_dimensions, self.hitbox_anchor_index) {
      return Err(ModelError::IndexLargerThanHitboxArea);
    }

    self.dimensions = new_dimensions;

    Ok(())
  }

  pub fn get_hitbox_dimensions(&self) -> &Rectangle {
    &self.dimensions
  }

  pub fn get_anchor_index(&self) -> usize {
    self.hitbox_anchor_index
  }

  pub fn get_anchor_as_coordinates(&self) -> (usize, usize) {
    self
      .hitbox_anchor_index
      .index_to_coordinates(self.dimensions.x)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn assign_anchor_index_valid() {
    let hitbox_dimensions = Rectangle::from((10, 10));
    let mut hitbox = Hitbox::new(hitbox_dimensions, 5);

    assert!(
      hitbox.assign_anchor_index(99).is_ok(),
      "{} > {}",
      hitbox_dimensions.area(),
      99
    );
  }

  #[test]
  #[should_panic]
  fn assign_anchor_index_invalid() {
    let mut hitbox = Hitbox::new(Rectangle::from((10, 10)), 5);

    hitbox.assign_anchor_index(100).unwrap();
  }
}
