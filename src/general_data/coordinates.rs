/// (x, y)
pub type Coordinates = (usize, usize);

pub enum Movements {
  Up,
  Down,
  Left,
  Right,
}

pub trait CoordinateMethods {
  fn coordinates_to_index(&self, width: usize) -> usize;

  fn add(&self, add: Coordinates) -> Self;
  fn subtract(&self, subtract: Coordinates) -> (isize, isize);

  fn get_coordinates_in_between(&self, bottom_right: &Self) -> Vec<Coordinates>;
}

#[allow(non_camel_case_types)]
pub trait usizeMethods {
  /// Returns (x, y).
  fn index_to_coordinates(&self, width: usize) -> (usize, usize);
}

impl usizeMethods for usize {
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

  fn get_coordinates_in_between(&self, bottom_right: &Self) -> Vec<Coordinates> {
    let mut coordinates_in_between = vec![];
    let mut coordiates_to_add = *self;

    coordinates_in_between.push(coordiates_to_add);

    while &coordiates_to_add != bottom_right {
      if coordiates_to_add.0 == bottom_right.0 {
        coordiates_to_add = (self.0, coordiates_to_add.1 + 1);
      } else {
        coordiates_to_add.0 += 1
      }

      coordinates_in_between.push(coordiates_to_add);
    }

    coordinates_in_between
  }
}
