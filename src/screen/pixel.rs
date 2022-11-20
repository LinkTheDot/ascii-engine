// Add a way to remove specific objects
// Add a way to remove all objects
// Have error handling for most methods
// Create an error type for Pixels

use crate::general_data::map_methods::*;
pub use crate::screen::pixel::{checks::*, pixel_assignments::*};
use crate::screen::pixel_data_types::*;
use crate::screen::screen_data::*;
use anyhow::anyhow;
use std::collections::{btree_map::Entry, BTreeMap, HashMap};

mod checks;
mod pixel_assignments;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Reassign {
  True,
  False,
}

/// A pixel makes up an individual part of the entire screen.
/// Pixels will hold a part of what the object within displays
/// as.
/// The assignment will determine what object that is currently
/// occupying the pixel will be displayed.
/// If there're multiple objects of the same name then the
/// assigned_display_number will determine which of those is
/// displayed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pixel {
  index: usize,
  assigned_display: Option<Key>,
  assigned_display_number: Option<AssignedNumber>,
  objects_within: BTreeMap<Key, AssignedObjects>,
}

impl Pixel {
  pub fn new(index: usize) -> Self {
    Pixel {
      index,
      assigned_display: None,
      assigned_display_number: None,
      objects_within: BTreeMap::new(),
    }
  }

  /// Returns the pixel display depending on what was assigned.
  /// If nothing was assigned then it'll return an EMPTY_PIXEL.
  pub fn display(&self) -> String {
    if let (Some(display_key), Some(assigned_display)) = &self.get_both_assignments() {
      self
        .objects_within
        .get(display_key.as_str())
        .unwrap()
        .get(assigned_display)
        .unwrap()
        .to_string()
    } else {
      EMPTY_PIXEL.to_string()
    }
  }

  /// Changes the assigned_display of the pixel
  pub fn change_display_to(
    &mut self,
    key: Key,
    assigned_number: AssignedNumber,
  ) -> anyhow::Result<()> {
    if self.contains_object(&key) {
      self.change_display(Some(key), Some(assigned_number));

      Ok(())
    } else {
      // add logging here
      let error_message = format!("No object named {} found in pixel {}", key, self.index);

      Err(anyhow!(error_message))
    }
  }

  /// Clears the current assignment on the pixel.
  pub fn clear_display_data(&mut self) {
    self.change_display(None, None);
  }

  /// Inserts the object assigned to the key whether it existed or not.
  ///
  /// If reassign is true, the assigned value to the pixel will change to the new object.
  pub fn insert_object(&mut self, key: Key, object: AssignedObject, reassign: Reassign) {
    if let Entry::Occupied(mut object_map) = self.objects_within.entry(key.clone()) {
      object_map.get_mut().insert(object.0, object.1);
    } else {
      let new_map = HashMap::from([(object.0, object.1)]);

      self.objects_within.insert(key.clone(), new_map);
    }

    if reassign == Reassign::True {
      // handle this once pixel errors are implemented
      let _ = self.change_display_to(key, object.0);
    }
  }

  /// Removes the data that's currently assigned to display and returns it
  /// Deletes the entry if there's only one object in there
  ///
  /// Reassign will automatically assign to the latest object inside the pixel
  // Change this to return a result once pixel errors are implemented
  pub fn remove_displayed_object(&mut self, reassign: Reassign) -> Option<KeyAndObjectDisplay> {
    if let (Some(key), Some(assigned_number)) = self.take_both_assignments() {
      if self.contains_object(&key) {
        let removed_data = self
          .remove_object(&key, &assigned_number, reassign)
          .unwrap();

        if reassign == Reassign::False {
          self.clear_display_data();
        }

        return Some(removed_data);
      }
    }

    None
  }

  /// Removes the data corresponding to the passed in object, and returns it.
  /// If the object doesn't exist then None is returned.
  pub fn remove_object(
    &mut self,
    key: &Key,
    assigned_number: &AssignedNumber,
    reassign: Reassign,
  ) -> Option<KeyAndObjectDisplay> {
    let removed_data = if self.contains_assignment(key, assigned_number) {
      //
      // None of these unwraps will panic as we
      // know the objects exist in the map
      //
      if self.contains_multiple_of(key) {
        let assigned_object = self
          .objects_within
          .get_mut(key)
          .unwrap()
          .remove_entry(assigned_number)
          .unwrap();

        Some((key.clone(), assigned_object))
      } else {
        let assigned_object = self
          .objects_within
          .remove(key)
          .unwrap()
          .remove_entry(assigned_number)
          .unwrap();

        Some((key.clone(), assigned_object))
      }
    } else {
      None
    };

    if removed_data.is_some() && reassign == Reassign::True {
      self.clear_display_data();

      self.reassign_display_data();
    }

    removed_data
  }

  /// Gets a reference to the display item currently inside the pixel
  pub fn get_current_display_data(&self) -> Option<(&Key, &ObjectDisplay)> {
    if let (Some(assigned_key), Some(assigned_number)) = &self.get_both_assignments() {
      if self.contains_object(assigned_key) {
        let object_display = self
          .get(assigned_key)
          .unwrap()
          .get(assigned_number)
          .unwrap();

        Some((assigned_key, object_display))
      } else {
        None
      }
    } else {
      None
    }
  }

  // redo this and make it more descriptive
  pub fn get_all_objects_of_assigned_key(&self) -> Option<&AssignedObjects> {
    if let Some(assigned_key) = &self.assigned_display {
      if self.contains_object(assigned_key) {
        Some(self.objects_within.get(assigned_key).unwrap())
      } else {
        None
      }
    } else {
      None
    }
  }

  /// Gets the object within as a reference
  pub fn get(&self, key: &Key) -> Option<&AssignedObjects> {
    self.objects_within.get(key)
  }

  /// Gets the object within as a mutable reference
  pub fn get_mut(&mut self, key: &Key) -> Option<&mut AssignedObjects> {
    self.objects_within.get_mut(key)
  }

  /// Gets the latest inserted object and returns it's key
  // This will be removed once objects are assigned with unique hashes
  pub fn get_latest_object_key(&self) -> Option<&Key> {
    self.objects_within.get_first_key()
  }

  /// Checks for the latest object that's been inserted into the pixel
  /// then gets the lowest number of that object
  pub fn get_new_object_assignment(&self) -> Option<(Key, AssignedNumber)> {
    if self.has_no_assignment() {
      if let Some(key) = self.get_latest_object_key() {
        let lowest_display_number = self.get(key).unwrap().get_lowest_key().unwrap();
        Some((key.clone(), *lowest_display_number))
      } else {
        None
      }
    } else {
      None
    }
  }

  /// Checks if there's an available object in the pixel
  /// if so it'll assign the pixel to that
  /// if not it'll do nothing
  pub fn reassign_display_data(&mut self) {
    if let Some((key, assigned_number)) = self.get_new_object_assignment() {
      let _ = self.change_display_to(key, assigned_number);
    } else {
      self.clear_display_data();
    }
  }

  pub fn get_all_data(&self) -> Vec<(&Key, &AssignedObjects)> {
    self.objects_within.iter().collect()
  }

  /// Changes the assigned display data
  fn change_display(&mut self, display: Option<Key>, number: Option<AssignedNumber>) {
    self.assigned_display = display;
    self.assigned_display_number = number;
  }
}
