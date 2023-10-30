use super::*;
use crate::models::model_appearance::sprites::*;
use crate::CONFIG;
use anyhow::*;
use core::result::Result::Ok;
use event_sync::EventSync;
use serde::{Deserialize, Serialize};
use std::{
  collections::{HashMap, VecDeque},
  time::Duration,
};

/// Handles the current running animations, when they started, and what the last run animation was.
#[derive(Default, Clone, Deserialize, Serialize)]
pub struct ModelAnimator {
  /// Contains the list of names of animations to be run.
  /// Should be push_back -> pop_front
  animation_queue: VecDeque<String>,
  /// Contains an EventSync for when the animation started.
  /// The tickrate contained is based on the tickrate in the config file.
  // #[serde(deserialize_with = "unpause_timer")] Think about this a bit more.
  current_animation_start: Option<EventSync>,
  last_run_animation: Option<String>,
}

impl ModelAnimator {
  /// If there's any animations running, returns
  pub fn get_current_model_appearance<'a>(
    &mut self,
    animation_list: &'a HashMap<String, AnimationFrames>,
  ) -> Option<&'a Sprite> {
    self.remove_missing_animations_from_queue(animation_list);

    if !self.has_animations_to_run() {
      return animation_list
        .get(self.get_last_run_animation()?)?
        .get_resting_appearance();
    }

    if let Err(error) = self.remove_finished_animations(animation_list) {
      log::error!("Animation Error: '{error}'");

      self.step_animation_queue(animation_list);
    }

    let ticks_since_start_of_animation = self
      .get_current_animation_start()?
      .ticks_since_started()
      .ok()?;

    let current_animation = animation_list.get(self.get_current_animation()?)?;
    let current_frame = current_animation.get_frame_based_on_ticks(ticks_since_start_of_animation);

    Some(current_frame?.get_appearance())
  }

  /// Returns true if there are animations that can be run.
  ///
  /// This means that either there's already an animation running, or there's animations lined up in the queue.
  pub fn has_animations_to_run(&self) -> bool {
    !self.animation_queue.is_empty()
  }

  /// Removes any animations that no longer exist in the animation_list, including the currently running one.
  fn remove_missing_animations_from_queue(
    &mut self,
    animation_list: &HashMap<String, AnimationFrames>,
  ) {
    if let Some(current_animation) = self.get_current_animation() {
      if !animation_list.contains_key(current_animation) {
        self.step_animation_queue(animation_list);
      }
    }

    self
      .animation_queue
      .retain(|name| animation_list.get(name).is_some());
  }

  /// Replaces the currently running animation and replaces it with the one that was passed in.
  pub fn overwrite_current_animation(&mut self, new_animation: String) {
    if let Some(last_run_animation) = self.animation_queue.pop_front() {
      self.last_run_animation = Some(last_run_animation);
    }

    self.animation_queue.push_back(new_animation);

    self.restart_animation_start();
  }

  /// Removes an animation from the front of the queue, updating the last run animation and start of the new animation.
  ///
  /// Should this method be called when the start of the animation is greater than the duration of the current animation.
  /// The EventSync will be updated accordingly.
  pub fn step_animation_queue(&mut self, animation_list: &HashMap<String, AnimationFrames>) {
    if let Some(remaining_time) = self.get_remaining_duration_of_current_animation(animation_list) {
      self.restart_animation_start_with_remaining_time(remaining_time);
    } else {
      self.restart_animation_start();
    }

    if let Some(last_run_animation) = self.animation_queue.pop_front() {
      if animation_list.contains_key(&last_run_animation) {
        self.last_run_animation = Some(last_run_animation);
      }
    }

    if self.animation_queue.is_empty() {
      self.current_animation_start = None;
    }
  }

  /// If the current animation is finished, returns Some(remaining_duration).
  /// Meaning, if the current animation lasts 3 ticks, and 4.1 ticks have passed since the start of the animation.
  /// A Duration of 1.1 ticks is returned.
  /// This can be used to recreate the EventSync for the start of the next animation in the queue.
  ///
  /// None is returned if the EventSync should be reset to 0 for any reason.
  /// None does *NOT* mean the current animation is or isn't finished. If you want to check that use [`ModelAnimator.current_animation_is_finished()`](ModelAnimator::current_animation_is_finished)
  fn get_remaining_duration_of_current_animation(
    &self,
    animation_list: &HashMap<String, AnimationFrames>,
  ) -> Option<Duration> {
    let animation_start = self.get_current_animation_start()?;
    let current_animation = animation_list.get(self.get_current_animation()?)?;
    let duration_of_last_run_animation = current_animation.get_total_duration()?;
    let ticks_since_started = animation_start.ticks_since_started().ok()?;

    if ticks_since_started > duration_of_last_run_animation {
      let remainder_time = animation_start.time_since_last_tick().ok()?;

      Some(
        Duration::from_millis(duration_of_last_run_animation * CONFIG.tick_duration as u64)
          + remainder_time,
      )
    } else {
      None
    }
  }

  /// Gets a reference to the current animation's name.
  fn get_current_animation(&self) -> Option<&str> {
    self.animation_queue.front().map(String::as_str)
  }

  /// Gets a reference to the current animation's EventSync
  fn get_current_animation_start(&self) -> Option<&EventSync> {
    self.current_animation_start.as_ref()
  }

  /// Gets a reference to the last run animation.
  fn get_last_run_animation(&self) -> Option<&str> {
    self.last_run_animation.as_deref()
  }

  /// Removes every animation running in the queue, adding the last existing animation that *would* have been run.
  pub fn clear_queue(&mut self, animation_list: &HashMap<String, AnimationFrames>) {
    if let Some(last_run_animation) = self
      .animation_queue
      .iter()
      .rev()
      .find(|animation_name| animation_list.contains_key(*animation_name))
      .map(String::to_owned)
    {
      self.last_run_animation = Some(last_run_animation);
    }

    self.current_animation_start = None;
  }

  /// Assigns the passed in new animation if there's none currently running.
  /// Otherwise adds the new animation to the back of the queue.
  pub fn add_new_animation_to_queue(&mut self, new_animation: String) {
    if self.get_current_animation().is_none() {
      self.overwrite_current_animation(new_animation);
    } else {
      self.animation_queue.push_back(new_animation);
    }
  }

  /// Restarts the currently stored EventSync keeping track of the current animation's start.
  ///
  /// If there are no animations currently running, assigns current_animation_start to None.
  fn restart_animation_start(&mut self) {
    self.current_animation_start = if self.has_animations_to_run() {
      Some(EventSync::new(CONFIG.tick_duration))
    } else {
      None
    }
  }

  fn restart_animation_start_with_remaining_time(&mut self, remaining_time: Duration) {
    self.current_animation_start = if self.has_animations_to_run() {
      Some(EventSync::from_starting_time(
        CONFIG.tick_duration,
        remaining_time,
      ))
    } else {
      None
    }
  }

  fn remove_finished_animations(
    &mut self,
    animation_list: &HashMap<String, AnimationFrames>,
  ) -> anyhow::Result<()> {
    let mut last_run_animation = None;

    if !self.current_animation_is_finished(animation_list)? {
      return Ok(());
    }

    let mut remaining_duration = self
      .get_current_animation_start()
      .ok_or(anyhow!("No animation start."))?
      .ticks_since_started()?;

    self
      .animation_queue
      .iter()
      .flat_map(|name| animation_list.get(name))
      // Collecting because a reference to the animation_queue is still technically held for some reason.
      .collect::<Vec<&AnimationFrames>>()
      .into_iter()
      .try_for_each(|animation| {
        let Some(duration_of_animation) = animation.get_total_duration() else {
          remaining_duration = 0;

          return None; // Animation doesn't end.
        };

        if remaining_duration >= duration_of_animation {
          remaining_duration -= duration_of_animation;

          last_run_animation = self.animation_queue.pop_front();

          Some(())
        } else {
          None
        }
      });

    let remainder_time = self
      .get_current_animation_start()
      .ok_or(anyhow!("No animation start."))?
      .time_since_last_tick()?;
    let remaining_duration =
      Duration::from_millis(remaining_duration * CONFIG.tick_duration as u64) + remainder_time;

    self.restart_animation_start_with_remaining_time(remaining_duration);

    if last_run_animation.is_some() {
      self.last_run_animation = last_run_animation;
    }

    Ok(())
  }

  /// Returns true if the current animation should be removed from the list either because it no longer exists,
  /// or the time that it's been running has exceeded its duration.
  fn current_animation_is_finished(
    &self,
    animation_list: &HashMap<String, AnimationFrames>,
  ) -> anyhow::Result<bool> {
    let Some(current_animation) = self.get_current_animation() else {
      return Ok(true);
    };
    let Some(current_animation) = animation_list.get(current_animation) else {
      return Err(anyhow!("Current running animation is missing"));
    };
    let Some(animation_duration) = current_animation.get_total_duration() else {
      return Ok(false); // Infinite animation duration.
    };

    let current_animation_start = self
      .get_current_animation_start()
      .ok_or(anyhow!("No animation start."))?;
    let ticks_since_animation_start = current_animation_start.ticks_since_started()?;

    Ok(ticks_since_animation_start >= animation_duration)
  }

  //   pub fn set_to_resting_frame(&self) -> Result<(), AnimationError> {
  //     // let Some(current_animation) = self.current_animation else {
  //     //   return Err(AnimationError::NoExistingAnimation);
  //     // };
  //     //
  //     // let Some(resting_frame) = current_animation.get_resting_appearance() else {
  //     //   return Err(AnimationError::AnimationHasNoRestingFrame);
  //     // };
  //     //
  //     // self.change_model_frame(resting_frame);
  //     //
  //     Ok(())
  //   }
}

