use crate::general_data::{coordinates::*, hasher};
use crate::objects::sprites::*;

/// This is the data that will be required for the Object derive macro.
///
/// ObjectData contains data such as, the object's unique hash, the position of the
/// defined center point, the strata, and the Sprite.
#[allow(unused)]
#[derive(Debug)]
pub struct ObjectData {
  unique_hash: u64,
  /// Based on where the center is.
  object_position: usize,
  strata: Strata,
  sprite: Sprite,
}

/// The Strata will the the priority on the screen.
/// That which has a higher Strata, will be above those with lower strata.
///
/// A Strata will contain an integer for anything with same Stratas.
/// An object with a lower number has a higher priority to show up on top.
///
/// If multiple objects have same Strata and Strata Numbers, the unique hashes will
/// be used to determine the one that stays on top.
#[derive(Debug, PartialEq, Eq)]
pub enum Strata {
  Top(u16),
  High(u16),
  Medium(u16),
  Low(u16),
  Background(u16),
}

impl ObjectData {
  pub fn new(object_position: Coordinates, sprite: Sprite, strata: Strata) -> Self {
    let unique_hash = hasher::get_unique_hash();

    Self {
      unique_hash,
      object_position: object_position.coordinates_to_index(),
      strata,
      sprite,
    }
  }

  pub fn get_unique_hash(&self) -> &u64 {
    &self.unique_hash
  }

  pub fn get_object_position(&self) -> &usize {
    &self.object_position
  }

  pub fn get_sprite(&self) -> &str {
    self.sprite.get_shape()
  }

  pub fn change_sprite(&mut self, new_model: String) {
    *self.sprite.get_mut_shape() = new_model;
  }

  pub fn get_hitbox(&self) -> &Vec<(isize, isize)> {
    self.sprite.get_hitbox()
  }

  pub fn change_hitbox(&mut self, new_hitbox: Hitbox) {
    self.sprite.change_hitbox(new_hitbox)
  }

  pub fn get_strata(&self) -> &Strata {
    &self.strata
  }

  pub fn change_strata(&mut self, new_strata: Strata) {
    self.strata = new_strata
  }
}
