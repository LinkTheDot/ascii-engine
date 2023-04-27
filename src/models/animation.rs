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
  fn increment_frame_iteration_counter(&mut self) -> Result<(), AnimationError> {
    if self.current_animation.is_some() {
      self.current_animation_iteration_counter += 1;
    } else {
      return Err(AnimationError::NoExistingAnimation);
    }

    Ok(())
  }

  /// Replaces the internal value for the last iteration a frame was changed on.
  fn update_when_last_frame_changed(&mut self, current_iteration: u64) {
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

              animator_data.update_when_last_frame_changed(iteration);

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
            // There shouldn't be any way for this to panic.
            animator_data.increment_frame_iteration_counter().unwrap();
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

      ModelAnimationData::start_animation_thread(&screen_data)
        .await
        .unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn start_thread_multiple_times() {
      let mut screen_data = ScreenData::new();

      screen_data.start_animation_thread().await.unwrap();
      ModelAnimationData::start_animation_thread(&screen_data)
        .await
        .unwrap();
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
        let overwrite_request =
          AnimationAction::OverwriteCurrentAnimation(replacing_animation_frames);

        let expected_frame_appearance = "ooooo\nooaoo\nooooo".to_string();

        let result = model_animator.run_request(overwrite_request);
        let frame_0_appearance = model_animator
          .current_animation
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
        let mut model_animator = ModelAnimatorData::new(model_data.clone());
        let frame_appearance = "--\n--a--\n-----".to_string();
        let new_frame = AnimationFrame::new(frame_appearance, 1, Some('-'));

        model_animator.change_model_frame(new_frame);

        println!("{}", model_data.get_sprite());
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
        model_animator.current_animation = Some(running_animation.clone());

        let expected_animation = Some(running_animation);

        let result = model_animator.overwrite_current_animation_with_first_in_queue();
        let current_animation = &model_animator.current_animation;

        assert!(result.is_err());
        assert_eq!(current_animation, &expected_animation);
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
        model_animator.current_animation = Some(running_animation);
        model_animator
          .animation_queue
          .push_front(queued_animation.clone());

        let expected_animation = Some(queued_animation);

        let result = model_animator.overwrite_current_animation_with_first_in_queue();
        let current_animation = &model_animator.current_animation;

        assert!(result.is_ok());
        assert_eq!(current_animation, &expected_animation);
      }

      #[test]
      fn queue_has_contents_no_running_animation() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);
        let queued_animation = get_test_animation_limited_run_count();
        model_animator
          .animation_queue
          .push_front(queued_animation.clone());

        let expected_animation = Some(queued_animation);

        let result = model_animator.overwrite_current_animation_with_first_in_queue();
        let current_animation = &model_animator.current_animation;

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
        model_animator.overwrite_current_animation_with_first_in_queue().unwrap();

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

      let animation_before = model_animator.current_animation.clone();

      model_animator.overwrite_current_animation(replacing_animation.clone());

      let animation_after = model_animator.current_animation.clone();

      assert_eq!(animation_before, Some(running_animation));
      assert_eq!(animation_after, Some(replacing_animation));
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

        let current_animation = model_animator.current_animation.clone();

        assert_eq!(current_animation, Some(animation));
        assert!(model_animator.animation_queue.is_empty());
      }

      #[test]
      fn existing_queue() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);
        let animation_one = get_test_animation_limited_run_count();
        let animation_two = get_test_animation_unlimited_run_count();
        let animation_three = get_test_animation();

        model_animator.add_new_animation_to_queue(animation_one.clone());
        model_animator.add_new_animation_to_queue(animation_two.clone());
        model_animator.add_new_animation_to_queue(animation_three.clone());

        let current_animation_one = model_animator.current_animation.clone();

        model_animator.overwrite_current_animation_with_first_in_queue().unwrap();
        let current_animation_two = model_animator.current_animation.clone();

        model_animator.overwrite_current_animation_with_first_in_queue().unwrap();
        let current_animation_three = model_animator.current_animation.clone();

        assert_eq!(current_animation_one, Some(animation_one));
        assert_eq!(current_animation_two, Some(animation_two));
        assert_eq!(current_animation_three, Some(animation_three));
      }
    }

    #[cfg(test)]
    mod increment_frame_iteration_counter_logic {
      use super::*;

      #[test]
      fn existing_animation() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);
        let animation = get_test_animation_limited_run_count();
        model_animator.add_new_animation_to_queue(animation);

        let expected_before_frame_counter = 0;
        let expected_after_frame_counter = 1;

        let before_frame_counter = model_animator.current_animation_iteration_counter;

        model_animator.increment_frame_iteration_counter().unwrap();

        let after_frame_counter = model_animator.current_animation_iteration_counter;

        assert_eq!(before_frame_counter, expected_before_frame_counter);
        assert_eq!(after_frame_counter, expected_after_frame_counter);
      }

      #[test]
      fn no_current_animation() {
        let model_data = get_test_model_data();
        let mut model_animator = ModelAnimatorData::new(model_data);

        let expected_result = Err(AnimationError::NoExistingAnimation);
        let expected_before_frame_counter = 0;
        let expected_after_frame_counter = 0;

        let before_frame_counter = model_animator.current_animation_iteration_counter;

        let result = model_animator.increment_frame_iteration_counter();

        let after_frame_counter = model_animator.current_animation_iteration_counter;

        assert_eq!(result, expected_result);
        assert_eq!(before_frame_counter, expected_before_frame_counter);
        assert_eq!(after_frame_counter, expected_after_frame_counter);
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
  fn get_test_animation() -> AnimationFrames {
    let frames = vec![
      AnimationFrame::new("rrrrr\nrrarr\nrrrrr".to_string(), 1, None),
      AnimationFrame::new("sssss\nssass\nsssss".to_string(), 1, None),
      AnimationFrame::new("ttttt\nttatt\nttttt".to_string(), 1, None),
    ];

    AnimationFrames::new(frames, AnimationLoopCount::Limited(3))
  }
}