impl std::fmt::Debug for ModelAnimator {
  fn fmt(
    &self,
    formatter: &mut std::fmt::Formatter<'_>,
  ) -> std::result::Result<(), std::fmt::Error> {
    let animation_start = self
      .current_animation_start
      .as_ref()
      .map(|e| e.time_since_started());

    formatter
      .debug_struct("ModelAnimator")
      .field("animation_queue", &self.animation_queue)
      .field("time_since_start_of_animation", &animation_start)
      .field("last_run_animation", &self.last_run_animation)
      .finish()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::testing_data::*;

  #[cfg(test)]
  mod add_new_animation_to_queue {
    use super::*;

    #[test]
    fn empty_queue() {
      let animation_list = get_test_animation_list();
      let animation_name = animation_list.keys().next().unwrap().to_owned();
      let mut model_animator = ModelAnimator::default();

      let expected_frame = animation_list.get(&animation_name).unwrap().get_frame(0);
      let expected_sprite = expected_frame.unwrap().get_appearance();

      model_animator.add_new_animation_to_queue(animation_name.clone());

      let current_sprite = model_animator
        .get_current_model_appearance(&animation_list)
        .unwrap();

      // Check state of fields.
      assert!(model_animator.last_run_animation.is_none());
      assert!(model_animator.current_animation_start.is_some());
      assert!(model_animator.animation_queue.len() == 1);
      // Check if the actual data is what's expected.
      assert_eq!(expected_sprite, current_sprite);
    }

    #[test]
    fn existing_queue() {
      let animation_list = get_test_animation_list();
      let animation_names: Vec<String> = animation_list.keys().map(|k| k.to_owned()).collect();
      let mut model_animator = ModelAnimator::default();

      let expected_frames: Vec<&Sprite> = animation_list
        .values()
        .map(|frames| frames.get_frame(0).unwrap().get_appearance())
        .collect();

      // Add the animations to the queue
      model_animator.add_new_animation_to_queue(animation_names[0].clone());
      model_animator.add_new_animation_to_queue(animation_names[1].clone());
      model_animator.add_new_animation_to_queue(animation_names[2].clone());

      // Get the first frame of each animation in the queues in order.
      let first_frames_of_animations: Vec<&Sprite> = (0..3)
        .map(|_| {
          let frame = model_animator
            .get_current_model_appearance(&animation_list)
            .unwrap();

          model_animator.step_animation_queue(&animation_list);

          frame
        })
        .collect();

      println!("{:?}", model_animator);

      // Check state of fields.
      assert!(model_animator.current_animation_start.is_none());
      assert!(model_animator.animation_queue.is_empty());
      assert_eq!(
        model_animator.last_run_animation,
        animation_names.get(2).cloned()
      );
      // Check if the actual data is what's expected.
      assert_eq!(first_frames_of_animations, expected_frames);
    }
  }

  #[cfg(test)]
  mod overwrite_current_animation_with_first_in_queue_logic {
    use super::*;

    #[test]
    fn empty_queue_running_animation() {
      let animation_list = get_test_animation_list();
      let animation_name = animation_list.keys().next().unwrap().to_owned();
      let mut model_animator = ModelAnimator::default();

      model_animator.add_new_animation_to_queue(animation_name.clone());

      model_animator.step_animation_queue(&animation_list);

      assert!(model_animator.current_animation_start.is_none());
      assert!(model_animator.animation_queue.is_empty());
      assert_eq!(model_animator.last_run_animation, Some(animation_name));
    }

    #[test]
    fn empty_queue_no_running_animation() {
      let animation_list = HashMap::new();
      let mut model_animator = ModelAnimator::default();

      model_animator.step_animation_queue(&animation_list);

      assert!(model_animator.current_animation_start.is_none());
      assert!(model_animator.animation_queue.is_empty());
      assert!(model_animator.last_run_animation.is_none());
    }

    #[test]
    fn queue_has_contents_while_running_animation() {
      let animation_list = get_test_animation_list();
      let animation_names: Vec<String> = animation_list.keys().map(|k| k.to_owned()).collect();
      let mut model_animator = ModelAnimator::default();

      // Add the animations to the queue
      model_animator.add_new_animation_to_queue(animation_names[0].clone());
      model_animator.add_new_animation_to_queue(animation_names[1].clone());
      model_animator.add_new_animation_to_queue(animation_names[2].clone());

      // Get the first frame of each animation in the queues in order.
      let running_animation_names: Vec<String> = (0..3)
        .map(|_| {
          let current_animation = model_animator
            .get_current_animation()
            .map(String::from)
            .unwrap();

          model_animator.step_animation_queue(&animation_list);

          current_animation
        })
        .collect();

      println!("{:?}", model_animator);

      // Check state of fields.
      assert!(model_animator.current_animation_start.is_none());
      assert!(model_animator.animation_queue.is_empty());
      assert_eq!(
        model_animator.last_run_animation,
        animation_names.get(2).cloned()
      );
      // Check if the actual data is what's expected.
      assert_eq!(running_animation_names, animation_names);
    }
  }

  #[cfg(test)]
  mod has_animation_to_run_logic {
    use super::*;

    #[test]
    fn has_animations() {
      let animation_list = get_test_animation_list();
      let animation_names: Vec<String> = animation_list.keys().map(|k| k.to_owned()).collect();
      let mut model_animator = ModelAnimator::default();

      model_animator.add_new_animation_to_queue(animation_names[0].clone());
      model_animator.add_new_animation_to_queue(animation_names[1].clone());

      let queue_and_current_result = model_animator.has_animations_to_run();

      // has current animation but none in queue
      model_animator.step_animation_queue(&animation_list);

      let empty_queue_and_current_result = model_animator.has_animations_to_run();

      assert!(queue_and_current_result);
      assert!(empty_queue_and_current_result);
    }

    #[test]
    fn has_no_animations() {
      let model_animator = ModelAnimator::default();

      assert!(!model_animator.has_animations_to_run());
    }
  }

  #[test]
  fn overwrite_current_animation_logic() {
    let animation_list = get_test_animation_list();
    let animation_names: Vec<String> = animation_list.keys().map(|k| k.to_owned()).collect();
    let mut model_animator = ModelAnimator::default();

    let expected_frames: Vec<&Sprite> = animation_list
      .values()
      .map(|frames| frames.get_frame(0).unwrap().get_appearance())
      .collect();

    model_animator.add_new_animation_to_queue(animation_names[0].clone());

    let first_frame_before = model_animator
      .get_current_model_appearance(&animation_list)
      .unwrap();

    model_animator.overwrite_current_animation(animation_names[1].clone());

    let first_frame_after = model_animator
      .get_current_model_appearance(&animation_list)
      .unwrap();

    assert_eq!(first_frame_before, expected_frames[0]);
    assert_eq!(first_frame_after, expected_frames[1]);
  }

  #[cfg(test)]
  mod get_current_model_appearance_logic {
    use super::*;

    // Tests if correct frames are being returned after x amount of time has passed.
    // Also checks if the state of the ModelAnimator is what's to be expected.
    #[test]
    fn frame_time_logic() {
      let animation_list = get_test_animation_list();
      let animation_name = "TestOne".to_string();
      let mut model_animator = ModelAnimator::default();
      let event_sync = EventSync::new(CONFIG.tick_duration);

      model_animator.add_new_animation_to_queue(animation_name.clone());

      let expected_animation =
        TestingData::get_test_animation(['x', 'y', 'z'], AnimationLoopCount::Limited(1));
      let expected_frames: Vec<&Sprite> = expected_animation
        .get_frames()
        .iter()
        .map(AnimationFrame::get_appearance)
        .collect();

      // Run all 3 frames in the animation
      let frames: Vec<&Sprite> = (0..3)
        .map(|_| {
          let frame_appearance = model_animator
            .get_current_model_appearance(&animation_list)
            .unwrap();

          event_sync.wait_for_tick().unwrap();

          frame_appearance
        })
        .collect();

      // The animation running is finished after 3 ticks
      assert!(model_animator
        .get_current_model_appearance(&animation_list)
        .is_none());
      assert_eq!(frames, expected_frames);
      assert!(
        model_animator.current_animation_start.is_none()
          && model_animator.animation_queue.is_empty(),
        "{:#?}",
        model_animator
      );
      assert_eq!(
        model_animator.last_run_animation,
        Some(animation_name),
        "{model_animator:#?}"
      );
    }

    // This tests to see if get_current_model_appearance() is correctly removing the current animation
    // when it's finished, and is replacing it with the first in queue.
    #[test]
    fn current_animation_is_finished_with_items_in_queue() {
      let animation_list = get_test_animation_list();
      let animation_names: Vec<String> = animation_list.keys().map(|k| k.to_owned()).collect();
      let mut model_animator = ModelAnimator::default();
      let event_sync = EventSync::new(CONFIG.tick_duration);

      let expected_animation = animation_list.get(&animation_names[1]).unwrap().clone();
      let expected_frames: Vec<&Sprite> = expected_animation
        .get_frames()
        .iter()
        .map(AnimationFrame::get_appearance)
        .collect();

      model_animator.add_new_animation_to_queue(animation_names[0].clone());
      model_animator.add_new_animation_to_queue(animation_names[1].clone());
      // Wait until the first animation is finished.
      event_sync.wait_for_x_ticks(3).unwrap();

      // Run all 3 frames in the animation
      let frames: Vec<&Sprite> = (0..3)
        .map(|_| {
          let frame_appearance = model_animator
            .get_current_model_appearance(&animation_list)
            .unwrap();

          event_sync.wait_for_tick().unwrap();

          frame_appearance
        })
        .collect();

      event_sync.wait_for_tick().unwrap();

      // The animation running is finished after 3 ticks

      assert!(model_animator
        .get_current_model_appearance(&animation_list)
        .is_none());
      assert_eq!(frames, expected_frames);
      assert!(
        model_animator.current_animation_start.is_none()
          && model_animator.animation_queue.is_empty(),
        "{:#?}",
        model_animator
      );
      assert_eq!(
        model_animator.last_run_animation,
        Some(animation_names[1].clone())
      );
    }

    // This tests if get_current_model_appearance() is skipping animations in the queue if they've finished
    // since having been added to the queue.
    #[test]
    fn skip_finished_animation_in_queue() {}
  }

  #[test]
  fn clean_animation_queue_logic() {
    let mut animation_list = get_test_animation_list();
    let animation_names: Vec<String> = animation_list.keys().map(|k| k.to_owned()).collect();
    let name_one = animation_names[0].clone();
    let name_two = animation_names[1].clone();
    let mut model_animator = ModelAnimator::default();

    model_animator.add_new_animation_to_queue(name_one.clone());
    model_animator.add_new_animation_to_queue(name_two.clone());

    animation_list.remove(&name_one);
    model_animator.remove_missing_animations_from_queue(&animation_list);

    assert_eq!(
      model_animator.get_current_animation(),
      Some(name_two.as_str())
    );
    assert!(model_animator.animation_queue.len() == 1);

    animation_list.remove(&name_two);
    model_animator.remove_missing_animations_from_queue(&animation_list);

    assert!(model_animator.animation_queue.is_empty());
  }

  #[test]
  fn start_time_is_resetting() {
    let animation_list = get_test_animation_list();
    let animation_names: Vec<String> = animation_list.keys().map(|k| k.to_owned()).collect();
    let mut model_animator = ModelAnimator::default();
    let event_sync = EventSync::new(CONFIG.tick_duration);

    model_animator.add_new_animation_to_queue(animation_names[0].clone());
    model_animator.add_new_animation_to_queue(animation_names[1].clone());

    event_sync.wait_for_tick().unwrap();

    // Check that the event_sync exists.
    assert_eq!(
      model_animator
        .current_animation_start
        .as_ref()
        .unwrap()
        .ticks_since_started(),
      Ok(1)
    );

    model_animator.step_animation_queue(&animation_list);

    // Check that the event_sync has been reset with a new animation.
    assert_eq!(
      model_animator
        .current_animation_start
        .as_ref()
        .unwrap()
        .ticks_since_started(),
      Ok(0)
    );

    model_animator.step_animation_queue(&animation_list);

    // Check that the evnet_sync is removes now that there are no animations.
    assert!(model_animator.current_animation_start.is_none());
  }

  // data for tests

  fn get_test_animation_list() -> HashMap<String, AnimationFrames> {
    let animations = [
      (
        "TestOne".to_string(),
        TestingData::get_test_animation(['x', 'y', 'z'], AnimationLoopCount::Limited(1)),
      ),
      (
        "TestTwo".to_string(),
        TestingData::get_test_animation(['-', 'b', 'c'], AnimationLoopCount::Limited(1)),
      ),
      (
        "TestThree".to_string(),
        TestingData::get_test_animation(['l', 'm', 'n'], AnimationLoopCount::Limited(1)),
      ),
    ];

    HashMap::from(animations)
  }
}
