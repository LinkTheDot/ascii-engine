use crate::screen::pixel::*;

pub trait PixelCheckMethods {
  fn is_empty(&self) -> bool;
  fn contains_object(&self, key: &Key) -> bool;
  fn contains_multiple_of(&self, key: &Key) -> bool;
  fn assigned_key_has_multiple_objects(&self) -> bool;
  fn has_no_assignment(&self) -> bool;
}

impl PixelCheckMethods for Pixel {
  /// Returns true if the pixel contains no object data
  fn is_empty(&self) -> bool {
    self.objects_within.is_empty()
  }

  /// Returns true if the input key/object is within the map
  fn contains_object(&self, key: &Key) -> bool {
    self.objects_within.contains_key(key)
  }

  /// Returns true if the input key has more than 1 object.
  /// Otherwise if there's 1 or 0 of the object, false is returned.
  fn contains_multiple_of(&self, key: &Key) -> bool {
    if let Some(object_inside) = self.objects_within.get(key) {
      object_inside.len() > 1
    } else {
      false
    }
  }

  /// Returns true if the data corresponding to the assigned display key
  /// has more than one object within it
  fn assigned_key_has_multiple_objects(&self) -> bool {
    if let Some(assigned_key) = &self.assigned_display {
      self.get(assigned_key).unwrap().len() > 1
    } else {
      false
    }
  }

  /// Returns true if the pixel currently has no assigned_display
  ///
  /// Does not include number display, as it should be a given that
  /// no assigned_display implies no assigned_display_number.
  fn has_no_assignment(&self) -> bool {
    self.assigned_display.is_none()
  }
}
