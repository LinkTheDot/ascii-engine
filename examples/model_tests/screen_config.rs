use crate::{Square, TeleportPad, Wall};
use ascii_engine::prelude::*;
use std::collections::HashMap;

#[allow(unused)]
use log::debug;

pub struct ScreenConfig {
  pub screen: ScreenData,
  models: ModelTypes,
}

struct ModelTypes {
  square_list: HashMap<u64, Square>,
  wall_list: HashMap<u64, Wall>,
  teleport_pads: HashMap<u64, TeleportPad>,
}

impl ScreenConfig {
  pub fn new(screen: ScreenData) -> Self {
    Self {
      screen,
      models: ModelTypes::new(),
    }
  }

  /// Adds the object internally and returns it's unique hash.
  pub fn add_square(&mut self, square: Square) -> Result<u64, ModelError> {
    let square_hash = square.get_unique_hash();
    self.screen.add_model(&square)?;

    self.models.square_list.insert(square_hash, square);

    Ok(square_hash)
  }

  #[allow(dead_code)]
  pub fn get_mut_square(&mut self, key: &u64) -> Option<&mut Square> {
    self.models.square_list.get_mut(key)
  }

  pub fn get_square(&self, key: &u64) -> Option<&Square> {
    self.models.square_list.get(key)
  }

  pub fn add_wall(&mut self, wall: Wall) -> Result<u64, ModelError> {
    let wall_hash = wall.get_unique_hash();
    self.screen.add_model(&wall)?;

    self.models.wall_list.insert(wall_hash, wall);

    Ok(wall_hash)
  }

  #[allow(dead_code)]
  pub fn get_mut_wall(&mut self, key: &u64) -> Option<&mut Wall> {
    self.models.wall_list.get_mut(key)
  }

  #[allow(dead_code)]
  pub fn get_wall(&self, key: &u64) -> Option<&Wall> {
    self.models.wall_list.get(key)
  }

  pub fn add_teleport_pads(
    &mut self,
    pad_1: TeleportPad,
    pad_2: TeleportPad,
  ) -> Result<(u64, u64), ModelError> {
    if pad_1.is_connected_to(&pad_2) {
      self.screen.add_model(&pad_1)?;
      self.screen.add_model(&pad_2)?;

      let pad_1_hash = pad_1.get_unique_hash();
      let pad_2_hash = pad_2.get_unique_hash();

      self.models.teleport_pads.insert(pad_1_hash, pad_1);
      self.models.teleport_pads.insert(pad_2_hash, pad_2);

      Ok((pad_1_hash, pad_2_hash))
    } else {
      // maybe make an error called "ModelError::Other("What went wrong")"
      Err(ModelError::Other(
        "Attempted to add teleport pads that weren't connected".to_string(),
      ))
    }
  }

  pub fn get_teleport_pad(&self, key: &u64) -> Option<&TeleportPad> {
    self.models.teleport_pads.get(key)
  }

  #[allow(dead_code)]
  pub fn get_mut_teleport_pad(&mut self, key: &u64) -> Option<&mut TeleportPad> {
    self.models.teleport_pads.get_mut(key)
  }
}

impl ModelTypes {
  fn new() -> Self {
    Self {
      square_list: HashMap::new(),
      wall_list: HashMap::new(),
      teleport_pads: HashMap::new(),
    }
  }
}
