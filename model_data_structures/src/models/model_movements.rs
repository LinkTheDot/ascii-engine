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
