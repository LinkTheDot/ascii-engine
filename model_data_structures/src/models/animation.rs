// // Change models to handle their own animations.
// // There's no reason to have some sort of communication system with models. It only
// // increases the complexity a lot.
// // Instead, a model's ModelAnimationData should contain an Arc<Mutex<ModelAnimatorData>>.
// // When the model wants to change something about it's current animation state, it'll
// // use methods tied to the ModelAnimatorData which will return a Guard wrapping the internal
// // ModelAnimatorData.
// // From there, the ModelAnimatorData will have methods similar to the AnimationAction.
// //
// // For removing the model from the animation list, a method will need to send a request to the
// // animation thread, then delete the internally stored reference to the ModelAnimator.
//
// use crate::errors::*;
use crate::models::errors::*;
use crate::models::model_appearance::sprites::Sprite;
use crate::models::model_data::ModelData;
pub use animation_frames::*;
pub use model_animator::*;
use std::collections::{hash_map::Entry, HashMap};
// pub use animation_connections::*;
// use event_sync::EventSync;
// use std::collections::VecDeque;
// use std::sync::{Arc, Mutex, MutexGuard};
// use tokio::sync::mpsc;

pub mod animation_connections;
pub mod animation_frames;
pub mod animation_frames_iterators;
pub mod model_animator;

/// Stores the list of existing model animations, and the data for the model's
/// current animation state.
#[derive(Default, Clone)]
pub struct ModelAnimationData {
  animations: HashMap<String, AnimationFrames>,
  model_animator: ModelAnimator,
  // /// Contains the list of names of animations to be run.
  // animation_queue: VecDeque<String>,
  // /// Contains the name of the current running animation.
  // current_animation: Option<String>,
  // /// Contains an EventSync for when the animation started.
  // /// The tickrate contained is based on the tickrate in the config file.
  // current_animation_start: Option<EventSync>,
}

impl ModelAnimationData {
  pub fn new<I>(model: ModelData, animation_list: I) -> Self
  where
    I: IntoIterator<Item = (String, AnimationFrames)>,
  {
    Self::from((model, animation_list))
  }

  //   // // TODO: List the errors.
  //   // pub fn from_file(animation_directory: std::path::PathBuf) -> Result<Self, AnimationError> {
  //   //   if !animation_directory.is_dir() {
  //   //     log::error!(
  //   //       "Attempted to build an object with an animation file instead of an animation directory"
  //   //     );
  //   //
  //   //     let animation_path = animation_directory.into_os_string();
  //   //
  //   //     return Err(AnimationError::AnimationDirectoryIsFile(animation_path));
  //   //   } else if !animation_directory.exists() {
  //   //     log::error!("Attempted to build an object with an invalid defined animation path");
  //   //
  //   //     let animation_path = animation_directory.into_os_string();
  //   //
  //   //     return Err(AnimationError::AnimationDirectoryDoesntExist(
  //   //       animation_path,
  //   //     ));
  //   //   }
  //   //
  //   //   let Ok(animation_directory_contents) = animation_directory.read_dir() else {
  //   //     let error =
  //   //       AnimationParserError::CouldntGetAnimationPath(animation_directory.into_os_string());
  //   //
  //   //     return Err(AnimationError::AnimationParserError(error));
  //   //   };
  //   //
  //   //   let _animation_directory_contents: Vec<PathBuf> = animation_directory_contents
  //   //     .filter_map(|file_dir_entry| Some(file_dir_entry.ok()?.path()))
  //   //     .filter(|file_path| file_path.extension() == Some(OsStr::new("animate")))
  //   //     .collect();
  //   //
  //   //   todo!()
  //   //   // AnimationParser::parse(animation_directory_contents)
  //   // }

  pub fn get_current_appearance(&mut self) -> Option<&Sprite> {
    self
      .model_animator
      .get_current_model_appearance(&self.animations)
  }

  /// Returns true if there's an animation with the given name.
  pub fn contains_animation(&self, animation_name: &str) -> bool {
    self.animations.contains_key(animation_name)
  }

  /// Removes the animation from the list of animations, and returns it if it existed.
  pub fn remove_animation_from_list(&mut self, animation_name: &str) -> Option<AnimationFrames> {
    let (_, animation_frames) = self.animations.remove_entry(animation_name)?;

    Some(animation_frames)
  }

  /// Adds a new animation for the model to be able to run.
  ///
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

  //   pub fn get_model_animator(&mut self) -> &ModelAnimator {
  //     &self.model_animator
  //   }
  //
  //   // pub fn send_model_animator_request(
  //   //   &mut self,
  //   //   model_hash: &u64,
  //   //   sender: &mpsc::UnboundedSender<AnimationRequest>,
  //   // ) {
  //   //   let animation_request = AnimationRequest {
  //   //     model_unique_hash: *model_hash,
  //   //     request: AnimationAction::AddAnimator(self.model_animator.clone()),
  //   //   };
  //   //
  //   //   let _ = sender.send(animation_request);
  //   // }
  //
  //   /// Returns a reference to the list of animations stored.
  //   pub fn get_animation_list(&self) -> &HashMap<String, AnimationFrames> {
  //     &self.animations
  //   }
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
