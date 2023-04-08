use ascii_engine::prelude::*;
use std::collections::HashMap;

pub struct CollisionChain {
  model_actions: HashMap<u64, CollisionAction>,
  action_chain_canceled: bool,
}

#[derive(Debug)]
#[allow(unused)] // Seriously why does it count as unread when I'm using this to print shit???
struct CollisionChainDisplayData {
  model_hash: u64,
  model_name: String,
  model_movement: MovementType,
}

pub struct CollisionAction {
  model: ModelData,
  movement: MovementType,
}

#[derive(Clone, Debug)]
pub enum MovementType {
  AbsoluteMovement((usize, usize)),
  RelativeMovement((isize, isize)),
}

impl CollisionChain {
  pub fn new() -> Self {
    Self {
      model_actions: HashMap::new(),
      action_chain_canceled: false,
    }
  }

  pub fn run_action_list(self) {
    if self.action_chain_canceled {
      return;
    }

    let action_list: Vec<CollisionAction> = self.model_actions.into_values().collect();

    action_list.into_iter().for_each(CollisionAction::act);
  }

  pub fn cancel_action_chain(&mut self) {
    self.action_chain_canceled = true;
  }

  pub fn add_action(&mut self, action: CollisionAction) {
    let model_hash = action.model.get_unique_hash();
    self.model_actions.insert(model_hash, action);
  }

  pub fn append(&mut self, other: Self) {
    other.model_actions.into_iter().for_each(|(hash, action)| {
      self.model_actions.insert(hash, action);
    });

    if other.action_chain_canceled {
      self.action_chain_canceled = true
    }
  }

  pub fn change_movement_of(&mut self, key: &u64, new_movement: MovementType) {
    let mut model_action = self.model_actions.get_mut(key).unwrap();

    model_action.movement = new_movement;
  }
}

impl CollisionAction {
  pub fn new(model: ModelData, movement: MovementType) -> Self {
    Self { model, movement }
  }

  fn act(mut self) {
    match self.movement {
      MovementType::AbsoluteMovement(movement) => self.model.absolute_movement(movement),
      MovementType::RelativeMovement(movement) => self.model.relative_movement(movement),
    };
  }
}

impl std::fmt::Display for CollisionChain {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    let mut name_hash_list: Vec<CollisionChainDisplayData> = self
      .model_actions
      .iter()
      .map(|(model_hash, action)| CollisionChainDisplayData {
        model_hash: *model_hash,
        model_name: action.model.get_name(),
        model_movement: action.movement.clone(),
      })
      .collect();

    name_hash_list.reverse();

    write!(f, "{:#?}", name_hash_list)
  }
}
