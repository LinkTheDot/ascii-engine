use crate::general_data::coordinates::*;
use crate::objects::object_data::*;
use crate::screen::screen_data::*;

pub const SQUARE_SHAPE: &str = // /
  "xxxX  Xxxx
xxX    Xxx
xX      Xx
xxX    Xxx
xxxX  Xxxx";

pub trait Square {
  fn create_hollow_square(position: Option<Coordinates>) -> Object;

  fn move_object(&mut self, screen_data: &mut ScreenData, move_to: ObjectMovements);
}

impl Square for Object {
  fn create_hollow_square(position: Option<Coordinates>) -> Object {
    Object::create("square", SQUARE_SHAPE, position)
  }

  // latest thing being worked on, need the squared shape
  #[allow(unused)]
  fn move_object(&mut self, screen_data: &mut ScreenData, move_to: ObjectMovements) {
    match move_to {
      ObjectMovements::Up => {
        if self.position.1 != 0 {
          self.position.1 - 1;
        }
      }
      ObjectMovements::Down => {
        if self.position.1 + 1 != GRID_WIDTH {
          self.position.1 + 1;
        }
      }
      ObjectMovements::Left => {
        if self.position.0 != 0 {
          self.position.0 - 1;
        }
      }
      ObjectMovements::Right => {
        if self.position.0 + 1 != GRID_HEIGHT {
          self.position.0 + 1;
        }
      }
    };

    let bottom_right_of_square = self.get_bottom_right_of_object();
    let mut square_space = self
      .position
      .get_coordinates_in_between(&bottom_right_of_square);

    let new_square_space = square_space.into_iter().map(|x| x);
  }
}
