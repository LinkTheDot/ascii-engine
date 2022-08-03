#![allow(unused)]

use crate::general_data::coordinates::Coordinates;
use crate::screen::screen_data::*;

#[derive(PartialEq)]
pub enum ObjectMovements {
  Up,
  Down,
  Left,
  Right,
}

#[derive(Debug)]
pub struct Object {
  pub name: String,
  pub width: usize,
  pub height: usize,
  pub object_shape: String,
  pub position: Coordinates,
}

impl Object {
  pub fn create(name: &str, object_shape: &str, position: Option<Coordinates>) -> Self {
    Object {
      name: name.to_string(),
      width: get_object_width(object_shape),
      height: get_object_height(object_shape),
      object_shape: object_shape.to_string(),
      position: if let Some(coords) = position {
        coords
      } else {
        (0, 0)
      },
    }
  }

  // try and implement get_coordinates_in_between and just iterate through
  // all the coordinates
  pub fn place_object(&self, screen_data: &mut ScreenData) {
    let mut pixel_position = self.position;

    for new_pixel_display in self.object_shape.chars() {
      match new_pixel_display {
        ' ' => {
          screen_data.change_pixel_display_at(&pixel_position, EMPTY_PIXEL);
          screen_data.insert_object_at(&pixel_position, &self.name);

          pixel_position.0 += 1
        }
        '\n' => {
          pixel_position.0 = self.position.0;
          pixel_position.1 += 1;
        }
        _ => {
          screen_data.change_pixel_display_at(&pixel_position, &new_pixel_display.to_string());
          screen_data.insert_object_at(&pixel_position, &self.name);

          pixel_position.0 += 1
        }
      }
    }
  }

  // latest thing being worked on, need the squared shape
  pub fn move_object(&mut self, screen_data: &mut ScreenData, move_to: ObjectMovements) {
    match move_to {
      ObjectMovements::Up => self.position.1 - 1,
      ObjectMovements::Down => self.position.1 + 1,
      ObjectMovements::Left => self.position.0 - 1,
      ObjectMovements::Right => self.position.0 + 1,
    };
  }

  pub fn get_bottom_right_of_objecrt(&self) -> Coordinates {
    todo!()
  }
}

pub fn get_object_width(object_shape: &str) -> usize {
  object_shape.split('\n').next().unwrap().len()
}

pub fn get_object_height(object_shape: &str) -> usize {
  object_shape.split('\n').count()
}
