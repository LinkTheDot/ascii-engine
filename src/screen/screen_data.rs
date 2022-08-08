use crate::general_data::coordinates::*;
use crate::objects::object_data::*;
use std::collections::{btree_map::Entry::Vacant, BTreeMap};
use std::error::Error;
use std::iter;
use std::sync::mpsc::*;

pub const GRID_WIDTH: usize = 175; // further testing may be required but it seems fine
pub const GRID_HEIGHT: usize = 40;
pub const EMPTY_PIXEL: &str = "O";

pub type Key = String;
pub type ObjectDisplay = String;
pub type KeyAndObjectDisplay = (Key, ObjectDisplay);

pub struct ScreenData {
  screen_update_receiver: Receiver<String>,
  screen_update_sender: Sender<String>,
  screen: Vec<Pixel>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pixel {
  assigned_display: Option<Key>,
  objects_within: BTreeMap<Key, Vec<ObjectDisplay>>,
}

impl Pixel {
  pub fn new() -> Self {
    Pixel {
      assigned_display: None,
      objects_within: BTreeMap::new(),
    }
  }

  pub fn display(&self) -> String {
    if let Some(display_key) = &self.assigned_display {
      self.objects_within.get(display_key).unwrap()[0].to_string()
    } else {
      EMPTY_PIXEL.to_string()
    }
  }

  pub fn change_display_to(&mut self, change_to: Key) {
    if self.objects_within.contains_key(&change_to) {
      self.assigned_display = Some(change_to);
    }
  }

  pub fn insert_object(&mut self, add_object: KeyAndObjectDisplay) {
    if let Vacant(entry) = self.objects_within.entry(add_object.0.clone()) {
      entry.insert(vec![add_object.1]);

      self.assigned_display = Some(add_object.0);
    } else {
      self
        .objects_within
        .get_mut(&add_object.0)
        .unwrap()
        .push(add_object.1);
    }
  }

  pub fn remove_displayed_object(&mut self) -> Option<KeyAndObjectDisplay> {
    if !self.is_empty() {
      if self.assigned_key_has_multiple_objects() {
        let removed_object_display = self
          .objects_within
          .get_mut(self.assigned_display.as_ref().unwrap())
          .unwrap()
          .remove(0);

        let copy_of_assinged_key = self.assigned_display.as_ref().unwrap().clone();

        Some((copy_of_assinged_key, removed_object_display))
      } else {
        let mut removed_object_display = self
          .objects_within
          .remove_entry(self.assigned_display.as_ref().unwrap())
          .unwrap();

        self.assigned_display = self.check_if_available_object().cloned();

        Some((removed_object_display.0, removed_object_display.1.remove(0)))
      }
    } else {
      None
    }
  }

  pub fn get_current_display_data(&self) -> Option<&Vec<ObjectDisplay>> {
    if let Some(assigned_key) = &self.assigned_display {
      Some(self.objects_within.get(assigned_key).unwrap())
    } else {
      None
    }
  }

  pub fn is_empty(&self) -> bool {
    self.objects_within.len() == 0
  }

  /// checks if there's a key still in the map and if so returns
  /// a reference to said key
  pub fn check_if_available_object(&self) -> Option<&Key> {
    self.objects_within.keys().next()
  }

  /// checks if the input key is within the map
  pub fn contains_object(&self, key: &Key) -> bool {
    self.objects_within.contains_key(key)
  }

  /// checks if the data corresponding to the assigned display key
  /// has more than 1 object within it
  pub fn assigned_key_has_multiple_objects(&self) -> bool {
    if let Some(assigned_key) = &self.assigned_display {
      self.objects_within.get(assigned_key).unwrap().len() > 1
    } else {
      false
    }
  }
}

impl ScreenData {
  pub fn new() -> Result<ScreenData, Box<dyn Error>> {
    let (screen_update_sender, screen_update_receiver) = channel();
    let screen = generate_pixel_grid();

    Ok(ScreenData {
      screen_update_receiver,
      screen_update_sender,
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

  pub fn change_pixel_display_at(&mut self, pixel_at: &Coordinates, change_to: Key) {
    self.screen[pixel_at.coordinates_to_index()].change_display_to(change_to)
  }

  pub fn insert_object_at(&mut self, pixel_at: &Coordinates, object: &KeyAndObjectDisplay) {
    self.screen[pixel_at.coordinates_to_index()].insert_object(object.clone())
  }

  pub fn insert_all_objects_at(
    &mut self,
    pixel_at: &Coordinates,
    objects: Vec<KeyAndObjectDisplay>,
  ) {
    for object_data in objects {
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

  pub fn remove_surface_data_at(&mut self, pixel_at: &Coordinates) -> Option<KeyAndObjectDisplay> {
    self.get_mut_pixel_at(pixel_at).remove_displayed_object()
  }

  // this needs to be rewritten
  /// this will take the latest object data within the first
  /// inserted pixel coordinates and move it to the second
  /// data of the overwritten pixel
  pub fn replace_latest_object_in_pixel(
    &mut self,
    pixel_1: &Coordinates,
    pixel_2: &Coordinates,
  ) -> Option<KeyAndObjectDisplay> {
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

  // if i ever need this then remake it to work with a BTreeMap of vectors
  //
  // /// this will take all of the object data within the first
  // /// inserted pixel coordinates and move it to the second
  // /// then return the data of the overwritten pixel
  // pub fn replace_all_objects_in_pixel(
  // &mut self,
  // pixel_1: &Coordinates,
  // pixel_2: &Coordinates,
  // ) -> Vec<KeyAndObjectDisplay> {
  // if !self.pixel_is_empty(pixel_1) {
  // let pixel_1_data = self
  // .get_mut_pixel_at(pixel_1)
  // .objects_within
  // .drain(..)
  // .collect::<Vec<ObjectAndDisplay>>();

  // let pixel_2_data = self
  // .get_mut_pixel_at(pixel_2)
  // .objects_within
  // .drain(..)
  // .collect::<Vec<ObjectAndDisplay>>();

  // self.insert_all_objects_at(pixel_2, &pixel_1_data);

  // pixel_2_data
  // } else {
  // vec![]
  // }
  // }

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
}

pub fn generate_pixel_grid() -> Vec<Pixel> {
  iter::repeat(Pixel::new())
    .take(GRID_WIDTH * GRID_HEIGHT)
    .collect()
}
