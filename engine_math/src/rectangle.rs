use screen_printer::printer::Printer;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub struct Rectangle {
  pub x: usize,
  pub y: usize,
}

impl Rectangle {
  // <T: Into<usize> + num_traits::cast::ToPrimitive>
  pub fn new(x: usize, y: usize) -> Self {
    Self { x, y }
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
    self.x * self.y
  }

  /// Returns true if the given index is within the bounds of the Rectangle.
  ///
  /// That means if you have a rectangle of size (5, 5), and you check for an index of
  /// 25, false is returned. That is because the max index in a rectangle of (5, 5) is 24.
  pub fn index_is_valid(&self, index: usize) -> bool {
    self.area() > index || self.area() == 0 && index == 0
  }

  /// Returns true if the two rectangles are colliding.
  pub fn is_colliding(
    &self,
    self_position: (isize, isize),
    other: &Self,
    other_position: (isize, isize),
  ) -> bool {
    // x1 < x2 + w2 &&
    // x2 < x1 + w1 &&
    // y1 < y2 + h2 &&
    // y2 < y1 + h1
    self_position.0 < other_position.0 + other.x as isize
      && other_position.0 < self_position.0 + self.x as isize
      && self_position.1 < other_position.1 + other.y as isize
      && other_position.1 < self_position.1 + self.y as isize
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
      let rectangle = Rectangle::from((2, 20));
      let index = 39;

      assert!(rectangle.index_is_valid(index));
    }

    #[test]
    fn invalid_index() {
      let rectangle = Rectangle::from((2, 20));
      let index = 40;

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

  #[cfg(test)]
  mod is_colliding_logic {
    use super::*;

    #[test]
    fn is_colliding() {
      let rectangle_one = Rectangle::from((10, 10));
      let rectangle_one_position = (0, 0);
      let rectangle_two = Rectangle::new(10, 10);
      let rectangle_two_position = (9, 0);

      assert!(rectangle_one.is_colliding(
        rectangle_one_position,
        &rectangle_two,
        rectangle_two_position
      ));
    }

    #[test]
    fn is_not_colliding() {
      let rectangle_one = Rectangle::from((10, 10));
      let rectangle_one_position = (0, 0);
      let rectangle_two = Rectangle::new(10, 10);
      let rectangle_two_position = (10, 0);

      assert!(!rectangle_one.is_colliding(
        rectangle_one_position,
        &rectangle_two,
        rectangle_two_position
      ));
    }

    #[test]
    fn area_is_zero() {
      let rectangle_one = Rectangle::new(0, 0);
      let rectangle_one_position = (0, 0);
      let rectangle_two = Rectangle::new(10, 10);
      let rectangle_two_position = (0, 0);

      assert!(!rectangle_one.is_colliding(
        rectangle_one_position,
        &rectangle_two,
        rectangle_two_position
      ));
    }
  }
}
