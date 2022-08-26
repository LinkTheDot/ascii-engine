#![allow(unused)]

use crate::general_data::coordinates::*;
use crate::objects::object_movements::*;
use crate::screen::pixel::AssignedNumber;
use crate::screen::screen_data::*;

/// Contains the name, shape, position, and whether or not the data
/// should be kept once the count reaches 0
pub struct ObjectInformation<'a> {
  name: &'a str,
  object_shape: &'a str,
  position: Coordinates,
  keep_data: bool,
}

impl<'a> ObjectInformation<'a> {
  pub fn from(
    name: &'a str,
    object_shape: &'a str,
    position: Option<Coordinates>,
    keep_data: Option<bool>,
  ) -> Self {
    let position = if let Some(coords) = position {
      coords
    } else {
      (0, 0)
    };

    let keep_data = if let Some(keep_data) = keep_data {
      keep_data
    } else {
      false
    };

    ObjectInformation {
      name,
      object_shape,
      position,
      keep_data,
    }
  }

  pub fn get_name(&self) -> &'a str {
    self.name
  }
}

#[derive(Debug)]
/// An object's assigned number should have no relevance to itself
/// and is merely there to help the screen identify different objects
/// with the same name
pub struct Object {
  pub name: Key,
  pub number: AssignedNumber,
  pub width: usize,
  pub height: usize,
  pub object_shape: String,
  pub position: Coordinates,
}

impl Object {
  pub fn create(object_information: ObjectInformation, screen: &mut ScreenData) -> Self {
    Object {
      name: object_information.name.to_string(),
      number: 0,
      width: get_object_width(object_information.object_shape),
      height: get_object_height(object_information.object_shape),
      object_shape: object_information.object_shape.to_string(),
      position: object_information.position,
    }
  }

  pub fn place_object(&self, screen_data: &mut ScreenData) {
    let mut pixel_position = self.position;

    screen_data.update_placed_objects(&self.name, Actions::Add);

    for new_pixel_display in self.object_shape.chars() {
      match new_pixel_display {
        ' ' => {
          let pixel_object_group = (self.name.clone(), (self.number, EMPTY_PIXEL.to_string()));

          screen_data.insert_object_at(&pixel_position, pixel_object_group);

          pixel_position.0 += 1
        }
        '\n' => {
          pixel_position.0 = self.position.0;
          pixel_position.1 += 1;
        }
        _ => {
          let pixel_object_group = (
            self.name.clone(),
            (self.number, new_pixel_display.to_string()),
          );

          screen_data.insert_object_at(&pixel_position, pixel_object_group);

          pixel_position.0 += 1
        }
      }
    }
  }

  /// Gets the coordinates at the bottom right of the object
  pub fn get_bottom_right_of_object(&self) -> Coordinates {
    (
      self.position.0 + self.width - 1,
      self.position.1 + self.height - 1,
    )
  }

  pub fn movement_goes_out_of_bounds(&self, move_to: ObjectMovements) -> bool {
    let new_position = match self.position.move_coords(&move_to) {
      Some(coords) => coords,
      None => return false,
    };

    new_position
      .get_object_bounds(&move_to, self.width, self.height)
      .is_some()
  }
}

pub fn get_object_width(object_shape: &str) -> usize {
  object_shape.split('\n').next().unwrap().len()
}

pub fn get_object_height(object_shape: &str) -> usize {
  object_shape.split('\n').count()
}
