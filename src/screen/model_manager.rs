use crate::screen::model_storage::*;
use crate::CONFIG;
use engine_math::coordinates::*;
use model_data_structures::models::{
  animation::*, errors::*, model_data::ModelData, model_movements::*,
};
use model_data_structures::prelude::AnimationFrames;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

// Describe the use of the model_manager
#[derive(Debug)]
pub struct ModelManager {
  model_storage: Arc<RwLock<ModelStorage>>,
  animation_thread_sender: Option<mpsc::UnboundedSender<AnimationRequest>>,
}

impl ModelManager {
  pub(crate) fn new(model_storage: Arc<RwLock<ModelStorage>>) -> Self {
    Self {
      model_storage,
      animation_thread_sender: None,
    }
  }

  pub(crate) fn add_animation_connection(
    &mut self,
    connection: mpsc::UnboundedSender<AnimationRequest>,
  ) {
    self.animation_thread_sender = Some(connection)
  }

  /// Takes a closure that uses the internal list of models.
  ///
  /// Returns the value resulted within the closure.
  pub fn get_model_list<F, T>(&self, closure: F) -> T
  where
    F: FnOnce(&HashMap<u64, ModelData>) -> T,
  {
    let read_guard = self.model_storage.read().unwrap();

    closure(read_guard.get_model_list())
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

  pub fn move_model(
    &mut self,
    model_hash: u64,
    movement: ModelMovement,
  ) -> Option<ModelCollisions> {
    let mut model = self.get_model(&model_hash)?;

    let new_position = match movement {
      ModelMovement::Absolute(movement) => {
        // This conversion will be removed once the world becomes infinite and cameras exist.
        let movement = (movement.0 as usize, movement.1 as usize);

        calculate_absolute_movement_frame_position(&model, movement)
      }
      ModelMovement::Relative(movement) => {
        calculate_relative_movement_frame_position(&model, movement)
      }
    };

    model.change_position(new_position);
    let collision_list = self.check_collisions_against_all_models(model, None);

    if !collision_list.is_empty() {
      Some(ModelCollisions {
        collider: model_hash,
        caused_movement: movement,
        collision_list,
      })
    } else {
      None
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
  /// # Errors
  ///
  /// - There was no model with that hash
  /// - The model in question didn't have an animation with the given name
  /// - The model had no animation data
  pub fn queue_model_animation(
    &mut self,
    _model: u64,
    _animation_name: String,
  ) -> Result<(), ModelError> {
    todo!()
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
    _model: u64,
    _animation_name: String,
  ) -> Result<(), ModelError> {
    todo!()
  }

  /// Clears the queue of animations to be run on the model, and stops the currently running animation.
  ///
  ///
  /// # Errors
  ///
  /// - There was no model with that hash
  /// - The model had no animation data
  pub fn clear_model_animation_queue(&mut self, _model: u64) -> Result<(), ModelError> {
    todo!()
  }

  /// Adds the animation to the model's list of stored animations.
  ///
  /// # Errors
  ///
  /// - There was no model with that hash
  /// - The model already contains an animation with the given name
  pub fn add_animation_to_model(
    &mut self,
    _model: u64,
    _animation: AnimationFrames,
    _animation_name: String,
  ) -> Result<(), ModelError> {
    todo!()
  }

  /// Adds a model to the animation thread to run it's animations.
  // Explain how to animate a model.
  ///
  /// # Errors
  ///
  /// - The animation thread isn't started.
  /// - There was no model with that hash
  /// - The model had no animation data
  pub fn add_model_to_animation_thread(&mut self, model_hash: u64) -> Result<(), ModelError> {
    let Some(animation_thread_sender) = &self.animation_thread_sender else {
      return Err(ModelError::AnimationError(
        AnimationError::AnimationThreadNotStarted,
      ));
    };

    let Some(mut model_data) = self.get_model(&model_hash) else {
      return Err(ModelError::ModelDoesntExist);
    };
    let Some(model_animation_data) = model_data.get_animation_data() else {
      return Err(ModelError::AnimationError(AnimationError::ModelHasNoAnimationData));
    };
    let mut model_animation_data = model_animation_data.lock().unwrap();

    model_animation_data.send_model_animator_request(model_hash, animation_thread_sender);

    Ok(())
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

fn calculate_absolute_movement_frame_position(
  model: &ModelData,
  new_position: (usize, usize),
) -> usize {
  let sprite_anchor_coordinates = model
    .get_sprite()
    .read()
    .unwrap()
    .get_anchor_index()
    .index_to_coordinates(CONFIG.grid_width as usize + 1)
    .to_isize();

  let anchored_placement = new_position.0 + ((CONFIG.grid_width as usize + 1) * new_position.1);
  let top_left_difference = (sprite_anchor_coordinates.0
    + (sprite_anchor_coordinates.1 * (CONFIG.grid_width as isize + 1)))
    as usize;

  anchored_placement - top_left_difference
}

fn calculate_relative_movement_frame_position(
  model: &ModelData,
  added_position: (isize, isize),
) -> usize {
  let true_screen_width = CONFIG.grid_width as isize + 1;
  let model_frame_top_left = model.get_frame_position() as isize;

  (added_position.0 + (true_screen_width * added_position.1) + model_frame_top_left) as usize
}

fn add_index_to_coordinates(coordinates: (isize, isize), index: usize) -> (isize, isize) {
  let (x, y) = index.index_to_coordinates(CONFIG.grid_width as usize + 1);

  (x as isize + coordinates.0, y as isize + coordinates.1)
}
