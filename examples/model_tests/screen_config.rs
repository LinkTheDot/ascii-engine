// use crate::{ConnectedTeleportPads, Player, TeleportPad, Wall};
// use ascii_engine::prelude::*;
// use std::collections::HashMap;
// use std::sync::{Arc, Mutex};
//
// /// Contains the ScreenData and list of models that exist.
// pub struct ScreenConfig {
//   pub screen: Arc<Mutex<ScreenData>>,
//   models: ModelTypes,
// }
//
// /// Contains different fields for the different types of models that exist.
// struct ModelTypes {
//   square_list: HashMap<u64, Player>,
//   wall_list: HashMap<u64, Wall>,
//   teleport_pads: HashMap<u64, TeleportPad>,
// }
//
// impl ScreenConfig {
//   pub fn new(screen: ScreenData) -> Self {
//     Self {
//       screen: Arc::new(Mutex::new(screen)),
//       models: ModelTypes::new(),
//     }
//   }
//
//   /// Adds the object internally and returns it's unique hash.
//   ///
//   /// # Errors
//   ///
//   /// - Returns an error when attempting to add a model that already exists.
//   pub fn add_square(&mut self, square: Player) -> Result<u64, ModelError> {
//     let square_hash = square.get_unique_hash();
//
//     self.screen.lock().unwrap().add_model(&square)?;
//
//     self.models.square_list.insert(square_hash, square);
//
//     Ok(square_hash)
//   }
//
//   /// Removes any mention of the square pertaining to the given hash.
//   ///
//   /// Returns the removed square, if it didn't exist returns None.
//   #[allow(dead_code)]
//   pub fn remove_square(&mut self, key: &u64) -> Option<Player> {
//     let square = self.models.square_list.remove(key)?;
//     let mut screen_lock = self.screen.lock().unwrap();
//
//     screen_lock.remove_model(key)?;
//
//     Some(square)
//   }
//
//   /// Gets a mutable reference to the square of the given unique hash.
//   ///
//   /// Returns None if the model didn't exist.
//   #[allow(dead_code)]
//   pub fn get_mut_square(&mut self, key: &u64) -> Option<&mut Player> {
//     self.models.square_list.get_mut(key)
//   }
//
//   /// Gets a reference to the square of the given unique hash.
//   ///
//   /// Returns None if the model didn't exist.
//   pub fn get_square(&self, key: &u64) -> Option<&Player> {
//     self.models.square_list.get(key)
//   }
//
//   /// Adds the object internally and returns it's unique hash.
//   ///
//   /// # Errors
//   ///
//   /// - Returns an error is returned when attempting to add a model that already exists.
//   pub fn add_wall(&mut self, wall: Wall) -> Result<u64, ModelError> {
//     let wall_hash = wall.get_unique_hash();
//
//     self.screen.lock().unwrap().add_model(&wall)?;
//
//     self.models.wall_list.insert(wall_hash, wall);
//
//     Ok(wall_hash)
//   }
//
//   /// Gets a mutable reference to the wall of the given unique hash.
//   ///
//   /// Returns None if the model didn't exist.
//   #[allow(dead_code)]
//   pub fn get_mut_wall(&mut self, key: &u64) -> Option<&mut Wall> {
//     self.models.wall_list.get_mut(key)
//   }
//
//   /// Gets a reference to the wall of the given unique hash.
//   ///
//   /// Returns None if the model didn't exist.
//   #[allow(dead_code)]
//   pub fn get_wall(&self, key: &u64) -> Option<&Wall> {
//     self.models.wall_list.get(key)
//   }
//
//   /// Adds both teleporters internally and returns a tuple of (pad1_hash, pad2_hash).
//   ///
//   /// # Errors
//   ///
//   /// - Returns an error when attempting to add a model that already exists.
//   /// - Returns an error if the two given pads are not connected to eachother.
//   pub fn add_teleport_pads(
//     &mut self,
//     pad_1: TeleportPad,
//     pad_2: TeleportPad,
//   ) -> Result<(u64, u64), ModelError> {
//     if pad_1.is_connected_to(&pad_2) {
//       let mut screen_lock = self.screen.lock().unwrap();
//
//       screen_lock.add_model(&pad_1)?;
//       screen_lock.add_model(&pad_2)?;
//       drop(screen_lock);
//
//       let pad_1_hash = pad_1.get_unique_hash();
//       let pad_2_hash = pad_2.get_unique_hash();
//
//       self.models.teleport_pads.insert(pad_1_hash, pad_1);
//       self.models.teleport_pads.insert(pad_2_hash, pad_2);
//
//       Ok((pad_1_hash, pad_2_hash))
//     } else {
//       Err(ModelError::Other(
//         "Attempted to add teleport pads that weren't connected".to_string(),
//       ))
//     }
//   }
//
//   /// Gets a reference to the teleport pad of the given unique hash.
//   ///
//   /// Returns None if the model didn't exist.
//   pub fn get_teleport_pad(&self, key: &u64) -> Option<&TeleportPad> {
//     self.models.teleport_pads.get(key)
//   }
//
//   /// Gets a mutable reference to the teleport pad of the given unique hash.
//   ///
//   /// Returns None if the model didn't exist.
//   #[allow(dead_code)]
//   pub fn get_mut_teleport_pad(&mut self, key: &u64) -> Option<&mut TeleportPad> {
//     self.models.teleport_pads.get_mut(key)
//   }
//
//   pub fn get_connected_teleport_pads(&self, key: &u64) -> Option<ConnectedTeleportPads> {
//     let teleport_pad = self.get_teleport_pad(key)?;
//
//     let other_teleport_pad_hash = teleport_pad.get_connected_pad_hash();
//     let other_teleport_pad = self.get_teleport_pad(&other_teleport_pad_hash)?;
//
//     Some(ConnectedTeleportPads {
//       teleport_pad_1: teleport_pad,
//       teleport_pad_2: other_teleport_pad,
//     })
//   }
// }
//
// impl ModelTypes {
//   fn new() -> Self {
//     Self {
//       square_list: HashMap::new(),
//       wall_list: HashMap::new(),
//       teleport_pads: HashMap::new(),
//     }
//   }
// }
