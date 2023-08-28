use super::*;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

/// Holds a reference to a model's Sprite, the queue of animations to be run,
/// the current animation that is running, and the list iteration the frame was updated.
///
/// A reference to this will be held by each animated model and the animation thread.
#[derive(Debug)]
pub struct ModelAnimator {
  model_sprite: Arc<RwLock<Sprite>>,
  animation_queue: VecDeque<AnimationFrames>,
  current_animation: Option<AnimationFramesIntoIter>,
  iteration_of_last_frame_change: u64,
}

impl ModelAnimator {
  pub fn new(model_sprite: Arc<RwLock<Sprite>>) -> Arc<Mutex<Self>> {
    Arc::new(Mutex::new(Self {
      model_sprite,
      animation_queue: VecDeque::new(),
      current_animation: None,
      iteration_of_last_frame_change: 0,
    }))
  }

  /// Replaces the appearance of the model with the given frame.
  ///
  /// Logs an error if the passed in frame contains an invalid sprite.
  pub fn change_model_frame(&mut self, new_frame: AnimationFrame) {
    let mut new_appearance = new_frame.get_appearance().to_owned();

    if let Err(error_list) = new_appearance.validity_check() {
      log::error!(
        "A model produced the following errors when replacing a frame during animation: {}",
        error_list
          .into_iter()
          .map(|err| format!("{err:?}"))
          .collect::<Vec<String>>()
          .join(" | ")
      );
    }

    std::mem::swap(
      &mut *self.model_sprite.write().unwrap(),
      &mut new_appearance,
    );
  }

