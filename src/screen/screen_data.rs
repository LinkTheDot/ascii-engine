#![allow(unused)]

use crate::general_data::coordinates::*;
use crate::objects::object_data::*;
use std::collections::HashMap;
use std::error::Error;
use std::iter;
use std::sync::mpsc::*;

pub const GRID_WIDTH: usize = 175; // further testing may be required but it seems fine
pub const GRID_HEIGHT: usize = 40;
pub const EMPTY_PIXEL: &str = "O";

pub type ObjectAndDisplay = (String, String);

pub struct ScreenData {
  screen_update_receiver: Receiver<String>,
  screen_update_sender: Sender<String>,
  object_data: HashMap<String, Object>,
  screen: Vec<Pixel>,
}

// change this from a vector to a hashmap that uses objects as keys
// and the display as the data inside
//
// look more into hashmaps to copy everything you've done with vectors
#[derive(Clone, Debug, PartialEq)]
pub struct Pixel {
  pub objects_within: Vec<ObjectAndDisplay>,
}

impl Pixel {
  pub fn new() -> Self {
    Pixel {
      objects_within: vec![],
    }
  }

  pub fn display(&self) -> String {
    if self.objects_within.is_empty() {
      EMPTY_PIXEL.to_string()
    } else {
      self.objects_within[0].1.clone()
    }
  }

  pub fn change_display_to(&mut self, change_to: &str) {
    self.objects_within[0].1 = change_to.to_string()
  }

  pub fn insert_object(&mut self, add_object: &ObjectAndDisplay) {
    self.objects_within.push(add_object.clone())
  }

  pub fn remove_object(&mut self) -> Option<ObjectAndDisplay> {
    if !self.is_empty() {
      Some(self.objects_within.remove(0))
    } else {
      None
    }
  }

  pub fn is_empty(&self) -> bool {
    self.objects_within.len() == 0
  }
}

impl ScreenData {
  pub fn new() -> Result<ScreenData, Box<dyn Error>> {
    let (screen_update_sender, screen_update_receiver) = channel();
    let screen = generate_pixel_grid();

    Ok(ScreenData {
      screen_update_receiver,
      screen_update_sender,
      object_data: HashMap::new(),
      screen,
    })
  }

  pub fn display(&self) -> String {
    self
      .screen
      .chunks(GRID_WIDTH)
      .map(|pixel_row| {
        pixel_row
          .iter()
          .map(|pixel| pixel.display())
          .collect::<String>()
          + "\n"
      })
      .collect()
  }

  pub fn change_pixel_display_at(&mut self, pixel_at: &Coordinates, change_to: &str) {
    self.screen[pixel_at.coordinates_to_index()].change_display_to(change_to)
  }

  pub fn insert_object_at(&mut self, pixel_at: &Coordinates, change_to: &ObjectAndDisplay) {
    self.screen[pixel_at.coordinates_to_index()].insert_object(change_to)
  }

  pub fn insert_all_objects_at(
    &mut self,
    pixel_at: &Coordinates,
    change_to: &Vec<ObjectAndDisplay>,
  ) {
    for object_data in change_to {
      self.screen[pixel_at.coordinates_to_index()].insert_object(object_data)
    }
  }

  pub fn pixel_is_empty(&self, pixel_at: &Coordinates) -> bool {
    self.screen[pixel_at.coordinates_to_index()].is_empty()
  }

  pub fn get_mut_pixel_at(&mut self, pixel_at: &Coordinates) -> &mut Pixel {
    &mut self.screen[pixel_at.coordinates_to_index()]
  }

  pub fn get_pixel_at(&self, pixel_at: &Coordinates) -> &Pixel {
    &self.screen[pixel_at.coordinates_to_index()]
  }

  pub fn remove_surface_data_at(&mut self, pixel_at: &Coordinates) -> Option<ObjectAndDisplay> {
    self.get_mut_pixel_at(pixel_at).remove_object()
  }

  /// this will take the latest object data within the first
  /// inserted pixel coordinates and move it to the second
  /// data of the overwritten pixel
  pub fn replace_latest_object_in_pixel(
    &mut self,
    pixel_1: &Coordinates,
    pixel_2: &Coordinates,
  ) -> Option<ObjectAndDisplay> {
    if !self.pixel_is_empty(pixel_1) {
      let pixel_1_data = self.remove_surface_data_at(pixel_1).unwrap();
      let pixel_2_data = if !self.pixel_is_empty(pixel_2) {
        self.remove_surface_data_at(pixel_2)
      } else {
        None
      };

      self.insert_object_at(pixel_2, &pixel_1_data);

      pixel_2_data
    } else {
      None
    }
  }

  /// this will take all of the object data within the first
  /// inserted pixel coordinates and move it to the second
  /// then return the data of the overwritten pixel
  pub fn replace_all_objects_in_pixel(
    &mut self,
    pixel_1: &Coordinates,
    pixel_2: &Coordinates,
  ) -> Vec<ObjectAndDisplay> {
    if !self.pixel_is_empty(pixel_1) {
      let pixel_1_data = self
        .get_mut_pixel_at(pixel_1)
        .objects_within
        .drain(..)
        .collect::<Vec<ObjectAndDisplay>>();

      let pixel_2_data = self
        .get_mut_pixel_at(pixel_2)
        .objects_within
        .drain(..)
        .collect::<Vec<ObjectAndDisplay>>();

      self.insert_all_objects_at(pixel_2, &pixel_1_data);

      pixel_2_data
    } else {
      vec![]
    }
  }

  pub fn transfer_latest_object_in_pixel_to(
    &mut self,
    pixel_1: &Coordinates,
    pixel_2: &Coordinates,
  ) {
    let object = self.remove_surface_data_at(pixel_1);

    if let Some(object) = object {
      self.insert_object_at(pixel_2, &object);
    }
  }

  pub fn move_latest_object_to_bottom_at(&mut self, pixel_at: &Coordinates) {
    self
      .get_mut_pixel_at(pixel_at)
      .objects_within
      .rotate_right(1)
  }
}

pub fn generate_pixel_grid() -> Vec<Pixel> {
  iter::repeat(Pixel::new())
    .take(GRID_WIDTH * GRID_HEIGHT)
    .collect()
}
