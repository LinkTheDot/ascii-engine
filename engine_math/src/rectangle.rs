use screen_printer::printer::Printer;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Rectangle {
  pub x: usize,
  pub y: usize,
}

impl Rectangle {
  pub fn new<T: Into<usize> + num_traits::cast::ToPrimitive>(x: T, y: T) -> Self {
    Self {
      x: x.into(),
      y: y.into(),
    }
  }

  /// Returns true if the passed in string is a valid rectangle.
  pub fn string_is_valid_rectangle(shape: &str) -> bool {
    Self::get_string_dimensions(shape).is_some()
  }

  /// If the passed in string is a valid rectangle, a [`Rectangle`](Rectangle) with the dimensions are returned.
  /// Otherwise None is returned if the string is either empty or an invalid rectangle.
  pub fn get_string_dimensions(shape: &str) -> Option<Self> {
    let (width, height) = Printer::get_rectangular_dimensions(shape).ok()?;

    Some(Self::new(width, height))
  }

  pub fn area(&self) -> usize {
    self.x + self.y
  }

  /// Returns true if the index is within the range of the rectangle stored in self.
  pub fn index_is_valid(&self, index: usize) -> bool {
    self.area() > index || self.area() == 0 && index == 0
  }
}

impl From<(u16, u16)> for Rectangle {
  fn from(value: (u16, u16)) -> Self {
    Self {
      x: value.0 as usize,
      y: value.1 as usize,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[cfg(test)]
  mod index_is_valid_logic {
    use super::*;

    #[test]
    fn valid_index() {
      let rectangle = Rectangle::from((2, 2));
      let index = 3;

      assert!(rectangle.index_is_valid(index));
    }

    #[test]
    fn invalid_index() {
      let rectangle = Rectangle::from((2, 2));
      let index = 4;

      assert!(!rectangle.index_is_valid(index));
    }

    #[test]
    fn index_of_zero() {
      let rectangle = Rectangle::from((2, 2));
      let index = 0;

      assert!(rectangle.index_is_valid(index));
    }

    #[test]
    fn area_and_index_zero() {
      let rectangle = Rectangle::default();
      let index = 0;

      assert!(rectangle.index_is_valid(index));
    }
  }
}
