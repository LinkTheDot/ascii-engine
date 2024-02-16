use crate::screen::model_storage::*;
use crate::CONFIG;
use engine_math::coordinates::*;
use model_data_structures::models::{
  errors::*, model_appearance::*, model_data::ModelData, model_movements::*,
};
use model_data_structures::prelude::AnimationFrames;
use std::collections::HashSet;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

// TODO: Describe the use of the model_manager
#[derive(Debug)]
pub struct ModelManager {
  model_storage: Arc<RwLock<ModelStorage>>,
  /// Holds a collision, and the timestamp for when that collision occurred.
  ///
  /// Order: push_back -> pop_front
  // TODO: This needs to be connected to every other model manager
  collision_events: Arc<RwLock<VecDeque<(Instant, ModelCollisions)>>>,
}

impl ModelManager {
  pub(crate) fn new(
    model_storage: Arc<RwLock<ModelStorage>>,
    collision_events: Arc<RwLock<VecDeque<(Instant, ModelCollisions)>>>,
  ) -> Self {
    Self {
      model_storage,
      collision_events,
    }
  }

  /// # Errors
  ///
  /// - An error is returned when attempting to add a model that already exists.
  pub fn add_models_to_world(&mut self, list_of_models: Vec<ModelData>) -> Result<(), ModelError> {
    let mut model_storage = self.model_storage.write().unwrap();

    list_of_models
      .into_iter()
      .try_for_each(|model| model_storage.insert(model))
  }

  /// Takes a closure that uses the internal list of models.
  ///
  /// Returns the value resulted within the closure.
  pub fn get_model_list<F, T>(&self, closure: F) -> T
  where
    F: FnOnce(&HashMap<u64, ModelData>) -> T,
  {
    let model_storage_read_guard = self.model_storage.read().unwrap();

    closure(model_storage_read_guard.get_model_list())
  }

  /// Returns a copy of the Model with the given hash.
  ///
  /// None is returned if there was no model in the world with the given hash.
  pub fn get_model(&self, model_hash: &u64) -> Option<ModelData> {
    self.model_storage.read().unwrap().get_model(model_hash)
  }

  /// Returns true if the model of the given hash exists in the world.
  pub fn model_exists(&self, model_hash: &u64) -> bool {
    self.model_storage.read().unwrap().model_exists(model_hash)
  }

  /// Returns the list of model collisions, none if the list was empty.
  ///
  /// # Errors
  ///
  /// - When the movement caused the model to move out of bounds in the negative direction.
  /// - When the passed in model doesn't exist.
  pub fn move_model(
    &mut self,
    model_hash: &u64,
    movement: ModelMovement,
  ) -> Result<Option<ModelCollisions>, ModelError> {
    let Some(mut model) = self.get_model(model_hash) else {
      return Err(ModelError::ModelDoesntExist);
    };

    let Some(new_position) = calculate_movement_of_model(&movement, &model) else {
      return Err(ModelError::ModelOutOfBounds);
    };

    model.change_position(new_position);
    let collision_list = self.check_collisions_against_all_models(model, None);

    if !collision_list.is_empty() {
      let collision = ModelCollisions {
        collider: *model_hash,
        caused_movement: movement,
        collision_list,
      };

      self.add_collision_to_list(collision.clone());

      Ok(Some(collision))
    } else {
      Ok(None)
    }
  }

  // TODO: List the errors.
  pub fn check_if_movement_causes_collisions(
    &self,
    model_hash: &u64,
    movement: ModelMovement,
  ) -> Result<Option<ModelCollisions>, ModelError> {
    let Some(model) = self.get_model(model_hash) else {
      return Err(ModelError::ModelDoesntExist);
    };

    let new_position = calculate_movement_of_model(&movement, &model);
    let collision_list = self.check_collisions_against_all_models(model, new_position);

    if !collision_list.is_empty() {
      Ok(Some(ModelCollisions {
        collider: *model_hash,
        caused_movement: movement,
        collision_list,
      }))
    } else {
      Ok(None)
    }
  }

  /// Returns a list of all models that the passed in model is colliding with.
  ///
  /// Takes an optional new position for the model for simulated collisions if the model was in that new
  /// position.
  fn check_collisions_against_all_models(
    &self,
    moving_model: ModelData,
    // Will be changed to coordinates once the world becomes infinite.
    new_model_position: Option<usize>,
  ) -> VecDeque<u64> {
    let model_id = moving_model.get_hash();

    let mut collision_list = VecDeque::new();

    if self.model_exists(&model_id) {
      self.get_model_list(|model_list| {
        for (hash, model_data) in model_list {
          if hash == &model_id {
            continue;
          }

          if models_are_colliding(&moving_model, new_model_position, model_data) {
            collision_list.push_front(model_data.get_hash());
          }
        }
      });
    }

    collision_list
  }

