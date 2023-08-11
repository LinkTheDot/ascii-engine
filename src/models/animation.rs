// Rename ModelAnimatorData to something else.
// It sounds and looks too similar to ModelAnimationData
use crate::errors::*;
use crate::general_data::hasher::get_unique_hash;
use crate::models::animation_file_parser::*;
use crate::models::model_data::ModelData;
use crate::screen::screen_data::ScreenData;
pub use animation_frames::*;
use lazy_static::lazy_static;
use std::collections::{hash_map::Entry, HashMap, VecDeque};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::thread::JoinHandle;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

pub mod animation_frames;
pub mod animation_frames_iterators;

lazy_static! {
  /// This is the hash that will be passed in when requesting to kill the animation thread.
  ///
  /// The only way to access this should be through an [`AnimationConnection`](AnimationConnection), which will only ever be owned by the screen.
  static ref KILL_HASH: u64 = get_unique_hash();
}

pub(crate) struct ModelAnimationData {
  animations: HashMap<String, AnimationFrames>,
  animation_communicator: Option<mpsc::UnboundedSender<AnimationRequest>>,
}

#[derive(Debug)]
struct ModelAnimatorData {
  model: ModelData,
  animation_queue: VecDeque<AnimationFrames>,
  current_animation: Option<AnimationFramesIntoIter>,
  iteration_of_last_frame_change: u64,
}

pub(crate) struct AnimationConnection {
  _handle: JoinHandle<()>,
  request_sender: mpsc::UnboundedSender<AnimationRequest>,
  kill_sender: oneshot::Sender<()>,
}

#[derive(Debug)]
pub(crate) struct AnimationRequest {
  model_unique_hash: u64,
  request: AnimationAction,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AnimationAction {
  AddToQueue(AnimationFrames),
  OverwriteCurrentAnimation(AnimationFrames),
  ClearQueue,
  AddAnimator(ModelData),
  RemoveAnimatior,
  KillThread,
}

impl ModelAnimatorData {
  fn new(model: ModelData) -> Self {
    Self {
      model,
      animation_queue: VecDeque::new(),
      current_animation: None,
      iteration_of_last_frame_change: 0,
    }
  }

  /// Returns true if the animation being run has requested to be removed from the list.
  fn run_request(&mut self, animation_action: AnimationAction) -> bool {
    match animation_action {
      AnimationAction::AddToQueue(animation_frames) => {
        self.add_new_animation_to_queue(animation_frames);
      }

      AnimationAction::OverwriteCurrentAnimation(animation_frames) => {
        self.overwrite_current_animation(animation_frames);
      }

      AnimationAction::ClearQueue => self.clear_queue(),

      AnimationAction::RemoveAnimatior => return true,

      AnimationAction::AddAnimator(_) => {
        log::error!("Attempted to add a model animator through another model.")
      }

      AnimationAction::KillThread => {
        panic!("It shouldn't be possible to call KillThread from a model.");
      }
    }

    false
  }

  // TODO prevent users from making invalid frames in the animation file parser
  /// This will panic if the frame is invalid in any way.
  ///
  /// That will cause a chain reaction where it'll poison every mutex for every instance of ModelData.
  /// This means that if any animation file has an incorrect animation sequence, the program will crash.
  fn change_model_frame(&mut self, new_frame: AnimationFrame) {
    let new_appearance = new_frame.get_appearance();
    let anchor_char_replacement = new_frame.get_anchor_replacement_char();

    if let Err(error) = self
      .model
      .change_sprite_appearance(new_appearance, anchor_char_replacement)
    {
      let error_message = format!(
        "A model has attempted to animate with an invalid frame. Model Hash: {}, Model Name: {}, Error: {error:?}", 
          self.model.get_unique_hash(),
          self.model.get_name());

      log::error!("{error_message}");

      panic!("{error_message}");
    }
  }

