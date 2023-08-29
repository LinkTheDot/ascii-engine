// Clean up this entire thing to work with generics.

/// (x, y)
pub type Coordinates = (usize, usize);

pub trait CoordinateMethods {
  /// Converts coordinates into an index of the given grid width.
  fn coordinates_to_index(&self, width: usize) -> usize;

  /// Returns the sum of the given coordinates.
  fn add(&self, add: Coordinates) -> Self;
  /// Returns the difference of the x and y values of the given coordinates.
  ///
  /// Self - Other
  fn subtract(&self, subtract: Coordinates) -> (isize, isize);

  /// Converts the coordinates (usize, usize) into (isize, isize).
  fn to_isize(&self) -> (isize, isize);

  /// Converts a tuple of (isize, isize) and converts it to (usize, usize)
  fn from_isize(coords: (isize, isize)) -> Self;
}

#[allow(non_camel_case_types)]
pub trait UsizeMethods {
  /// Converts the given index to a set of coordinates of the passed in grid width.
  fn index_to_coordinates(&self, width: usize) -> (usize, usize);
}

impl UsizeMethods for usize {
  fn index_to_coordinates(&self, width: usize) -> (usize, usize) {
    (self % width, self / width)
  }
}

impl CoordinateMethods for Coordinates {
  fn coordinates_to_index(&self, width: usize) -> usize {
    self.0 + width * self.1
  }

  fn add(&self, add: Coordinates) -> Self {
    (self.0 + add.0, self.1 + add.1)
  }

  fn subtract(&self, subtract: Coordinates) -> (isize, isize) {
    (
      self.0 as isize - subtract.0 as isize,
      self.1 as isize - subtract.1 as isize,
    )
  }

  fn to_isize(&self) -> (isize, isize) {
    (self.0 as isize, self.1 as isize)
  }

  fn from_isize(coords: (isize, isize)) -> Self {
    (coords.0 as usize, coords.1 as usize)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn add_logic() {
    let left = (1, 1);
    let right = (1, 1);

    let expected_result = (2, 2);

    let result = left.add(right);

    assert_eq!(result, expected_result);
  }

  #[test]
  fn subtract_logic() {
    let left = (1, 1);
    let right = (1, 1);

    let expected_result = (0, 0);

    let result = left.subtract(right);

    assert_eq!(result, expected_result);
  }
}
