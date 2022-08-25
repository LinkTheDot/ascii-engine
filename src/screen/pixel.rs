use crate::general_data::map_methods::*;
use crate::screen::screen_data::*;
use std::collections::{BTreeMap, HashMap};

pub type AssignedNumber = u32;
pub type AssignedObject = (AssignedNumber, ObjectDisplay);
pub type AssignedObjects = HashMap<AssignedNumber, ObjectDisplay>;

#[derive(Clone, Debug, PartialEq)]
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
    if let (Some(display_key), Some(assigned_display)) =
      (&self.assigned_display, &self.assigned_display_number)
    {
      return self
        .objects_within
        .get(display_key.as_str())
        .unwrap()
        .get(&assigned_display)
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

  // make it change the number as well
  // if one isn't specified then search for the latest
  // number key and use that one
  pub fn change_display_to(&mut self, change_to: Key, assigned_number: Option<AssignedNumber>) {
    if assigned_number.is_some() {
      self.assigned_display_number = assigned_number;
      self.assigned_display = Some(change_to);
    } else if self.objects_within.contains_key(&change_to) {
      let new_number = self
        .objects_within
        .get(&change_to)
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .clone();

      self.assigned_display_number = Some(new_number);
      self.assigned_display = Some(change_to);
    }
  }

  /// Inserts the object and if there's no assignment
  /// it'll assign the inserted object to the pixel
  pub fn insert_object(&mut self, key: Key, item: AssignedObject) {
    if self.objects_within.contains_key(&key) {
      self
        .objects_within
        .get_mut(&key)
        .unwrap()
        .insert(item.0, item.1)
        .unwrap_or_else(|| "".to_string());
    } else {
      self.change_display_to(key.clone(), Some(item.0));

      let mut new_map = HashMap::new();
      new_map.insert(item.0, item.1);

      self.objects_within.insert(key.clone(), new_map);
    }
  }

  /// Removes the data that's currently assigned to display and returns it
  pub fn remove_displayed_object(&mut self) -> Option<KeyAndObjectDisplay> {
    if !self.is_empty() {
      if self.assigned_key_has_multiple_objects() {
        let removed_object_display = self.remove_object_assigned_number().unwrap();
        let copy_of_assinged_key = self.assigned_display.as_ref().unwrap().clone();

        Some((copy_of_assinged_key, removed_object_display))
      } else {
        let mut removed_object_display = self
          .objects_within
          .remove_entry(self.assigned_display.as_ref().unwrap())
          .unwrap();

        self.assigned_display = self.check_if_available_object().cloned();

        Some((
          removed_object_display.0,
          removed_object_display
            .1
            .remove_entry(self.get_assigned_number().unwrap())
            .unwrap(),
        ))
      }
    } else {
      None
    }
  }

  /// Only removes the object, doesn't clear it from memory
  /// if there're 0 objects left inside
  pub fn remove_object_assigned_number(&mut self) -> Option<AssignedObject> {
    if let (Some(object_key), Some(assigned_number)) = (
      self.assigned_display.as_ref(),
      self.assigned_display_number.as_ref(),
    ) {
      let assigned_object = self
        .objects_within
        .get_mut(object_key)
        .unwrap()
        .remove_entry(&assigned_number);

      if self.objects_within.get(object_key).unwrap().len() > 1 {
        return assigned_object;
      }
    }

    None
  }

  pub fn get_current_display_data(&self) -> Option<&AssignedObjects> {
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

  pub fn get_assigned_key(&self) -> Option<&Key> {
    self.assigned_display.as_ref()
  }

  pub fn get_assigned_number(&self) -> Option<&AssignedNumber> {
    self.assigned_display_number.as_ref()
  }
}