  /// Returns an error if the queue was empty.
  ///
  /// Removes an animation from the front of the queue and assigns it to the currently looping animation.
  ///
  /// This method also restarts the ``current_animation_iteration_counter``.
  fn overwrite_current_animation_with_first_in_queue(&mut self) -> Result<(), AnimationError> {
    if let Some(new_animation) = self.animation_queue.pop_front() {
      self.overwrite_current_animation(new_animation);
    } else {
      return Err(AnimationError::EmptyQueue);
    }

    Ok(())
  }

  /// Returns true if there are animations that can be run.
  ///
  /// This means that either there's already an animation running, or there's animations lined up in the queue.
  fn has_animations_to_run(&self) -> bool {
    self.has_animations_queued() || self.current_animation.is_some()
  }

  /// Returns true if there's any animations currently in the queue.
  fn has_animations_queued(&self) -> bool {
    !self.animation_queue.is_empty()
  }

  /// Replaces the currently running animation and replaces it with the one that was passed in.
  ///
  /// This method also restarts the ``current_animation_iteration_counter``.
  fn overwrite_current_animation(&mut self, new_animation: AnimationFrames) {
    self.current_animation = Some(new_animation.into_iter());
  }

  fn clear_queue(&mut self) {
    self.animation_queue.clear();
  }

  /// Assigns the passed in new animation if there's none currently running.
  /// Otherwise adds the new animation to the back of the queue.
  fn add_new_animation_to_queue(&mut self, new_animation: AnimationFrames) {
    if self.current_animation.is_none() {
      self.current_animation = Some(new_animation.into_iter())
    } else {
      self.animation_queue.push_back(new_animation);
    }
  }

  /// Replaces the internal value for the last iteration a frame was changed on.
  fn update_when_last_frame_changed(&mut self, current_iteration: u64) {
    self.iteration_of_last_frame_change = current_iteration;
  }

  /// Gets the next animation frame and returns it.
  ///
  /// None is returned if there is no currently assigned animation.
  fn next_frame(&mut self) -> Option<AnimationFrame> {
    self.current_animation.as_mut()?.next()
  }

  /// Returns None if there is no current animation
  fn get_current_animation(&self) -> Option<&AnimationFramesIntoIter> {
    self.current_animation.as_ref()
  }

  /// Returns None if there is no current animation.
  fn get_current_animation_frame_duration(&self) -> Option<u64> {
    self.get_current_animation()?.current_frame_duration()
  }

  /// Returns true if the current frame's duration is over.
  ///
  /// If there is no currently running animation `false` is returned.
  fn current_frame_duration_is_finished(&self, current_iteration: u64) -> bool {
    if let Some(current_frame_tick_duration) = self.get_current_animation_frame_duration() {
      self.iteration_of_last_frame_change + current_frame_tick_duration <= current_iteration
    } else {
      true
    }
  }
}

impl ModelAnimationData {
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
      let error = AnimationParserError::CouldntGetAnimationPath(animation_directory.into_os_string());

      return Err(AnimationError::AnimationParserError(error));
    };

    let animation_directory_contents: Vec<PathBuf> = animation_directory_contents
      .filter_map(|file_dir_entry| Some(file_dir_entry.ok()?.path()))
      .filter(|file_path| file_path.extension() == Some(OsStr::new("animate")))
      .collect();

