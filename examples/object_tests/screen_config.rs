use crate::{Square, Wall};
use ascii_engine::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct ScreenConfig {
  pub screen: ScreenData,
  objects: ObjectTypes,
}

struct ObjectTypes {
  square: HashMap<u64, Arc<RwLock<Square>>>,
  wall: HashMap<u64, Arc<RwLock<Wall>>>,
}

impl ScreenConfig {
  pub fn new(screen: ScreenData) -> Self {
    Self {
      screen,
      objects: ObjectTypes::new(),
    }
  }

  pub fn add_square(&mut self, mut square: Square) -> Result<Arc<RwLock<Square>>, ObjectError> {
    let square_hash = square.get_unique_hash();
    self.screen.add_object(&mut square)?;

    let square = square.wrap_self();
    self.objects.square.insert(square_hash, square.clone());

    Ok(square)
  }

  pub fn get_square(&self, key: &u64) -> Arc<RwLock<Square>> {
    match self.objects.square.get(key) {
      Some(square) => square.clone(),
      None => panic!("No object by the name of {key}"),
    }
  }

  pub fn add_wall(&mut self, mut wall: Wall) -> Result<Arc<RwLock<Wall>>, ObjectError> {
    let wall_hash = wall.get_unique_hash();
    self.screen.add_object(&mut wall)?;

    let wrapped_wall = wall.wrap_self();
    self.objects.wall.insert(wall_hash, wrapped_wall.clone());

    Ok(wrapped_wall)
  }

  pub fn get_wall(&self, key: &u64) -> Arc<RwLock<Wall>> {
    self.objects.wall.get(key).unwrap().clone()
  }
}

impl ObjectTypes {
  fn new() -> Self {
    Self {
      square: HashMap::new(),
      wall: HashMap::new(),
    }
  }
}