  /// Queues the animation of with the given name for the model.
  ///
  /// The animation will be run once all other animations added before it have finished running in the queue.
  /// To force an animation to run over all others, use [`model_manager.overwrite_current_model_animation`](ModelManager::overwrite_current_model_animation).
  ///
  /// Takes a boolean that, when true, will duplicate the animation to run in the queue.
  /// If false is passed in, the animation won't be queued if it's already in the queue.
  ///
  /// # Errors
  ///
  /// - There was no model with that hash
  /// - The model in question didn't have an animation with the given name
  /// - The model had no animation data
  pub fn queue_model_animation(
    &mut self,
    model_hash: &u64,
    animation_name: &str,
    duplicate_animation_in_queue: bool,
  ) -> Result<(), ModelError> {
    let model_appearance = self.get_model_appearance(model_hash)?;
    let mut model_appearance = model_appearance.lock().unwrap();

    if !duplicate_animation_in_queue
      && model_appearance
        .animation_is_currently_queued(animation_name)
        .is_some()
    {
      return Ok(());
    }

    model_appearance
      .queue_model_animation(animation_name)
      .map_err(Into::into)
  }

  /// Force stops the currently running animation and starts running the animation
  /// of the given name for the model.
  ///
  /// If the model does not contain an animation with the given name, the currently
  /// running animation will not be stopped.
  ///
  /// # Errors
  ///
  /// - There was no model with that hash
  /// - The model in question didn't have an animation with the given name
  /// - The model had no animation data
  pub fn overwrite_current_model_animation(
    &mut self,
    model_hash: &u64,
    animation_name: &str,
  ) -> Result<(), ModelError> {
    let model_appearance = self.get_model_appearance(model_hash)?;
    let mut model_appearance = model_appearance.lock().unwrap();

    model_appearance
      .overwrite_current_model_animation(animation_name)
      .map_err(Into::into)
  }

  /// Adds the animation to the model's list of stored animations.
  ///
  /// # Errors
  ///
  /// - There was no model with that hash
  /// - The model already contains an animation with the given name
  pub fn add_animation_to_model(
    &mut self,
    model_hash: &u64,
    animation_name: String,
    animation: AnimationFrames,
  ) -> Result<(), ModelError> {
    let model_appearance = self.get_model_appearance(model_hash)?;
    let mut model_appearance = model_appearance.lock().unwrap();

    let _ = model_appearance.add_animation_to_model(animation_name, animation);

    Ok(())
  }

  /// Clears the queue of animations to be run on the model, and stops the currently running animation.
  ///
  ///
  /// # Errors
  ///
  /// - There was no model with that hash
  /// - The model had no animation data
  pub fn clear_model_animation_queue(&mut self, model_hash: &u64) -> Result<(), ModelError> {
    let model_appearance = self.get_model_appearance(model_hash)?;
    let mut model_appearance = model_appearance.lock().unwrap();

    model_appearance.clear_model_animation_queue();

    Ok(())
  }

  pub fn remove_current_model_animation(&mut self, model_hash: &u64) -> Result<(), ModelError> {
    let model_appearance = self.get_model_appearance(model_hash)?;
    let mut model_appearance = model_appearance.lock().unwrap();

    model_appearance.remove_current_model_animation();

    Ok(())
  }

  fn get_model_appearance(
    &mut self,
    model_hash: &u64,
  ) -> Result<Arc<Mutex<ModelAppearance>>, ModelError> {
    Ok(
      self
        .get_model(model_hash)
        .ok_or(ModelError::ModelDoesntExist)?
        .get_appearance_data(),
    )
  }

  /// Returns the keys to every model that exists in the world with any of the given tag(s).
  ///
  /// Takes an option to get every model with exactly the given tags.
  pub fn get_models_with_tags<S: AsRef<str>>(&self, tags: Vec<S>) -> Vec<u64> {
    let model_storage = self.model_storage.read().unwrap();
    let model_list = model_storage.get_model_list();

    model_list
      .iter()
      .filter(|(_, model)| model.contains_tags(&tags))
      .map(|(hash, _)| *hash)
      .collect()
  }

