use std::collections::HashMap;

pub type AssignedNumber = u32;
pub type AssignedObject = (AssignedNumber, ObjectDisplay);
pub type AssignedObjects = HashMap<AssignedNumber, ObjectDisplay>;
pub type Key = String;

pub type ObjectDisplay = String;
pub type KeyAndObjectDisplay = (Key, AssignedObject);
pub type CurrentAndTotalObjects = (CurrentlyExistingObjects, TotalExistingObjects);

pub type CurrentlyExistingObjects = u32;
pub type TotalExistingObjects = u32;

#[derive(Debug, PartialEq)]
struct ObjectData {
  assigned_name: String,
  assigned_number: u32,
  assigned_display: String,
}