  /// Returns an error if the queue was empty.
  ///
  /// Removes an animation from the front of the queue and assigns it to the currently looping animation.
  ///
  /// This method also restarts the ``current_animation_iteration_counter``.
  pub fn overwrite_current_animation_with_first_in_queue(&mut self) -> Result<(), AnimationError> {
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
  pub fn has_animations_to_run(&self) -> bool {
    self.has_animations_queued() || self.is_running_an_animation()
  }

  /// Returns true if there's any animations currently in the queue.
  pub fn has_animations_queued(&self) -> bool {
    !self.animation_queue.is_empty()
  }

  /// Returns true if there is a currently running animation.
  pub fn is_running_an_animation(&self) -> bool {
    self.current_animation.is_some()
  }

  /// Replaces the currently running animation and replaces it with the one that was passed in.
  ///
  /// This method also restarts the ``current_animation_iteration_counter``.
  pub fn overwrite_current_animation(&mut self, new_animation: AnimationFrames) {
    self.current_animation = Some(new_animation.into_iter());
  }

  pub fn clear_queue(&mut self) {
    self.animation_queue.clear();
  }

  /// Assigns the passed in new animation if there's none currently running.
  /// Otherwise adds the new animation to the back of the queue.
  pub fn add_new_animation_to_queue(&mut self, new_animation: AnimationFrames) {
    if self.current_animation.is_none() {
      self.current_animation = Some(new_animation.into_iter())
    } else {
      self.animation_queue.push_back(new_animation);
    }
  }

  /// Replaces the internal value for the last iteration a frame was changed on.
  pub fn update_when_last_frame_changed(&mut self, current_iteration: u64) {
    self.iteration_of_last_frame_change = current_iteration;
  }

  /// Gets the next animation frame and returns it.
  ///
  /// None is returned if there is no currently assigned animation.
  pub fn next_frame(&mut self) -> Option<AnimationFrame> {
    self.current_animation.as_mut()?.next()
  }

  /// Returns None if there is no current animation
  pub fn get_current_animation(&self) -> Option<&AnimationFramesIntoIter> {
    self.current_animation.as_ref()
  }

  /// Returns None if there is no current animation.
  pub fn get_current_animation_frame_duration(&self) -> Option<u64> {
    self.get_current_animation()?.current_frame_duration()
  }

  /// Returns true if the current frame's duration is over.
  ///
  /// If there is no currently running animation `false` is returned.
  pub fn current_frame_duration_is_finished(&self, current_iteration: u64) -> bool {
    if let Some(current_frame_tick_duration) = self.get_current_animation_frame_duration() {
      self.iteration_of_last_frame_change + current_frame_tick_duration <= current_iteration
    } else {
      true
    }
  }

  /// Returns the last time the animator updated the model's appearance.
  ///
  /// Based on the iteration
  pub fn get_iteration_of_last_frame_change(&self) -> u64 {
    self.iteration_of_last_frame_change
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[cfg(test)]
  mod add_new_animation_to_queue {
    use super::*;

    #[test]
    fn empty_queue() {
      let mut model_data = get_test_model_data();
      let model_sprite = model_data.get_sprite();
      let model_animator = ModelAnimator::new(model_sprite);
      let mut model_animator = model_animator.lock().unwrap();
      let animation = get_test_animation_limited_run_count();

      model_animator.add_new_animation_to_queue(animation.clone());

      let current_animation = model_animator.next_frame();

      assert_eq!(current_animation, animation.get_frame(0).cloned());
      assert!(model_animator.animation_queue.is_empty());
    }

    #[test]
    fn existing_queue() {
      let mut model_data = get_test_model_data();
      let model_sprite = model_data.get_sprite();
      let model_animator = ModelAnimator::new(model_sprite);
      let mut model_animator = model_animator.lock().unwrap();
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

  #[cfg(test)]
  mod overwrite_current_animation_with_first_in_queue_logic {
    use super::*;

    #[test]
    fn empty_queue_running_animation() {
      let mut model_data = get_test_model_data();
      let model_sprite = model_data.get_sprite();
      let model_animator = ModelAnimator::new(model_sprite);
      let mut model_animator = model_animator.lock().unwrap();
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
      let mut model_data = get_test_model_data();
      let model_sprite = model_data.get_sprite();
      let model_animator = ModelAnimator::new(model_sprite);
      let mut model_animator = model_animator.lock().unwrap();

      let result = model_animator.overwrite_current_animation_with_first_in_queue();

      assert!(result.is_err());
      assert!(model_animator.current_animation.is_none());
    }

    #[test]
    fn queue_has_contents_running_animation() {
      let mut model_data = get_test_model_data();
      let model_sprite = model_data.get_sprite();
      let model_animator = ModelAnimator::new(model_sprite);
      let mut model_animator = model_animator.lock().unwrap();
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
      let mut model_data = get_test_model_data();
      let model_sprite = model_data.get_sprite();
      let model_animator = ModelAnimator::new(model_sprite);
      let mut model_animator = model_animator.lock().unwrap();
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
      let mut model_data = get_test_model_data();
      let model_sprite = model_data.get_sprite();
      let model_animator = ModelAnimator::new(model_sprite);
      let mut model_animator = model_animator.lock().unwrap();
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
      let mut model_data = get_test_model_data();
      let model_sprite = model_data.get_sprite();
      let model_animator = ModelAnimator::new(model_sprite);
      let model_animator = model_animator.lock().unwrap();

      assert!(!model_animator.has_animations_to_run());
    }
  }

  #[test]
  fn overwrite_current_animation_logic() {
    let mut model_data = get_test_model_data();
    let model_sprite = model_data.get_sprite();
    let model_animator = ModelAnimator::new(model_sprite);
    let mut model_animator = model_animator.lock().unwrap();
    let running_animation = get_test_animation_limited_run_count();
    let replacing_animation = get_test_animation_unlimited_run_count();
    model_animator.add_new_animation_to_queue(running_animation.clone());

    let first_frame_before = model_animator.next_frame();

    model_animator.overwrite_current_animation(replacing_animation.clone());

    let first_frame_after = model_animator.next_frame();

    assert_eq!(first_frame_before, running_animation.get_frame(0).cloned());
    assert_eq!(first_frame_after, replacing_animation.get_frame(0).cloned());
  }

  #[test]
  fn update_when_last_frame_changed_logic() {
    let mut model_data = get_test_model_data();
    let model_sprite = model_data.get_sprite();
    let model_animator = ModelAnimator::new(model_sprite);
    let mut model_animator = model_animator.lock().unwrap();

    let before_last_frame_change = model_animator.get_iteration_of_last_frame_change();

    model_animator.update_when_last_frame_changed(10);

    let after_last_frame_change = model_animator.get_iteration_of_last_frame_change();

    assert_eq!(before_last_frame_change, 0);
    assert_eq!(after_last_frame_change, 10);
  }

  #[cfg(test)]
  mod get_current_animation_logic {
    use super::*;

    #[test]
    fn running_animation() {
      let mut model_data = get_test_model_data();
      let model_sprite = model_data.get_sprite();
      let model_animator = ModelAnimator::new(model_sprite);
      let model_animation = get_test_animation(3);
      let mut model_animator = model_animator.lock().unwrap();

      model_animator.add_new_animation_to_queue(model_animation.clone());

      let expected_first_frame = model_animation.get_frame(0).cloned();

      let current_animation = model_animator.get_current_animation().unwrap();
      let first_frame_of_current = current_animation.get_current_frame();

      assert_eq!(first_frame_of_current, expected_first_frame);
    }

    #[test]
    fn no_running_animation() {
      let mut model_data = get_test_model_data();
      let model_sprite = model_data.get_sprite();
      let model_animator = ModelAnimator::new(model_sprite);
      let model_animator = model_animator.lock().unwrap();

      let current_animation = model_animator.get_current_animation();

      assert!(current_animation.is_none());
    }
  }

  #[cfg(test)]
  mod current_frame_duration_is_finished_logic {
    use super::*;

    #[test]
    fn running_frame_not_finished() {
      let mut model_data = get_test_model_data();
      let model_sprite = model_data.get_sprite();
      let model_animator = ModelAnimator::new(model_sprite);
      let model_animation = get_test_animation(1);
      let mut model_animator_guard = model_animator.lock().unwrap();

      model_animator_guard.add_new_animation_to_queue(model_animation);

      assert!(!model_animator_guard.current_frame_duration_is_finished(0));
    }

    #[test]
    fn running_frame_finished() {
      let mut model_data = get_test_model_data();
      let model_sprite = model_data.get_sprite();
      let model_animator = ModelAnimator::new(model_sprite);
      let mut model_animator = model_animator.lock().unwrap();
      let model_animation = get_test_animation(1);

      model_animator.add_new_animation_to_queue(model_animation);

      let current_frame_duration = model_animator
        .get_current_animation_frame_duration()
        .unwrap();

      assert!(model_animator.current_frame_duration_is_finished(current_frame_duration));
    }

    #[test]
    fn no_running_animation() {
      let mut model_data = get_test_model_data();
      let model_sprite = model_data.get_sprite();
      let model_animator = ModelAnimator::new(model_sprite);
      let model_animator_guard = model_animator.lock().unwrap();

      // This method will return true if there is no animation.
      assert!(model_animator_guard.current_frame_duration_is_finished(0));
    }
  }

  // Testing an invalid frame isn't possible, because it's not possible to
  // create an invalid sprite.
  // The only way would be a corrupted model file.
  #[cfg(test)]
  mod change_model_frame_logic {
    use super::*;

    #[test]
    fn valid_input() {
      let mut model_data = get_test_model_data();
      let model_sprite = model_data.get_sprite();
      let model_animator = ModelAnimator::new(model_sprite);
      let mut model_animator = model_animator.lock().unwrap();

      let mut frame = Sprite::new();
      frame
        .change_shape("-----\n--a--\n-----".to_string(), Some('a'), Some('-'))
        .unwrap();
      let new_frame = AnimationFrame::new(frame, 1);

      let expected_appearance = "-----\n-----\n-----".to_string();

      model_animator.change_model_frame(new_frame);

      let model_appearance = model_data.get_sprite().read().unwrap().get_appearance();

      assert_eq!(model_appearance, expected_appearance);
    }
  }

  // data for tests

  const WORLD_POSITION: (usize, usize) = (10, 10);

  fn get_test_model_data() -> ModelData {
    let test_model_path = std::path::Path::new("../tests/models/test_square.model");

    ModelData::from_file(test_model_path, WORLD_POSITION).unwrap()
  }

  // This is temporary until animation file parsers are a thing.
  fn get_test_animation_limited_run_count() -> AnimationFrames {
    let frames = get_test_frames(vec![
      ("lllll\nllall\nlllll".to_string(), 1, 'l'),
      ("mmmmm\nmmamm\nmmmmm".to_string(), 1, 'm'),
      ("nnnnn\nnnann\nnnnnn".to_string(), 1, 'n'),
    ]);

    AnimationFrames::new(frames, AnimationLoopCount::Limited(5))
  }

  // This is temporary until animation file parsers are a thing.
  fn get_test_animation_unlimited_run_count() -> AnimationFrames {
    let frames = get_test_frames(vec![
      ("ooooo\nooaoo\nooooo".to_string(), 1, 'o'),
      ("ppppp\nppapp\nppppp".to_string(), 1, 'p'),
      ("qqqqq\nqqaqq\nqqqqq".to_string(), 1, 'q'),
    ]);

    AnimationFrames::new(frames, AnimationLoopCount::Forever)
  }

  // This is temporary until animation file parsers are a thing.
  fn get_test_animation(loop_count: u64) -> AnimationFrames {
    let frames = get_test_frames(vec![
      ("rrrrr\nrrarr\nrrrrr".to_string(), 1, 'r'),
      ("sssss\nssass\nsssss".to_string(), 1, 's'),
      ("ttttt\nttatt\nttttt".to_string(), 1, 't'),
    ]);

    AnimationFrames::new(frames, AnimationLoopCount::Limited(loop_count))
  }

  /// Gets a list of AnimationFrames of the given appearances, center replacement characters, and durations.
  ///
  /// The default anchor is `a`.
  fn get_test_frames(appearances: Vec<(String, u32, char)>) -> Vec<AnimationFrame> {
    let mut base_frame = Sprite::new();
    base_frame.change_anchor_character('a').unwrap();

    appearances
      .into_iter()
      .map(|(appearance, duration, center_replacement)| {
        base_frame
          .change_shape(appearance, None, Some(center_replacement))
          .unwrap();

        AnimationFrame::new(base_frame.clone(), duration)
      })
      .collect()
  }
}