  /// Returns the list of tags tied to the given model.
  ///
  /// None is returned if the model didn't exist.
  pub fn get_tags_of_model(&self, model_hash: u64) -> Option<HashSet<String>> {
    let model = self.get_model(&model_hash)?;

    Some(model.get_tags())
  }

  /// Drains the collisions that've occurred since the last time this method was called.
  pub fn take_collision_events(&mut self) -> VecDeque<(Instant, ModelCollisions)> {
    std::mem::take(&mut self.collision_events.write().unwrap())
  }

  /// Takes a copy of the current collisions that've occurred since the last time the list was drained.
  pub fn clone_collision_events(&self) -> VecDeque<(Instant, ModelCollisions)> {
    self.collision_events.read().unwrap().clone()
  }

  /// Checks the list of collisions to see if the passed in model has collided with anything.
  ///
  /// Returns the list of collisions and their timestamps if they existed, None otherwise.
  /// This method does *not* drain any collisions from the list, rather it clones every collision event
  /// and its respective timestamp.
  pub fn model_has_collided(&self, model: &u64) -> Option<VecDeque<(Instant, ModelCollisions)>> {
    let collision_list: VecDeque<(Instant, ModelCollisions)> = self
      .collision_events
      .read()
      .unwrap()
      .iter()
      .filter_map(|(timestamp, collision)| {
        if &collision.collider == model || collision.collision_list.contains(model) {
          Some((*timestamp, collision.clone()))
        } else {
          None
        }
      })
      .collect();

    (!collision_list.is_empty()).then_some(collision_list)
  }

  /// Adds a collision to the back to the list and creates an Instant of the current time.
  fn add_collision_to_list(&mut self, collision: ModelCollisions) {
    self
      .collision_events
      .write()
      .unwrap()
      .push_back((Instant::now(), collision))
  }
}

fn models_are_colliding(
  model_one: &ModelData,
  new_model_one_position: Option<usize>,
  model_two: &ModelData,
) -> bool {
  if model_one.hitbox_is_empty() || model_two.hitbox_is_empty() {
    return false;
  }

  let model_one_index = match new_model_one_position {
    Some(index) => index,
    None => model_one.get_frame_position(),
  };

  let model_one_hitbox_position = add_index_to_coordinates(
    model_one.sprite_to_hitbox_anchor_difference(),
    model_one_index,
  );
  let model_two_hitbox_position = add_index_to_coordinates(
    model_two.sprite_to_hitbox_anchor_difference(),
    model_two.get_frame_position(),
  );

  let model_one_hitbox_dimensions = model_one.get_hitbox_dimensions();
  let model_two_hitbox_dimensions = model_two.get_hitbox_dimensions();

  model_one_hitbox_dimensions.is_colliding(
    model_one_hitbox_position,
    &model_two_hitbox_dimensions,
    model_two_hitbox_position,
  )
}

fn calculate_relative_movement_frame_position(
  model: &ModelData,
  added_position: &(isize, isize),
) -> Option<usize> {
  let screen_width = CONFIG.grid_width as isize + 1;
  let model_frame_top_left = model.get_frame_position() as isize;

  let new_position = added_position.0 + (screen_width * added_position.1) + model_frame_top_left;

  if new_position >= 0 {
    Some(new_position as usize)
  } else {
    None
  }
}

fn add_index_to_coordinates(coordinates: (isize, isize), index: usize) -> (isize, isize) {
  let (x, y) = index.index_to_coordinates(CONFIG.grid_width as usize + 1);

  (x as isize + coordinates.0, y as isize + coordinates.1)
}

/// Returns the new frame position of the model based on the movement.
///
/// None is returned if the movement caused the top left of the model to go negative.
fn calculate_movement_of_model(movement: &ModelMovement, model: &ModelData) -> Option<usize> {
  match movement {
    ModelMovement::Absolute(movement) => {
      // This conversion will be removed once the world becomes infinite and cameras exist.
      let movement = (movement.0 as usize, movement.1 as usize);

      model.calculate_top_left_index_from(movement)
    }

    ModelMovement::Relative(movement) => {
      calculate_relative_movement_frame_position(model, movement)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use model_data_structures::models::testing_data::*;

  #[test]
  fn calculate_relative_movement_frame_position_out_of_bounds() {
    let model = TestingData::new_test_model((10, 10));
    let added_position = (-10, -10); // 0, 0 puts top left at -2. -1.

    let result = calculate_relative_movement_frame_position(&model, &added_position);

    assert!(result.is_none());
  }
}