    AnimationParser::parse(animation_directory_contents)
  }

  pub fn new() -> Self {
    Self {
      animations: HashMap::new(),
      animation_communicator: None,
    }
  }

  /// Returns true if this animation_data has been added to the animation thread.
  pub fn is_started(&self) -> bool {
    self.animation_communicator.is_some()
  }

  pub fn contains_animation(&self, animation_name: &str) -> bool {
    self.animations.contains_key(animation_name)
  }

  /// Starts the thread that will handle model animations
  ///
  // Describe how to send requests and whatnot
  ///
  /// # Errors
  ///
  /// - Returns an error when the model animation thread already exists.
  pub(crate) fn start_animation_thread(
    screen_data: &ScreenData,
  ) -> Result<AnimationConnection, AnimationError> {
    if screen_data.animation_thread_started() {
      return Err(AnimationError::AnimationThreadAlreadyStarted);
    }

    let (sender, mut receiver) = mpsc::unbounded_channel::<AnimationRequest>();
    let (kill_sender, mut kill_receiver) = oneshot::channel::<()>();
    let event_sync = screen_data.get_event_sync();

    let animation_thread_handle = std::thread::spawn(move || {
      let mut model_animator_data_list: HashMap<u64, ModelAnimatorData> = HashMap::new();
      let mut iteration = 0;

      while kill_receiver.try_recv().is_err() {
        iteration += 1;

        event_sync.wait_for_tick();

        if Self::no_animators_are_running(&model_animator_data_list) {
          log::debug!("No animators running, animation thread waiting for a request.");

          if let Some(request_data) = receiver.blocking_recv() {
            log::debug!("No animators running, animation thread got a request!");
            log::debug!("The request was from {}", request_data.model_unique_hash);

            if request_data.model_unique_hash == *KILL_HASH {
              if request_data.request != AnimationAction::KillThread {
                log::error!("Attempted to make a request through the screen with the kill hash.");
              } else {
                break;
              }
            }

            Self::run_model_request(&mut model_animator_data_list, request_data);
          }
        } else {
          while let Ok(request_data) = receiver.try_recv() {
            Self::run_model_request(&mut model_animator_data_list, request_data);
          }
        }

        // TODO Stop the user from making an animation with 0 frames, that will cause a divide by 0.
        // TODO Force at least 1 tick wait times between frames when parsing the animation file.
        model_animator_data_list
          .values_mut()
          .for_each(|animation_data| {
            if animation_data.has_animations_to_run() {
              if animation_data.current_animation.is_none() {
                animation_data
                  .overwrite_current_animation_with_first_in_queue()
                  .unwrap();
              }

              if animation_data.current_frame_duration_is_finished(iteration) {
                let new_model_frame = match animation_data.next_frame() {
                  Some(animation_frame) => animation_frame,
                  None => {
                    if animation_data
                      .overwrite_current_animation_with_first_in_queue()
                      .is_err()
                    {
                      return;
                    } else {
                      animation_data.next_frame().unwrap()
                    }
                  }
                };

                animation_data.update_when_last_frame_changed(iteration);
                animation_data.change_model_frame(new_model_frame);
              }
            }
          });
      }

      log::warn!("Animation thread has ended.");
    });

    Ok(AnimationConnection {
      _handle: animation_thread_handle,
      request_sender: sender,
      kill_sender,
    })
  }

  /// # Errors
  ///
  /// - An error is returned when the model hasn't started it's animations.
  pub(crate) fn send_request(
    &mut self,
    model_hash: u64,
    request: AnimationAction,
  ) -> Result<(), AnimationError> {
    if let Some(animation_sender) = &self.animation_communicator {
      let animation_action_request = AnimationRequest {
        model_unique_hash: model_hash,
        request,
      };

      animation_sender.send(animation_action_request).unwrap();
    } else {
      return Err(AnimationError::AnimationNotStarted);
    }

    Ok(())
  }

  /// # Errors
  ///
  /// - An error is returned when the given animation name doesn't exist in the list of animations.
  pub(crate) fn get_animation(
    &self,
    animation_name: String,
  ) -> Result<AnimationFrames, AnimationError> {
    let animation = self.animations.get(&animation_name);

    match animation {
      Some(animation_frames) => Ok(animation_frames.clone()),
      None => Err(AnimationError::AnimationDoesntExist),
    }
  }

  /// # Errors
  ///
  /// - An error is returned when the given animation already exists.
  pub(crate) fn add_new_animation_to_list(
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

  pub(crate) fn remove_animation_from_list(
    &mut self,
    animation_name: String,
  ) -> Result<AnimationFrames, AnimationError> {
    if let Some((_, animation_frames)) = self.animations.remove_entry(&animation_name) {
      Ok(animation_frames)
    } else {
      Err(AnimationError::AnimationDoesntExist)
    }
  }

  /// # Errors
  ///
  /// - An error is returned when the model hasn't started it's animations.
  pub(crate) fn assign_communicator(
    &mut self,
    communicator: mpsc::UnboundedSender<AnimationRequest>,
  ) -> Result<(), AnimationError> {
    if self.animation_communicator.is_none() {
      self.animation_communicator = Some(communicator);
    } else {
      return Err(AnimationError::AnimationDataAlreadyHasConnection);
    }

    Ok(())
  }

  fn no_animators_are_running(model_animator_list: &HashMap<u64, ModelAnimatorData>) -> bool {
    model_animator_list.values().all(|model_animator| {
      model_animator.animation_queue.is_empty() && model_animator.current_animation.is_none()
    })
  }

  fn run_model_request(
    model_animator_list: &mut HashMap<u64, ModelAnimatorData>,
    request: AnimationRequest,
  ) {
    if let Some(called_model_animator) = model_animator_list.get_mut(&request.model_unique_hash) {
      let removal_request = called_model_animator.run_request(request.request);

      if removal_request {
        model_animator_list.remove(&request.model_unique_hash);
      }
    } else if let AnimationAction::AddAnimator(add_model) = request.request {
      model_animator_list.insert(request.model_unique_hash, ModelAnimatorData::new(add_model));
    } else {
      log::warn!("Attempted to call an animation request with an invalid model hash");
    }
  }
}

