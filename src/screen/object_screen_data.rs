pub type CurrentAndTotalObjects = (CurrentlyExistingObjects, TotalExistingObjects);
pub type CurrentlyExistingObjects = u32;
pub type TotalExistingObjects = u32;

pub trait ObjectDataMethods {
  fn get_currently_existing(&self) -> CurrentlyExistingObjects;
  fn get_total_existed(&self) -> TotalExistingObjects;
}

#[allow(unused)]
/// This struct will be apart of the screen struct.
/// When a new object is created a new one of these will be made
/// which shall keep track of the amount of times said object
/// appears on the screen, and has existed ever on the screen.
/// There's an extra 'keep_data' part of the struct which will determine
/// whether or not this object's information should be deleted if
/// 'currently_existing' ever reaches 0 after it reaches > 1.
/// False will be the default for this.
/// ( currently isn't fully implemented )
#[derive(Debug)]
pub struct ObjectScreenData {
  name: String,
  keep_data: bool, // determines whether or not data should be kept once currently_existing reaches 0
  currently_existing: CurrentlyExistingObjects,
  total_count: TotalExistingObjects,
}

impl ObjectScreenData {
  pub fn new(name: &str) -> Self {
    ObjectScreenData {
      name: name.to_owned(),
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

  pub fn set_keep_data(&mut self, keep_data: bool) {
    self.keep_data = keep_data
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
