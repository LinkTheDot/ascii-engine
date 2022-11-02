// add a way to remove specific objects
// add a way to remove all objects

use crate::general_data::map_methods::*;
use crate::screen::screen_data::*;
use std::collections::{BTreeMap, HashMap};

pub type AssignedNumber = u32;
pub type AssignedObject = (AssignedNumber, ObjectDisplay);
pub type AssignedObjects = HashMap<AssignedNumber, ObjectDisplay>;

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
  assigned_display: Option<Key>,
  assigned_display_number: Option<AssignedNumber>,
  objects_within: BTreeMap<Key, AssignedObjects>,
}

impl Pixel {
  pub fn new() -> Self {
    Pixel {
      assigned_display: None,
      assigned_display_number: None,
      objects_within: BTreeMap::new(),
    }
  }

  /// Returns the pixel display depending on what was assigned
  /// if nothing was assigned then it'll return what an EMPTY_PIXEL is defined as
  pub fn display(&self) -> String {
    if let Some((display_key, assigned_display)) = &self.get_both_assignments() {
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
  /// If the input is invalid it'll set the display to None
  pub fn change_display_to(
    &mut self,
    change_to: Option<Key>,
    assigned_number: Option<AssignedNumber>,
  ) {
    if change_to.is_some() && assigned_number.is_some() {
      if self.contains_object(change_to.as_ref().unwrap()) {
        self.change_display(change_to, assigned_number);
      } else {
        self.change_display(None, None);
      }
    } else if change_to.is_some() && assigned_number.is_none() {
      let key = change_to.unwrap();

      let lowest_display_number = *self.get(&key).unwrap().get_lowest_key().unwrap();

      self.change_display(Some(key), Some(lowest_display_number));
    } else {
      self.change_display(None, None);
    }
  }

  /// Inserts the object and if reassign is true will assign the new data
  /// as the given pixel's new display assignment
  pub fn insert_object(&mut self, key: Key, item: AssignedObject, reassign: bool) {
    if self.objects_within.contains_key(&key) {
      self
        .objects_within
        .get_mut(&key)
        .unwrap()
        .insert(item.0, item.1)
        .unwrap_or_else(|| "".to_string());
    } else {
      let mut new_map = HashMap::new();
      new_map.insert(item.0, item.1);

      self.objects_within.insert(key.clone(), new_map);
    }

    if reassign {
      self.change_display_to(Some(key.clone()), Some(item.0));
    }
  }

  /// Removes the data that's currently assigned to display and returns it
  /// Deletes the entry if there's only 1 object in there
  pub fn remove_displayed_object(&mut self) -> Option<KeyAndObjectDisplay> {
    let pixel_data = if !self.is_empty() && !self.has_no_assignment() {
      if self.assigned_key_has_multiple_objects() {
        let key = self.get_assigned_key().unwrap().clone();
        let number_and_display = self.remove_object_assigned_number(true).unwrap();

        Some((key, number_and_display))
      } else {
        let mut object = self
          .objects_within
          .remove_entry(self.assigned_display.as_ref().unwrap().as_str())
          .unwrap();

        let key = object.0;
        let assigned_object = object.1.drain().next().unwrap();

        self.change_display(None, None);

        Some((key, assigned_object))
      }
    } else {
      None
    };

    self.reassign_display_data();

    pixel_data
  }

  /// Removes the object pertaining to the assigned number
  /// if there's more than one it'll just remove the one
  /// if there's only 1 it'll remove the map inside along with the item
  pub fn remove_object_assigned_number(&mut self, reassign: bool) -> Option<AssignedObject> {
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

    if removed_data.is_some() && reassign {
      self.change_display_to(None, None);

      self.reassign_display_data();
    }

    removed_data
  }

  /// Gets a reference to the display item currently inside the pixel
  pub fn get_current_display_data(&self) -> Option<&ObjectDisplay> {
    if let Some((assigned_key, assigned_number)) = &self.get_both_assignments() {
      if self.contains_object(assigned_key) {
        self.get(*assigned_key).unwrap().get(assigned_number)
      } else {
        None
      }
    } else {
      None
    }
  }

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
  /// has more than 1 object within it
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

  /// Gets a reference to both the current assigend object and object number
  pub fn get_both_assignments(&self) -> Option<(&Key, &AssignedNumber)> {
    if !self.has_no_assignment() {
      Some((
        self.get_assigned_key().as_ref().unwrap(),
        self.get_assigned_number().as_ref().unwrap(),
      ))
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
      self.change_display_to(Some(key), Some(assigned_number));
    } else {
      self.change_display_to(None, None);
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