impl AnimationConnection {
  pub(crate) fn clone_sender(&self) -> mpsc::UnboundedSender<AnimationRequest> {
    self.request_sender.clone()
  }

  /// # Errors
  ///
  /// - An error is returned when the animation thread doesn't exist.
  pub(crate) fn kill_thread(self) {
    self.kill_sender.send(()).unwrap();

    let kill_request = AnimationRequest {
      model_unique_hash: *KILL_HASH,
      request: AnimationAction::KillThread,
    };

    self.request_sender.send(kill_request).unwrap();
  }
}

impl core::fmt::Debug for ModelAnimationData {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{{ ... }}")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[cfg(test)]
  mod start_animation_thread_logic {
    use super::*;

    #[test]
    fn start_thread_once() {
      let screen_data = ScreenData::new();

      ModelAnimationData::start_animation_thread(&screen_data).unwrap();
    }

    #[test]
    #[should_panic]
    fn start_thread_multiple_times() {
      let mut screen_data = ScreenData::new();

      screen_data.start_animation_thread().unwrap();
      ModelAnimationData::start_animation_thread(&screen_data).unwrap();
    }
  }

  // how to test the animation thread
  //   check if it's waiting properly
  //     > start the thread
  //     > add 2 models
  //     > wait 2 ticks for it to sit and wait
  //     > send in 2 animation requests for both models at the same time
  //     > if only one of the models changed after that, it waited properly
  //   check if it's running all animations
  //     > start the thread
  //     > add 2 models
  //     > send requests to add animations
  //     > if they both changed, it's running requests properly

  #[cfg(test)]
  mod model_animator_methods {
    use super::*;

    #[cfg(test)]
    mod run_request_logic {
      use super::*;

      #[test]
      fn add_to_queue_no_running_animation() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);
        let animation_frames = get_test_animation_limited_run_count();
        let request = AnimationAction::AddToQueue(animation_frames);

        let result = model_animator.run_request(request);

        assert!(!result);
        assert!(model_animator.animation_queue.is_empty());
        assert!(model_animator.current_animation.is_some());
      }

