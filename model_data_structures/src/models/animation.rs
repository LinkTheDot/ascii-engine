// Change models to handle their own animations.
// There's no reason to have some sort of communication system with models. It only
// increases the complexity a lot.
// Instead, a model's ModelAnimationData should contain an Arc<Mutex<ModelAnimatorData>>.
// When the model wants to change something about it's current animation state, it'll
// use methods tied to the ModelAnimatorData which will return a Guard wrapping the internal
// ModelAnimatorData.
// From there, the ModelAnimatorData will have methods similar to the AnimationAction.
//
// For removing the model from the animation list, a method will need to send a request to the
// animation thread, then delete the internally stored reference to the ModelAnimator.

use crate::errors::*;
use crate::models::{model_data::ModelData, sprites::Sprite};
pub use animation_connections::*;
pub use animation_frames::*;
pub use model_animator::*;
use std::collections::{hash_map::Entry, HashMap};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::mpsc;

pub mod animation_connections;
pub mod animation_frames;
pub mod animation_frames_iterators;
pub mod model_animator;

/// Stores a model's animations and and current animation state.
#[derive(Debug)]
pub struct ModelAnimationData {
  animations: HashMap<String, AnimationFrames>,
  model_animator: Arc<Mutex<ModelAnimator>>,
}

impl ModelAnimationData {
  pub fn new<I>(model: ModelData, animation_list: I) -> Self
  where
    I: IntoIterator<Item = (String, AnimationFrames)>,
  {
    Self::from((model, animation_list))
  }

  pub fn from_file(animation_directory: std::path::PathBuf) -> Result<Self, AnimationError> {
    if !animation_directory.is_dir() {
      log::error!(
        "Attempted to build an object with an animation file instead of an animation directory"
      );

      let animation_path = animation_directory.into_os_string();

      return Err(AnimationError::AnimationDirectoryIsFile(animation_path));
    } else if !animation_directory.exists() {
      log::error!("Attempted to build an object with an invalid defined animation path");

      let animation_path = animation_directory.into_os_string();

      return Err(AnimationError::AnimationDirectoryDoesntExist(
        animation_path,
      ));
    }

    let Ok(animation_directory_contents) = animation_directory.read_dir() else {
      let error =
        AnimationParserError::CouldntGetAnimationPath(animation_directory.into_os_string());

      return Err(AnimationError::AnimationParserError(error));
    };

    let _animation_directory_contents: Vec<PathBuf> = animation_directory_contents
      .filter_map(|file_dir_entry| Some(file_dir_entry.ok()?.path()))
      .filter(|file_path| file_path.extension() == Some(OsStr::new("animate")))
      .collect();

    todo!()
    // AnimationParser::parse(animation_directory_contents)
  }

  pub fn contains_animation(&self, animation_name: &str) -> bool {
    self.animations.contains_key(animation_name)
  }

  pub fn remove_animation_from_list(&mut self, animation_name: String) -> Option<AnimationFrames> {
    let (_, animation_frames) = self.animations.remove_entry(&animation_name)?;

    Some(animation_frames)
  }

  /// # Errors
  ///
  /// - An error is returned when the given animation already exists.
  pub fn add_new_animation_to_list(
    &mut self,
    animation_name: String,
    animation: AnimationFrames,
  ) -> Result<(), AnimationError> {
    if let Entry::Vacant(entry) = self.animations.entry(animation_name) {
      entry.insert(animation);
    } else {
      return Err(AnimationError::AnimationAlreadyExists);
    }

    Ok(())
  }

  /// Returns a reference to the animation of the given name.
  ///
  /// None is returned if there was no animation with that name.
  pub fn get_animation(&self, animation_name: &str) -> Option<&AnimationFrames> {
    self.animations.get(animation_name)
  }

  /// Returns the MutexGuard for the internal [`ModelAnimator`](crate::models::animation::model_animator::ModelAnimator).
  pub fn get_model_animator(&mut self) -> MutexGuard<ModelAnimator> {
    self.model_animator.lock().unwrap()
  }

  pub fn send_model_animator_request(
    &mut self,
    model_hash: &u64,
    sender: &mpsc::UnboundedSender<AnimationRequest>,
  ) {
    let animation_request = AnimationRequest {
      model_unique_hash: *model_hash,
      request: AnimationAction::AddAnimator(self.model_animator.clone()),
    };

    let _ = sender.send(animation_request);
  }

  /// Returns a reference to the list of animations stored.
  pub fn get_animation_list(&self) -> &HashMap<String, AnimationFrames> {
    &self.animations
  }
}

impl<I> From<(ModelData, I)> for ModelAnimationData
where
  I: IntoIterator<Item = (String, AnimationFrames)>,
{
  fn from((model, animation_list): (ModelData, I)) -> Self {
    let animation_list: HashMap<String, AnimationFrames> = animation_list.into_iter().collect();
    let model_sprite = model.get_sprite();

    Self {
      animations: animation_list,
      model_animator: ModelAnimator::new(model_sprite),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::testing_data::*;

  const WORLD_POSITION: (usize, usize) = (10, 10);

  #[test]
  fn model_animation_data_from() {
    let test_animation_name = "test_animation".to_string();
    let test_animation =
      TestingData::get_test_animation(['r', 's', 't'], AnimationLoopCount::Limited(1));
    let animation_list: Vec<(String, AnimationFrames)> =
      vec![(test_animation_name.clone(), test_animation.clone())];
    let model = TestingData::new_test_model(WORLD_POSITION);

    let expected_animations_list = HashMap::from([(test_animation_name, test_animation)]);

    let animation_data = ModelAnimationData::from((model, animation_list));

    assert_eq!(animation_data.animations, expected_animations_list);
  }
}
