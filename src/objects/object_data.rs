#![allow(unused)]

use crate::general_data::coordinates::*;
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

  pub fn place_object(&self, screen_data: &mut ScreenData) {
    let mut pixel_position = self.position;

    for new_pixel_display in self.object_shape.chars() {
      match new_pixel_display {
        ' ' => {
          let pixel_object_group = (self.name.clone(), EMPTY_PIXEL.to_string());

          screen_data.insert_object_at(&pixel_position, &pixel_object_group);

          pixel_position.0 += 1
        }
        '\n' => {
          pixel_position.0 = self.position.0;
          pixel_position.1 += 1;
        }
        _ => {
          let pixel_object_group = (self.name.clone(), new_pixel_display.to_string());

          screen_data.insert_object_at(&pixel_position, &pixel_object_group);

          pixel_position.0 += 1
        }
      }
    }
  }

  pub fn get_bottom_right_of_object(&self) -> Coordinates {
    (
      self.position.0 + self.width - 1,
      self.position.1 + self.height - 1,
    )
  }
}

pub fn get_object_width(object_shape: &str) -> usize {
  object_shape.split('\n').next().unwrap().len()
}

pub fn get_object_height(object_shape: &str) -> usize {
  object_shape.split('\n').count()
}