      #[test]
      fn add_to_queue_currently_running_animation() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);
        let animation_frames = get_test_animation_limited_run_count();
        model_animator.overwrite_current_animation(animation_frames.clone());
        let request = AnimationAction::AddToQueue(animation_frames);

        let result = model_animator.run_request(request);

        assert!(!result);
        assert!(!model_animator.animation_queue.is_empty());
        assert!(model_animator.current_animation.is_some());
      }

      #[test]
      fn overwrite_current_animation() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);
        let animation_frames = get_test_animation_limited_run_count();
        let initial_appearance_request = AnimationAction::AddToQueue(animation_frames);
        model_animator.run_request(initial_appearance_request);
        let replacing_animation_frames = get_test_animation_unlimited_run_count();
        let overwrite_request =
          AnimationAction::OverwriteCurrentAnimation(replacing_animation_frames);

        let expected_frame_appearance = "ooooo\nooaoo\nooooo".to_string();

        let result = model_animator.run_request(overwrite_request);
        let frame_0_appearance = model_animator
          .next_frame()
          .unwrap()
          .get_appearance()
          .to_owned();

        assert!(!result);
        assert!(model_animator.animation_queue.is_empty());
        assert_eq!(frame_0_appearance, expected_frame_appearance);
      }

      #[test]
      fn clear_queue() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);
        let animation_frames = get_test_animation_limited_run_count();
        let appearance_request = AnimationAction::AddToQueue(animation_frames);
        model_animator.run_request(appearance_request.clone());
        model_animator.run_request(appearance_request);

        assert!(!model_animator.animation_queue.is_empty());

        let request = AnimationAction::ClearQueue;
        let result = model_animator.run_request(request);

        assert!(!result);
        assert!(!model_animator.has_animations_queued());
      }

      #[test]
      // Nothing should happen other than an error being logged
      fn add_animator() {
        let model_data = get_test_model_data();
        let other_model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);
        let request = AnimationAction::AddAnimator(other_model_data);

        let result = model_animator.run_request(request);

        assert!(!result);
      }

      #[test]
      fn remove_animator() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);
        let request = AnimationAction::RemoveAnimatior;

        let result = model_animator.run_request(request);

        assert!(result);
      }
    }

    #[cfg(test)]
    mod change_model_frame_logic {
      use super::*;

      #[test]
      fn valid_input() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data.clone());
        let frame_appearance = "-----\n--a--\n-----".to_string();
        let new_frame = AnimationFrame::new(frame_appearance, 1, Some('-'));

        let expected_appearance = "-----\n-----\n-----".to_string();

        model_animator.change_model_frame(new_frame);

        let model_appearance = model_data.get_sprite();

        assert_eq!(model_appearance, expected_appearance);
      }

      #[test]
      #[should_panic]
      fn invalid_frame_shape() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);
        let frame_appearance = "--\n--a--\n-----".to_string();
        let new_frame = AnimationFrame::new(frame_appearance, 1, Some('-'));

        model_animator.change_model_frame(new_frame);
      }
    }

    #[cfg(test)]
    mod overwrite_current_animation_with_first_in_queue_logic {
      use super::*;

      #[test]
      fn empty_queue_running_animation() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);
        let running_animation = get_test_animation_limited_run_count();
        model_animator.overwrite_current_animation(running_animation.clone());

        let expected_animation = running_animation.get_frame(0).cloned();

        let result = model_animator.overwrite_current_animation_with_first_in_queue();
        let current_animation = model_animator.next_frame();

        assert!(result.is_err());
        assert_eq!(current_animation, expected_animation);
      }

      #[test]
      fn empty_queue_no_running_animation() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);

        let result = model_animator.overwrite_current_animation_with_first_in_queue();

        assert!(result.is_err());
        assert!(model_animator.current_animation.is_none());
      }

      #[test]
      fn queue_has_contents_running_animation() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);
        let running_animation = get_test_animation_limited_run_count();
        let queued_animation = get_test_animation_unlimited_run_count();
        model_animator.overwrite_current_animation(running_animation);
        model_animator
          .animation_queue
          .push_front(queued_animation.clone());

        let expected_animation = queued_animation.get_frame(0).cloned();

        let result = model_animator.overwrite_current_animation_with_first_in_queue();
        let current_animation = model_animator.next_frame();

        assert!(result.is_ok());
        assert_eq!(current_animation, expected_animation);
      }

      #[test]
      fn queue_has_contents_no_running_animation() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);
        let queued_animation = get_test_animation_limited_run_count();
        model_animator
          .animation_queue
          .push_front(queued_animation.clone());

        let expected_animation = queued_animation.get_frame(0).cloned();

        let result = model_animator.overwrite_current_animation_with_first_in_queue();
        let current_animation = &model_animator.next_frame();

        assert!(result.is_ok());
        assert_eq!(current_animation, &expected_animation);
      }
    }

    #[cfg(test)]
    mod has_animation_to_run_logic {
      use super::*;

      #[test]
      fn has_animations() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);
        let animation_one = get_test_animation_limited_run_count();
        let animation_two = get_test_animation_limited_run_count();

        model_animator.add_new_animation_to_queue(animation_one);
        model_animator.add_new_animation_to_queue(animation_two);

        let queue_and_current_result = model_animator.has_animations_to_run();

        // has current animation but none in queue
        model_animator
          .overwrite_current_animation_with_first_in_queue()
          .unwrap();

        let empty_queue_and_current_result = model_animator.has_animations_to_run();

        assert!(queue_and_current_result);
        assert!(empty_queue_and_current_result);
      }

      #[test]
      fn has_no_animations() {
        let model_data = get_test_model_data();
        let model_animator = ModelAnimatorData::new(model_data);

        assert!(!model_animator.has_animations_to_run());
      }
    }

    #[test]
    fn overwrite_current_animation_logic() {
      let model_data = get_test_model_data();
      let mut model_animator = ModelAnimatorData::new(model_data);
      let running_animation = get_test_animation_limited_run_count();
      let replacing_animation = get_test_animation_unlimited_run_count();
      model_animator.add_new_animation_to_queue(running_animation.clone());

      let first_frame_before = model_animator.next_frame();

      model_animator.overwrite_current_animation(replacing_animation.clone());

      let first_frame_after = model_animator.next_frame();

      assert_eq!(first_frame_before, running_animation.get_frame(0).cloned());
      assert_eq!(first_frame_after, replacing_animation.get_frame(0).cloned());
    }

    #[cfg(test)]
    mod add_new_animation_to_queue {
      use super::*;

      #[test]
      fn empty_queue() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);
        let animation = get_test_animation_limited_run_count();

        model_animator.add_new_animation_to_queue(animation.clone());

        let current_animation = model_animator.next_frame();

        assert_eq!(current_animation, animation.get_frame(0).cloned());
        assert!(model_animator.animation_queue.is_empty());
      }

      #[test]
      fn existing_queue() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);
        let animation_one = get_test_animation_limited_run_count();
        let animation_two = get_test_animation_unlimited_run_count();
        let animation_three = get_test_animation(3);

        model_animator.add_new_animation_to_queue(animation_one.clone());
        model_animator.add_new_animation_to_queue(animation_two.clone());
        model_animator.add_new_animation_to_queue(animation_three.clone());

        let first_frame_of_animation_one = model_animator.next_frame();

        model_animator
          .overwrite_current_animation_with_first_in_queue()
          .unwrap();
        let first_frame_of_animation_two = model_animator.next_frame();

        model_animator
          .overwrite_current_animation_with_first_in_queue()
          .unwrap();
        let first_frame_of_animation_three = model_animator.next_frame();

        assert_eq!(
          first_frame_of_animation_one,
          animation_one.get_frame(0).cloned()
        );
        assert_eq!(
          first_frame_of_animation_two,
          animation_two.get_frame(0).cloned()
        );
        assert_eq!(
          first_frame_of_animation_three,
          animation_three.get_frame(0).cloned()
        );
      }
    }

    #[test]
    fn update_when_last_frame_changed_logic() {
      let model_data = get_test_model_data();
      let mut model_animator = ModelAnimatorData::new(model_data);

      let before_last_frame_change = model_animator.iteration_of_last_frame_change;

      model_animator.update_when_last_frame_changed(10);

      let after_last_frame_change = model_animator.iteration_of_last_frame_change;

      assert_eq!(before_last_frame_change, 0);
      assert_eq!(after_last_frame_change, 10);
    }

    #[cfg(test)]
    mod get_current_animation_logic {
      use super::*;

      #[test]
      fn running_animation() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);
        let model_animation = get_test_animation(3);
        model_animator.add_new_animation_to_queue(model_animation.clone());

        let expected_first_frame = model_animation.get_frame(0).cloned();

        let current_animation = model_animator.get_current_animation().unwrap();
        let first_frame_of_current = current_animation.get_current_frame();

        assert_eq!(first_frame_of_current, expected_first_frame);
      }

      #[test]
      fn no_running_animation() {
        let model_data = get_test_model_data();
        let model_animator = ModelAnimatorData::new(model_data);

        let current_animation = model_animator.get_current_animation();

        assert!(current_animation.is_none());
      }
    }

    #[cfg(test)]
    mod current_frame_duration_is_finished_logic {
      use super::*;

      #[test]
      fn running_frame_not_finished() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);
        let model_animation = get_test_animation(1);
        model_animator.add_new_animation_to_queue(model_animation);

        assert!(!model_animator.current_frame_duration_is_finished(0));
      }

      #[test]
      fn running_frame_finished() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);
        let model_animation = get_test_animation(1);
        model_animator.add_new_animation_to_queue(model_animation);

        let current_frame_duration = model_animator
          .get_current_animation_frame_duration()
          .unwrap();

        assert!(model_animator.current_frame_duration_is_finished(current_frame_duration));
      }

      #[test]
      fn no_running_animation() {
        let model_data = get_test_model_data();
        let model_animator = ModelAnimatorData::new(model_data);

        // This method will return true if there is no animation.
        assert!(model_animator.current_frame_duration_is_finished(0));
      }
    }

    #[test]
    #[should_panic]
    fn kill_thread_through_model() {
      let model_data = get_test_model_data();
      let mut model_animator = ModelAnimatorData::new(model_data);

      let request = AnimationAction::KillThread;

      model_animator.run_request(request);
    }
  }

  #[cfg(test)]
  mod model_animation_data_tests {
    use super::*;

    #[cfg(test)]
    mod from_file_logic {
      // use super::*;

      #[test]
      #[ignore]
      fn empty_path() {}

      #[test]
      #[ignore]
      fn invalid_extension() {}
    }

    #[cfg(test)]
    mod animation_thread_logic {
      use super::*;

      // This test is a bit complicated.
      //
      // Basically, we start the thread and add two models to the animator list.
      // If no animations are running, the thread should be sitting and waiting for a request.
      // During this state, once it receives a request it only runs one of the requests queued.
      //
      // This means if we send in two animation requests while it's waiting, only one gets run.
      // So after getting it to a state of waiting, we send an animation request for each model.
      // If only one of the models changed, then we know that the thread was waiting because
      //   it only ran one of those animation requests.
      #[test]
      // Chance of failing due to time sensitivity
      fn thread_is_awaiting_requests() {
        let mut screen = ScreenData::new();
        let mut model_one = get_test_model_data();
        let mut model_two = get_test_model_data();
        let model_animation = get_test_animation(5);
        log::info!("I am running test \"thread is awaiting requests\"");

        // THIS IS TEMPORARY UNTIL MODEL ANIMATION FILES ARE IMPLEMENTED
        {
          model_one
            .assign_model_animation(ModelAnimationData::new())
            .unwrap();
          model_two
            .assign_model_animation(ModelAnimationData::new())
            .unwrap();

          model_one
            .add_new_animation_to_list("test_animation".into(), model_animation.clone())
            .unwrap();
          model_two
            .add_new_animation_to_list("test_animation".into(), model_animation)
            .unwrap();
        }
        // THIS IS TEMPORARY UNTIL MODEL ANIMATION FILES ARE IMPLEMENTED

        screen.start_animation_thread().unwrap();

        model_one.start_animation(&screen).unwrap();
        model_two.start_animation(&screen).unwrap();

        screen.get_event_sync().wait_for_x_ticks(3);

        model_one
          .queue_next_animation("test_animation".into())
          .unwrap();
        model_two
          .queue_next_animation("test_animation".into())
          .unwrap();
        // Wait for it to process the first animation request before killing the thread.
        screen.get_event_sync().wait_for_x_ticks(1);

        screen.stop_animation_thread().unwrap();

        let model_one_appearance = model_one.get_sprite();
        let model_two_appearance = model_two.get_sprite();

        assert_ne!(model_one_appearance, model_two_appearance);
      }

      #[test]
      // Chance of failing due to time sensitivity
      fn thread_is_running_animations() {
        let mut screen = ScreenData::new();
        let mut model_one = get_test_model_data();
        let mut model_two = get_test_model_data();
        let model_animation = get_test_animation(5);
        log::info!("I am running test \"thread is running animations\"");

        let base_model_appearance = model_one.get_sprite();

        // THIS IS TEMPORARY UNTIL MODEL ANIMATION FILES ARE IMPLEMENTED
        {
          model_one
            .assign_model_animation(ModelAnimationData::new())
            .unwrap();
          model_two
            .assign_model_animation(ModelAnimationData::new())
            .unwrap();

          model_one
            .add_new_animation_to_list("test_animation".into(), model_animation.clone())
            .unwrap();
          model_two
            .add_new_animation_to_list("test_animation".into(), model_animation)
            .unwrap();
        }
        // THIS IS TEMPORARY UNTIL MODEL ANIMATION FILES ARE IMPLEMENTED

        screen.start_animation_thread().unwrap();

        model_one.start_animation(&screen).unwrap();
        model_two.start_animation(&screen).unwrap();

        model_one
          .queue_next_animation("test_animation".into())
          .unwrap();
        model_two
          .queue_next_animation("test_animation".into())
          .unwrap();
        // Wait for it to process the animation requests before checking for changes.
        screen.get_event_sync().wait_for_x_ticks(5);

        let model_one_appearance = model_one.get_sprite();
        let model_two_appearance = model_two.get_sprite();

        println!("{}", model_one_appearance);
        println!("{}", model_two_appearance);

        assert_ne!(model_one_appearance, base_model_appearance);
        assert_ne!(model_two_appearance, base_model_appearance);

        screen.stop_animation_thread().unwrap();
      }
    }
  }

  // data for tests

  const WORLD_POSITION: (usize, usize) = (10, 10);

  fn get_test_model_data() -> ModelData {
    let test_model_path = std::path::Path::new("tests/models/test_square.model");

    ModelData::from_file(test_model_path, WORLD_POSITION).unwrap()
  }

  // This is temporary until animation file parsers are a thing.
  fn get_test_animation_limited_run_count() -> AnimationFrames {
    let frames = vec![
      AnimationFrame::new("lllll\nllall\nlllll".to_string(), 1, None),
      AnimationFrame::new("mmmmm\nmmamm\nmmmmm".to_string(), 1, None),
      AnimationFrame::new("nnnnn\nnnann\nnnnnn".to_string(), 1, None),
    ];

    AnimationFrames::new(frames, AnimationLoopCount::Limited(5))
  }

  // This is temporary until animation file parsers are a thing.
  fn get_test_animation_unlimited_run_count() -> AnimationFrames {
    let frames = vec![
      AnimationFrame::new("ooooo\nooaoo\nooooo".to_string(), 1, None),
      AnimationFrame::new("ppppp\nppapp\nppppp".to_string(), 1, None),
      AnimationFrame::new("qqqqq\nqqaqq\nqqqqq".to_string(), 1, None),
    ];

    AnimationFrames::new(frames, AnimationLoopCount::Forever)
  }

  // This is temporary until animation file parsers are a thing.
  fn get_test_animation(loop_count: u64) -> AnimationFrames {
    let frames = vec![
      AnimationFrame::new("rrrrr\nrrarr\nrrrrr".to_string(), 1, None),
      AnimationFrame::new("sssss\nssass\nsssss".to_string(), 1, None),
      AnimationFrame::new("ttttt\nttatt\nttttt".to_string(), 1, None),
    ];

    AnimationFrames::new(frames, AnimationLoopCount::Limited(loop_count))
  }
}
