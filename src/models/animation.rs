use crate::errors::*;
use crate::models::animation_file_parser::*;
use crate::models::model_data::ModelData;
pub use animation_frames::*;
use event_sync::EventSync;
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
    event_sync: EventSync,
  ) -> Result<AnimationConnection, AnimationError> {
    let (sender, mut receiver) = mpsc::channel::<AnimationRequest>(200);

    let animation_thread_handle = tokio::spawn(async move {
      let mut model_animator_data_list: HashMap<u64, ModelAnimatorData> = HashMap::new();

      // possibly add a way where if all lists are empty (aka nothing is happening), just wait for requests
      (0..u64::MAX).for_each(|iteration| {
        event_sync.wait_for_tick();

        while let Ok(request_data) = receiver.try_recv() {
          if let Some(called_model_animator) =
            model_animator_data_list.get_mut(&request_data.model_unique_hash)
          {
            if called_model_animator.run_request(request_data.request) {
              model_animator_data_list.remove(&request_data.model_unique_hash);
            }
          } else if let AnimationAction::AddAnimator(add_model) = request_data.request {
            model_animator_data_list.insert(
              request_data.model_unique_hash,
              ModelAnimatorData::new(add_model),
            );
          } else {
            log::warn!("Attempted to call an animation request with an invalid model hash");
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
