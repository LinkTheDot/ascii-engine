use crate::general_data::coordinates::*;
use crate::objects::object_data::*;

pub const SQUARE_SHAPE: &str = // /
  "xxxX  Xxxx
xxX    Xxx
xX      Xx
xxX    Xxx
xxxX  Xxxx";

#[allow(unused)]
fn square_instructions(instructions: String) {
  todo!()
}

pub trait Square {
  fn create_hollow_square(position: Option<Coordinates>) -> Object;
}

impl Square for Object {
  fn create_hollow_square(position: Option<Coordinates>) -> Object {
    Object::create("square", 1, 1, SQUARE_SHAPE, square_instructions, position)
  }
}
