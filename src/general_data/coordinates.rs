use crate::objects::object_data::ObjectMovements;
use crate::screen::screen_data::{GRID_HEIGHT, GRID_WIDTH};

pub type Coordinates = (usize, usize);

pub trait CoordinateMethods {
  fn index_to_coordinates(index: usize) -> Self;
  fn coordinates_to_index(&self) -> usize;

  fn add(&self, add: Coordinates) -> Option<Coordinates>;
  fn subtract(&self, subtract: Coordinates) -> Option<Coordinates>;

  fn move_coords(&self, move_to: &ObjectMovements) -> Option<Coordinates>;

  fn get_coordinates_in_between(&self, bottom_right: &Self) -> Vec<Coordinates>;
}

impl CoordinateMethods for Coordinates {
  fn index_to_coordinates(index: usize) -> Self {
    (index / GRID_WIDTH, index % GRID_WIDTH)
  }

  fn coordinates_to_index(&self) -> usize {
    self.0 + GRID_WIDTH * self.1
  }

  fn add(&self, add: Coordinates) -> Option<Coordinates> {
    if self.0 != GRID_WIDTH && self.1 != GRID_HEIGHT {
      Some((self.0 - add.0, self.1 - add.1))
    } else {
      None
    }
  }

  fn subtract(&self, subtract: Coordinates) -> Option<Coordinates> {
    if self.0 != 0 && self.1 != 0 {
      Some((self.0 - subtract.0, self.1 - subtract.1))
    } else {
      None
    }
  }

  fn move_coords(&self, move_to: &ObjectMovements) -> Option<Coordinates> {
    match move_to {
      ObjectMovements::Up => self.subtract((0, 1)),
      ObjectMovements::Down => self.add((0, 1)),
      ObjectMovements::Left => self.subtract((1, 0)),
      ObjectMovements::Right => self.add((1, 0)),
    }
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
