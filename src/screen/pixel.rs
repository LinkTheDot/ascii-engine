// Add a way to remove specific objects
// Add a way to remove all objects
// Have error handling for most methods
// Create an error type for Pixels

use crate::general_data::map_methods::*;
use crate::screen::screen_data::*;
use anyhow::anyhow;
use std::collections::{btree_map::Entry, BTreeMap, HashMap};

pub type AssignedNumber = u32;
pub type AssignedObject = (AssignedNumber, ObjectDisplay);
pub type AssignedObjects = HashMap<AssignedNumber, ObjectDisplay>;

#[derive(PartialEq, Clone, Copy)]
pub enum Reassign {
  True,
  False,
}

#[derive(Clone, Debug, PartialEq)]
/// A pixel makes up an individual part of the entire screen.
/// Pixels will hold a part of what the object within displays
/// as.
/// The assignment will determine what object that is currently
/// occupying the pixel will be displayed.
/// If there're multiple objects of the same name then the
/// assigned_display_number will determine which of those is
/// displayed.
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
      return self
        .objects_within
        .get(display_key.as_str())
        .unwrap()
        .get(assigned_display)
        .unwrap()
        .to_string();
    } else if !self.is_empty() {
      let first_key = self.objects_within.get_first_key();

      if let Some(expected_key) = first_key {
        let pixel_data = self.objects_within.get(expected_key).unwrap();

        if !pixel_data.is_empty() {
          return pixel_data.get(&0).unwrap().to_string();
        }
      }
    }

    EMPTY_PIXEL.to_string()
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

  /// Clears the current assignment on the pixel
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
    if let Some(key) = self.get_assigned_key() {
      if self.contains_object(key) {
        let removed_data = self.remove_object_assigned_number(reassign).unwrap();
        let key = self.assigned_display.take().unwrap();

        if reassign == Reassign::False {
          self.clear_display_data();
        }

        return Some((key, removed_data));
      }
    }

    None
  }

  /// Removes the object pertaining to the assigned number
  /// if there's more than one it'll just remove the one
  /// if there's only one it'll remove the map inside along with the item
  pub fn remove_object_assigned_number(&mut self, reassign: Reassign) -> Option<AssignedObject> {
    let removed_data = if let (Some(object_key), Some(assigned_number)) = (
      self.assigned_display.as_ref(),
      self.assigned_display_number.as_ref(),
    ) {
      if self.objects_within.get(object_key).unwrap().len() > 1 {
        self
          .objects_within
          .get_mut(object_key)
          .unwrap()
          .remove_entry(assigned_number)
      } else {
        self
          .objects_within
          .remove(object_key)
          .unwrap()
          .remove_entry(assigned_number)
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
  pub fn get_current_display_data(&self) -> Option<&ObjectDisplay> {
    if let (Some(assigned_key), Some(assigned_number)) = &self.get_both_assignments() {
      if self.contains_object(assigned_key) {
        self.get(*assigned_key).unwrap().get(assigned_number)
      } else {
        None
      }
    } else {
      None
    }
  }

  // redo this and make it more descriptive
  pub fn get_all_current_display_data(&self) -> Option<&AssignedObjects> {
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

  /// Returns true if the pixel contains no object data
  pub fn is_empty(&self) -> bool {
    self.objects_within.is_empty()
  }

  /// Returns true if the input key/object is within the map
  pub fn contains_object(&self, key: &Key) -> bool {
    self.objects_within.contains_key(key)
  }

  /// Returns true if the data corresponding to the assigned display key
  /// has more than one object within it
  pub fn assigned_key_has_multiple_objects(&self) -> bool {
    if let Some(assigned_key) = &self.assigned_display {
      self.get(assigned_key).unwrap().len() > 1
    } else {
      false
    }
  }

  /// Gets a reference to the current assigned object key
  pub fn get_assigned_key(&self) -> Option<&Key> {
    self.assigned_display.as_ref()
  }

  /// Gets a reference to the current assigned_object_number key
  pub fn get_assigned_number(&self) -> Option<&AssignedNumber> {
    self.assigned_display_number.as_ref()
  }

  /// Gets a reference to both the current assigned object and object number
  pub fn get_both_assignments(&self) -> (Option<&Key>, Option<&AssignedNumber>) {
    (self.get_assigned_key(), self.get_assigned_number())
  }

  /// Gets a copy of the current assigned object key
  pub fn clone_assigned_key(&self) -> Option<Key> {
    self.assigned_display.clone()
  }

  /// Gets a copy of the current assigned_object_number key
  pub fn clone_assigned_number(&self) -> Option<AssignedNumber> {
    // add .clone() once objects use unique hashes
    self.assigned_display_number
  }

  /// Gets a copy of both the current assigned object and object number
  pub fn clone_both_assignments(&self) -> (Option<Key>, Option<AssignedNumber>) {
    (self.clone_assigned_key(), self.clone_assigned_number())
  }

  /// Gets the object within as a reference
  pub fn get(&self, key: &Key) -> Option<&AssignedObjects> {
    self.objects_within.get(key)
  }

  /// Gets the object within as a mutable reference
  pub fn get_mut(&mut self, key: &Key) -> Option<&mut AssignedObjects> {
    self.objects_within.get_mut(key)
  }

  /// Returns true if the pixel currently has no assigned_display
  /// Does not include number display, as it should be a given that
  /// no assigned_display implies no assigned_display_number
  pub fn has_no_assignment(&self) -> bool {
    self.assigned_display.is_none()
  }

  /// Gets the latest inserted object and returns it's key
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
