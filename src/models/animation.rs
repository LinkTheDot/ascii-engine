use crate::errors::*;
use crate::models::animation_file_parser::*;
use crate::models::model_data::ModelData;
use crate::screen::screen_data::ScreenData;
pub use animation_frames::*;
use std::collections::{hash_map::Entry, HashMap, VecDeque};
use std::ffi::OsStr;
use std::fs::File;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub mod animation_frames;

pub struct ModelAnimationData {
  animations: HashMap<String, AnimationFrames>,
  animation_communicator: Option<mpsc::Sender<AnimationRequest>>,
}

#[derive(Debug)]
struct ModelAnimatorData {
  model: ModelData,
  animation_queue: VecDeque<AnimationFrames>,
  current_animation: Option<AnimationFrames>,
  current_animation_iteration_counter: u64,
  iteration_of_last_frame_change: u64,
}

pub struct AnimationConnection {
  pub handle: JoinHandle<()>,
  pub request_sender: mpsc::Sender<AnimationRequest>,
}

#[derive(Debug)]
pub struct AnimationRequest {
  model_unique_hash: u64,
  request: AnimationAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnimationAction {
  AddToQueue(AnimationFrames),
  OverwriteCurrentAnimation(AnimationFrames),
  ClearQueue,
  AddAnimator(ModelData),
  RemoveAnimatior,
}

impl ModelAnimatorData {
  fn new(model: ModelData) -> Self {
    Self {
      model,
      animation_queue: VecDeque::new(),
      current_animation: None,
      current_animation_iteration_counter: 0,
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
      let error_message = format!("A model has attempted to animate with an invalid frame. Model Hash: {}, Model Name: {}, Error: {error:?}", 
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
    self.current_animation_iteration_counter = 0;

    let new_animation = self.animation_queue.pop_front();

    if new_animation.is_none() {
      return Err(AnimationError::EmptyQueue);
    }

    self.current_animation = new_animation;

    Ok(())
  }

  /// Returns true if there are animations that can be run.
  ///
  /// This means that either there's already an animation running, or there's animations lined up in the queue.
  fn has_animations_to_run(&self) -> bool {
    !self.animation_queue.is_empty() || self.current_animation.is_some()
  }

  /// Replaces the currently running animation and replaces it with the one that was passed in.
  ///
  /// This method also restarts the ``current_animation_iteration_counter``.
  fn overwrite_current_animation(&mut self, new_animation: AnimationFrames) {
    self.current_animation_iteration_counter = 0;
    self.current_animation = Some(new_animation);
  }

  fn clear_queue(&mut self) {
    self.animation_queue.clear();
  }

  /// Assigns the passed in new animation if there's none currently running.
  /// Otherwise adds the new animation to the back of the queue.
  fn add_new_animation_to_queue(&mut self, new_animation: AnimationFrames) {
    if self.current_animation.is_none() {
      self.current_animation = Some(new_animation)
    } else {
      self.animation_queue.push_back(new_animation);
    }
  }

  /// Increments the counter for how many times this animation has changed the model's appearance.
  fn increment_frame_iteration_counter(&mut self) {
    self.current_animation_iteration_counter += 1;
  }

  fn frame_change_on_iteration(&mut self, current_iteration: u64) {
    self.iteration_of_last_frame_change = current_iteration;
  }
}

impl ModelAnimationData {
  pub fn from_file(animation_file_path: &std::path::Path) -> Result<Self, AnimationError> {
    if animation_file_path.extension() != Some(OsStr::new("animate")) {
      return Err(AnimationError::NonAnimationFile);
    }

    let animation_file = File::open(animation_file_path);

    match animation_file {
      Ok(file) => AnimationParser::parse(file),
      Err(_) => {
        let file_path = animation_file_path
          .file_name()
          // Unwrap and convert the OsStr to an OsString.
          .map(|path_string| path_string.to_owned());

        Err(AnimationError::AnimationFileDoesntExist(file_path))
      }
    }
  }

  /// Returns true if this animation_data has started it's animation thread.
  pub fn is_started(&self) -> bool {
    self.animation_communicator.is_some()
  }

  pub fn new() -> Self {
    Self {
      animations: HashMap::new(),
      animation_communicator: None,
    }
  }

  /// Starts the thread that will handle model animations
  ///
  // Describe how to send requests and whatnot
  ///
  /// # Errors
  ///
  /// - Returns an error when the model animation thread already exists.
  pub async fn start_animation_thread(
    screen_data: &ScreenData,
  ) -> Result<AnimationConnection, AnimationError> {
    if screen_data.animation_thread_already_started() {
      return Err(AnimationError::AnimationThreadAlreadyStarted);
    }

    // change this to be mpsc::unbounded_channel
    let (sender, mut receiver) = mpsc::channel::<AnimationRequest>(200);
    let event_sync = screen_data.get_event_sync();

    let animation_thread_handle = tokio::spawn(async move {
      let mut model_animator_data_list: HashMap<u64, ModelAnimatorData> = HashMap::new();

      (0..u64::MAX).for_each(|iteration| {
        event_sync.wait_for_tick();

        if Self::no_animators_are_running(&model_animator_data_list) {
          if let Some(request_data) = receiver.blocking_recv() {
            Self::run_model_request(&mut model_animator_data_list, request_data);
          }
        } else {
          while let Ok(request_data) = receiver.try_recv() {
            Self::run_model_request(&mut model_animator_data_list, request_data);
          }
        }

        model_animator_data_list
          .values_mut()
          .filter_map(|animation_data| {
            if animation_data.has_animations_to_run() {
              if animation_data.current_animation.is_none() {
                animation_data
                  .overwrite_current_animation_with_first_in_queue()
                  .unwrap();
              }

              return Some(animation_data);
            }

            None
          })
          .filter_map(|animator_data| {
            let animation_frames = animator_data.current_animation.as_ref().unwrap();
            let current_frame_index =
              animator_data.current_animation_iteration_counter % animation_frames.frame_count();
            let current_frame_tick_duration = animation_frames
              .get_frame(current_frame_index)
              .unwrap()
              .get_frame_duration();

            if animator_data.iteration_of_last_frame_change + current_frame_tick_duration as u64
              == iteration
            {
              if animation_frames
                .reached_loop_count(animator_data.current_animation_iteration_counter)
              {
                let _ = animator_data.overwrite_current_animation_with_first_in_queue();
              }

              animator_data.frame_change_on_iteration(iteration);

              return Some(animator_data);
            }

            None
          })
          // TODO Stop the user from making an animation with 0 frames, will cause a divide by 0.
          // TODO Force at least 1 tick wait times between frames when parsing the animation file.
          .for_each(|animator_data| {
            let animation_frames = animator_data.current_animation.as_ref().unwrap();
            let current_frame_index =
              animator_data.current_animation_iteration_counter % animation_frames.frame_count();

            let new_model_frame = animation_frames
              .get_frame(current_frame_index)
              .cloned()
              .unwrap();

            animator_data.change_model_frame(new_model_frame);
            animator_data.increment_frame_iteration_counter();
          });
      });
    });

    Ok(AnimationConnection {
      handle: animation_thread_handle,
      request_sender: sender,
    })
  }

  /// # Errors
  ///
  /// - An error is returned when the model hasn't started it's animations.
  pub async fn send_request(
    &mut self,
    model_hash: u64,
    request: AnimationAction,
  ) -> Result<(), AnimationError> {
    if let Some(animation_sender) = &self.animation_communicator {
      let animation_action_request = AnimationRequest {
        model_unique_hash: model_hash,
        request,
      };

      animation_sender
        .send(animation_action_request)
        .await
        .unwrap();
    } else {
      return Err(AnimationError::AnimationNotStarted);
    }

    Ok(())
  }

  /// # Errors
  ///
  /// - An error is returned when the given animation name doesn't exist in the list of animations.
  pub fn get_animation(&self, animation_name: String) -> Result<AnimationFrames, AnimationError> {
    let animation = self.animations.get(&animation_name);

    match animation {
      Some(animation_frames) => Ok(animation_frames.clone()),
      None => Err(AnimationError::AnimationDoesntExist),
    }
  }

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

  pub fn remove_animation_from_list(
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
  pub fn assign_communicator(
    &mut self,
    communicator: mpsc::Sender<AnimationRequest>,
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
  pub fn clone_sender(&self) -> mpsc::Sender<AnimationRequest> {
    self.request_sender.clone()
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

    #[tokio::test]
    async fn start_thread_once() {
      let screen_data = ScreenData::new();

      ModelAnimationData::start_animation_thread(&screen_data).await.unwrap();
    }
    
    #[tokio::test]
    #[should_panic]
    async fn start_thread_multiple_times() {
      let mut screen_data = ScreenData::new();

      screen_data.start_animation_thread().await.unwrap();
      ModelAnimationData::start_animation_thread(&screen_data).await.unwrap();
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
  mod model_animtor_methods {
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
        model_animator.current_animation = Some(animation_frames.clone());
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
        let overwrite_request = AnimationAction::OverwriteCurrentAnimation(replacing_animation_frames);

        let expected_frame_appearance = "ooooo\nooaoo\nooooo".to_string();
        
        let result = model_animator.run_request(overwrite_request);
        let frame_0_appearance = model_animator.current_animation
          .unwrap()
          .get_frame(0)
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
        assert!(model_animator.animation_queue.is_empty());
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
}
