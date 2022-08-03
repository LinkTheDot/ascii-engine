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

pub struct ScreenData {
  screen_update_receiver: Receiver<String>,
  screen_update_sender: Sender<String>,
  object_data: HashMap<String, Object>,
  screen: Vec<Pixel>,
}

#[derive(Clone, Debug)]
pub struct Pixel {
  display_as: String,
  objects_within: Vec<String>,
}

impl Pixel {
  pub fn new() -> Self {
    Pixel {
      display_as: EMPTY_PIXEL.to_string(),
      objects_within: vec![],
    }
  }

  pub fn display(&self) -> String {
    self.display_as.clone()
  }

  pub fn change_display_to(&mut self, change_to: &str) {
    self.display_as = change_to.to_string()
  }

  pub fn insert_object(&mut self, add_object: &str) {
    self.objects_within.push(add_object.to_string())
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

  pub fn change_pixel_display_at(&mut self, coords: &Coordinates, change_to: &str) {
    self.screen[coords.coordinates_to_index()].change_display_to(change_to)
  }

  pub fn insert_object_at(&mut self, coords: &Coordinates, change_to: &str) {
    self.screen[coords.coordinates_to_index()].insert_object(change_to)
  }
}

pub fn generate_pixel_grid() -> Vec<Pixel> {
  iter::repeat(Pixel::new())
    .take(GRID_HEIGHT * GRID_WIDTH)
    .collect()
}
