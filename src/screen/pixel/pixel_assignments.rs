use crate::screen::pixel::*;

pub trait PixelAssignmentMethods {
  fn get_assigned_key(&self) -> Option<&Key>;
  fn get_assigned_number(&self) -> Option<&AssignedNumber>;
  fn get_both_assignments(&self) -> (Option<&Key>, Option<&AssignedNumber>);

  fn clone_assigned_key(&self) -> Option<Key>;
  fn clone_assigned_number(&self) -> Option<AssignedNumber>;
  fn clone_both_assignments(&self) -> (Option<Key>, Option<AssignedNumber>);

  fn take_assigned_key(&mut self) -> Option<Key>;
  fn take_assigned_number(&mut self) -> Option<AssignedNumber>;
  fn take_both_assignments(&mut self) -> (Option<Key>, Option<AssignedNumber>);
}

impl PixelAssignmentMethods for Pixel {
  /// Gets a reference to the current assigned object key
  fn get_assigned_key(&self) -> Option<&Key> {
    self.assigned_display.as_ref()
  }

  /// Gets a reference to the current assigned_object_number key
  fn get_assigned_number(&self) -> Option<&AssignedNumber> {
    self.assigned_display_number.as_ref()
  }

  /// Gets a reference to both the current assigned object and object number
  fn get_both_assignments(&self) -> (Option<&Key>, Option<&AssignedNumber>) {
    (self.get_assigned_key(), self.get_assigned_number())
  }

  /// Gets a copy of the current assigned object key
  fn clone_assigned_key(&self) -> Option<Key> {
    self.assigned_display.clone()
  }

  /// Gets a copy of the current assigned_object_number key
  fn clone_assigned_number(&self) -> Option<AssignedNumber> {
    // add .clone() once objects use unique hashes
    self.assigned_display_number
  }

  /// Gets a copy of both the current assigned object and object number
  fn clone_both_assignments(&self) -> (Option<Key>, Option<AssignedNumber>) {
    (self.clone_assigned_key(), self.clone_assigned_number())
  }

  /// Takes the assigned key leaving None in it's place
  fn take_assigned_key(&mut self) -> Option<Key> {
    self.assigned_display.take()
  }

  /// Gets a copy of the current assigned_object_number key
  fn take_assigned_number(&mut self) -> Option<AssignedNumber> {
    self.assigned_display_number.take()
  }

  /// Gets a copy of both the current assigned object and object number
  fn take_both_assignments(&mut self) -> (Option<Key>, Option<AssignedNumber>) {
    (self.take_assigned_key(), self.take_assigned_number())
  }
}
