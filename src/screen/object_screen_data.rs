use crate::screen::screen_data::*;
use std::collections::HashMap;

pub type CurrentlyExistingObjects = u32;
pub type TotalExistingObjects = u32;
pub type AssignedObjects = HashMap<u32, ObjectDisplay>;

pub trait ObjectDataMethods {
  fn get_currently_existing(&self) -> CurrentlyExistingObjects;
  fn get_total_existed(&self) -> TotalExistingObjects;
}

#[allow(unused)]
#[derive(Debug)]
pub struct ObjectScreenData {
  name: String,
  keep_data: bool, // determines whether or not data should be kept once currently_existing reaches 0
  currently_existing: CurrentlyExistingObjects,
  total_count: TotalExistingObjects,
}

impl ObjectScreenData {
  pub fn new(name: &String) -> Self {
    ObjectScreenData {
      name: name.clone(),
      keep_data: false,
      currently_existing: 0,
      total_count: 0,
    }
  }

  /// increases the total existing number of objects by 1
  pub fn increment_total(&mut self) {
    self.total_count += 1;
  }

  /// increases the currently present number of objects by 1
  pub fn increment_current(&mut self) {
    self.total_count += 1;
  }

  /// decreases the existing number of objects
  pub fn decrement_current(&mut self) {
    if self.currently_existing > 0 {
      self.currently_existing -= 1;
    }
  }

  pub fn get_currently_existing(&self) -> CurrentlyExistingObjects {
    self.currently_existing
  }

  pub fn get_total_count(&self) -> TotalExistingObjects {
    self.total_count
  }

  pub fn object_still_exists(&self) -> bool {
    todo!()
  }
}

impl ObjectDataMethods for CurrentAndTotalObjects {
  fn get_currently_existing(&self) -> CurrentlyExistingObjects {
    self.0
  }

  fn get_total_existed(&self) -> TotalExistingObjects {
    self.1
  }
}
