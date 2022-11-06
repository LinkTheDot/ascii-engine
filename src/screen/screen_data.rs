use crate::general_data::coordinates::*;
use crate::objects::object_data::ObjectInformation;
use crate::screen::{object_screen_data::*, pixel::*};
use std::collections::HashMap;
use std::error::Error;
use std::iter;
use thread_clock::Clock;

pub const GRID_WIDTH: usize = 175;
pub const GRID_HEIGHT: usize = 40;
pub const TICK_DURATION: u32 = 24;
pub const EMPTY_PIXEL: &str = "O";
pub const GRID_SPACER: &str = "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n";

pub type Key = String;
pub type ObjectDisplay = String;
pub type KeyAndObjectDisplay = (Key, AssignedObject);
pub type CurrentAndTotalObjects = (CurrentlyExistingObjects, TotalExistingObjects);

/// This is in the context of the update_placed_objects function
/// but could technically be used anywhere
pub enum Actions {
  Add,
  Subtract,
}

/// Contains all of the data for the screen such as
/// The clock
/// The counter for all objects that exist
/// The set of pixels that make up the screen
pub struct ScreenData {
  screen_clock: Clock,
  existing_objects: HashMap<String, ObjectScreenData>,
  screen: Vec<Pixel>,
}

impl ScreenData {
  /// Creates a new screen which in the process starts a new clock
  /// thread
  pub fn new() -> Result<ScreenData, Box<dyn Error>> {
    let mut screen_clock = Clock::custom(TICK_DURATION).unwrap_or_else(|error| {
      panic!("An error has occurred while spawning a clock thread: '{error}'")
    });
    let screen = generate_pixel_grid();

    screen_clock.start();

    Ok(ScreenData {
      screen_clock,
      existing_objects: HashMap::new(),
      screen,
    })
  }

  /// Returns the screen as a string depending on what each pixel
  /// is assigned
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

  /// Changes the assigned_display of the pixel
  /// If the input is invalid it'll set the display to None
  pub fn change_pixel_display_at(
    &mut self,
    pixel_at: &Coordinates,
    change_to: Option<Key>,
    assigned_number: Option<AssignedNumber>,
  ) {
    self.screen[pixel_at.coordinates_to_index()].change_display_to(change_to, assigned_number)
  }

  /// Inserts the object at the given coordinates
  /// and if there's no assignment it'll assign
  /// the inserted object to the pixel if reassign is true
  pub fn insert_object_at(
    &mut self,
    at_pixel: &Coordinates,
    insert: KeyAndObjectDisplay,
    reassign: bool,
  ) {
    self.screen[at_pixel.coordinates_to_index()].insert_object(insert.0, insert.1, reassign)
  }

  /// Inserts all objects passed in to the
  /// given coordinates, does not reassign
  /// the pixel for any of them
  pub fn insert_all_objects_at(
    &mut self,
    pixel_at: &Coordinates,
    objects: Vec<KeyAndObjectDisplay>,
  ) {
    for object_data in objects {
      self.insert_object_at(pixel_at, object_data, false)
    }
  }

  /// Returns true if the pixel contains no object data
  pub fn pixel_is_empty(&self, pixel_at: &Coordinates) -> bool {
    self.screen[pixel_at.coordinates_to_index()].is_empty()
  }

  pub fn get_mut_pixel_at(&mut self, pixel_at: &Coordinates) -> &mut Pixel {
    &mut self.screen[pixel_at.coordinates_to_index()]
  }

  pub fn get_pixel_at(&self, pixel_at: &Coordinates) -> &Pixel {
    &self.screen[pixel_at.coordinates_to_index()]
  }

  /// Removes the data that's currently assigned to display and returns it
  /// Deletes the entry if there's only 1 object in there
  pub fn remove_displayed_object_data_at(
    &mut self,
    pixel_at: &Coordinates,
  ) -> Option<KeyAndObjectDisplay> {
    self.get_mut_pixel_at(pixel_at).remove_displayed_object()
  }

  /// This will take the assigned object display data within the first
  /// enserted pixel's coordinates and move it to the second
  /// data of the overwritten pixel is returned as an optional
  // change latest to assigned
  pub fn replace_latest_object_in_pixel(
    &mut self,
    pixel_1: &Coordinates,
    pixel_2: &Coordinates,
  ) -> Option<KeyAndObjectDisplay> {
    if !self.pixel_is_empty(pixel_1) {
      let pixel_1_data = self.remove_displayed_object_data_at(pixel_1).unwrap();
      let pixel_2_data = if !self.pixel_is_empty(pixel_2) {
        self.remove_displayed_object_data_at(pixel_2)
      } else {
        None
      };

      self.insert_object_at(pixel_2, pixel_1_data, true);

      pixel_2_data
    } else {
      None
    }
  }

  /// Moves the data from pixel_1 to pixel_2 and reassigns
  /// the pixel to display the new data
  pub fn transfer_assigned_object_in_pixel_to(
    &mut self,
    pixel_1: &Coordinates,
    pixel_2: &Coordinates,
  ) {
    let object = self.remove_displayed_object_data_at(pixel_1);

    if let Some(object) = object {
      self.insert_object_at(pixel_2, object, true);
    }
  }

  /// Returns the currently existing and total number of objects
  /// that have existed with the same name
  pub fn object_already_exists(&self, name: &String) -> Option<CurrentAndTotalObjects> {
    self.existing_objects.get(name).map(|object_data| {
      (
        object_data.get_currently_existing(),
        object_data.get_total_count(),
      )
    })
  }

  /// If the object exists then it'll increment the total count
  /// if it does not exist then it'll add the object to the list
  pub fn update_existing_objects(&mut self, object: &ObjectInformation) {
    if self
      .existing_objects
      .contains_key(&object.get_name().to_string())
    {
      self
        .existing_objects
        .get_mut(&object.get_name().to_string())
        .unwrap()
        .increment_total();
    } else {
      let object_name = object.get_name().to_string();

      self
        .existing_objects
        .insert(object_name.clone(), ObjectScreenData::new(&object_name));

      self.update_total_objects(&object_name);
    }
  }

  /// Increments or Decrements the currently placed number of objects depending
  /// on what is passed in
  pub fn update_placed_objects(&mut self, name: &Key, action: Actions) {
    if let Some(object_screen_data) = self.existing_objects.get_mut(name) {
      match action {
        Actions::Add => object_screen_data.increment_current(),
        Actions::Subtract => object_screen_data.decrement_current(),
      }
    }
  }

  /// If the object exists then it'll increment the total number of them
  fn update_total_objects(&mut self, name: &Key) {
    if self.existing_objects.contains_key(name) {
      self
        .existing_objects
        .get_mut(name)
        .unwrap()
        .increment_total();
    }
  }

  /// Returns a reference to the given object_screen_data
  pub fn get_existing_object(&self, object: &String) -> Option<&ObjectScreenData> {
    if self.existing_objects.contains_key(object) {
      self.existing_objects.get(object)
    } else {
      None
    }
  }

  pub fn wait_for_x_ticks(&mut self, x: u32) {
    let _ = self.screen_clock.wait_for_x_ticks(x);
  }
}

/// Generates a 1-Dimensional grid of Pixels
pub fn generate_pixel_grid() -> Vec<Pixel> {
  iter::repeat(Pixel::new())
    .take(GRID_WIDTH * GRID_HEIGHT)
    .collect()
}
