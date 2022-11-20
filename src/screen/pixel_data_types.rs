use std::collections::HashMap;
use std::fmt;

pub type Key = String;
pub type AssignedNumber = u32;
pub type ObjectDisplay = String;

pub type AssignedObject = (AssignedNumber, ObjectDisplay);
pub type AssignedObjects = HashMap<AssignedNumber, ObjectDisplay>;
pub type KeyAndObjectDisplay = (Key, AssignedObject);

#[derive(Debug, PartialEq, Eq)]
pub struct ObjectData {
  pub name: String,
  pub number: u32,
  pub display: String,
}

impl fmt::Display for ObjectData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.name)
  }
}
