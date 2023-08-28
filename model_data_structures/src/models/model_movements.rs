use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct ModelCollisions {
  pub collider: u64,
  pub caused_movement: ModelMovement,
  pub collision_list: VecDeque<u64>,
}

#[derive(Debug, Clone, Copy)]
pub enum ModelMovement {
  Absolute((isize, isize)),
  Relative((isize, isize)),
}

// impl ModelCollisions {
//   /// Returns a reference to the list of model hashes that the collider collided with.
//   pub fn get_collision_list(&self) -> &VecDeque<u64> {
//     &self.collision_list
//   }
//
//   /// Returns the movement that the initial collider did to cause the collisions.
//   pub fn get_collider_movement(&self) -> ModelMovement {
//     self.caused_movement
//   }
//
//   /// Returns the ID of the model that moved, causing the collisions.
//   pub fn get_initial_collider_id(&self) -> u64 {
//     self.collider
//   }
// }
