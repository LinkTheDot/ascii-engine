use crate::screen::screen_data::GRID_WIDTH;

pub type Coordinates = (usize, usize);

pub trait CoordinateMethods {
  fn index_to_coordinates(index: usize) -> Self;
  fn coordinates_to_index(&self) -> usize;
}

impl CoordinateMethods for Coordinates {
  fn index_to_coordinates(index: usize) -> Self {
    (index / GRID_WIDTH, index % GRID_WIDTH)
  }

  fn coordinates_to_index(&self) -> usize {
    self.0 + GRID_WIDTH * self.1
  }
}
