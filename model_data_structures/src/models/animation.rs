// TODO
// Add a way to freeze the animation queue and pausing the stored EventSync.
// Add a way to default to the resting frame of an animation, while pausing the stored EventSync.
// Add a way to assign a default animation to fall back on when there are none running in the queue.

use crate::models::model_appearance::sprites::Sprite;
pub use animation_frames::*;
pub use errors::*;
pub use model_animator::*;
use serde::Serializer;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;

pub mod animation_connections;
pub mod animation_frames;
pub mod animation_frames_iterators;
pub mod errors;
pub mod model_animator;

/// Stores the list of existing model animations, and the data for the model's
/// current animation state.
#[derive(Default, Clone, Deserialize, Serialize)]
pub struct ModelAnimationData {
  animations: HashMap<String, AnimationFrames>,
  #[serde(serialize_with = "remove_running_animations")]
  model_animator: RefCell<ModelAnimator>,
}

/// Reset the ModelAnimator when serializing.
fn remove_running_animations<S>(
  _: &RefCell<ModelAnimator>,
  serializer: S,
) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  ModelAnimator::default().serialize(serializer)
}

impl ModelAnimationData {
  pub fn new<I>(animation_list: I) -> Self
  where
    I: IntoIterator<Item = (String, AnimationFrames)>,
  {
    Self::from(animation_list)
  }

  /// Returns the current appearance of the model based on running animations.
  pub fn get_current_appearance(&self) -> Option<&Sprite> {
    self
      .model_animator
      .borrow_mut()
      .get_current_model_appearance(&self.animations)
  }

  /// Returns true if there's an animation with the given name.
  pub fn contains_animation(&self, animation_name: &str) -> bool {
    self.animations.contains_key(animation_name)
  }

  /// Removes the animation from the list of animations, and returns it if it existed.
  /// This will also stop the animation from running in the queue.
  pub fn remove_animation_from_list(&mut self, animation_name: &str) -> Option<AnimationFrames> {
    let (_, animation_frames) = self.animations.remove_entry(animation_name)?;

    Some(animation_frames)
  }

  /// Adds a new animation for the model to be able to run.
  ///
  /// If an animation of that name already exists, the old one is returned and the new one is stored in its place.
  pub fn add_new_animation_to_list(
    &mut self,
    animation_name: String,
    animation: AnimationFrames,
  ) -> Option<AnimationFrames> {
    self.animations.insert(animation_name, animation)
  }

  /// Returns a reference to the animation of the given name.
  ///
  /// None is returned if there was no animation with that name.
  pub fn get_animation(&self, animation_name: &str) -> Option<&AnimationFrames> {
    self.animations.get(animation_name)
  }

  /// Adds an animation to the back of the queue.
  ///
  /// # Errors
  ///
  /// - An animation of that name doesn't exist, and therefore cannot be added to the queue.
  pub fn queue_animation(&mut self, animation_name: impl AsRef<str>) -> Result<(), AnimationError> {
    let animation_name = animation_name.as_ref().to_owned();

    if self.contains_animation(&animation_name) {
      self
        .model_animator
        .borrow_mut()
        .add_new_animation_to_queue(animation_name);
    } else {
      return Err(AnimationError::AnimationDoesntExist {
        invalid_animation_name: animation_name.clone(),
      });
    }

    Ok(())
  }

  /// Stops the currently running animation, and replaces it with the one passed in.
  ///
  /// # Errors
  ///
  /// - An animation of that name doesn't exist, and therefore cannot be added to the queue.
  pub fn overwrite_current_model_animation(
    &mut self,
    new_animation: impl AsRef<str>,
  ) -> Result<(), AnimationError> {
    let new_animation = new_animation.as_ref();

    if self.contains_animation(new_animation) {
      self
        .model_animator
        .borrow_mut()
        .overwrite_current_animation(new_animation.to_owned());
    } else {
      return Err(AnimationError::AnimationDoesntExist {
        invalid_animation_name: new_animation.to_string(),
      });
    }

    Ok(())
  }

  /// Removes every animation running in the animation queue.
  ///
  /// The last existing animation to be run in the list is assigned to the last_run_animation.
  pub fn clear_animation_queue(&mut self) {
    self
      .model_animator
      .borrow_mut()
      .clear_queue(&self.animations)
  }

  /// Removes the currently running animation from the queue, and moves on to the next animation in queue.
  pub fn remove_current_model_animation_from_queue(&mut self) {
    self
      .model_animator
      .borrow_mut()
      .step_animation_queue(&self.animations)
  }

  /// Returns a reference to the stored animations.
  pub fn get_animation_list(&self) -> &HashMap<String, AnimationFrames> {
    &self.animations
  }
}

impl<I> From<I> for ModelAnimationData
where
  I: IntoIterator<Item = (String, AnimationFrames)>,
{
  fn from(animation_list: I) -> Self {
    let animation_list: HashMap<String, AnimationFrames> = animation_list.into_iter().collect();

    Self {
      animations: animation_list,
      ..Default::default()
    }
  }
}

impl std::fmt::Debug for ModelAnimationData {
  fn fmt(
    &self,
    formatter: &mut std::fmt::Formatter<'_>,
  ) -> std::result::Result<(), std::fmt::Error> {
    formatter
      .debug_struct("ModelAnimationData")
      .field("animations", &self.animations)
      .finish()
  }
}

// #[cfg(test)]
// mod tests {
//   use super::*;
//   use crate::models::testing_data::*;
//
//   #[test]
//   fn model_animation_data_from() {
//     let test_animation_name = TestingData::ANIMATION_NAME.to_owned();
//     let (model, test_animation) = TestingData::new_test_model_animated((10, 10), ['1', '2', '3']);
//
//     let expected_animations_list =
//       HashMap::from([(test_animation_name.clone(), test_animation.clone())]);
//
//     let animation_data =
//       ModelAnimationData::from((model, vec![(test_animation_name, test_animation)]));
//
//     assert_eq!(animation_data.animations, expected_animations_list);
//   }
//
//   #[test]
//   fn contains_animation_logic() {
//     let test_animation_name = TestingData::ANIMATION_NAME;
//     let (mut model, _) = TestingData::new_test_model_animated((10, 10), ['1', '2', '3']);
//
//     let model_animation_data = model.get_animation_data().unwrap();
//     let mut model_animation_data = model_animation_data.lock().unwrap();
//
//     assert!(model_animation_data.contains_animation(test_animation_name));
//
//     let _ = model_animation_data
//       .remove_animation_from_list(test_animation_name)
//       .unwrap();
//
//     assert!(!model_animation_data.contains_animation(test_animation_name));
//   }
//
//   #[test]
//   fn add_new_animation_to_list_logic() {
//     let animation_name = "test";
//     let animation =
//       TestingData::get_test_animation(['1', '2', '3'], AnimationLoopCount::Limited(1));
//     let mut model = TestingData::new_test_model_with_animation((10, 10), vec![]);
//
//     let model_animation_data = model.get_animation_data().unwrap();
//     let mut model_animation_data = model_animation_data.lock().unwrap();
//
//     model_animation_data
//       .add_new_animation_to_list(animation_name.to_owned(), animation.clone())
//       .unwrap();
//
//     assert!(model_animation_data.contains_animation(animation_name));
//
//     let result = model_animation_data
//       .add_new_animation_to_list(animation_name.to_owned(), animation)
//       .unwrap_err();
//
//     assert_eq!(result, AnimationError::AnimationAlreadyExists);
//   }
// }
