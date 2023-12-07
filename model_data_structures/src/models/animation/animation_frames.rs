use crate::errors::*;
use crate::models::model_appearance::sprites::Sprite;
use serde::{Deserialize, Serialize};

// TODO: Add documentation
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AnimationFrames {
  frames: Vec<AnimationFrame>,
  /// Determines if this animation should loop forever or a set amount of times.
  loop_count: AnimationLoopCount,
  /// The appearance of the model when there are no animations running.
  resting_appearance: Option<Sprite>,
}

// TODO: Add documentation
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AnimationFrame {
  appearance: Sprite,
  /// This is how many ticks this frame should live for.
  frame_duration: u32,
}

// TODO: Add documentation
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum AnimationLoopCount {
  Forever,
  /// Contains how many times an animation should loop for.
  Limited(u64),
}

impl AnimationFrames {
  // TODO: Add documentation
  pub fn new(
    animation_frames: Vec<AnimationFrame>,
    loop_count: AnimationLoopCount,
    resting_appearance: Option<Sprite>,
  ) -> Self {
    Self {
      frames: animation_frames,
      loop_count,
      resting_appearance,
    }
  }

  /// Returns a reference of the given frame index.
  ///
  /// The index does *not* wrap around based on loop count.
  /// If there are only 3 frames, only 0-2 will be valid.
  ///
  /// Returns None when the index in greater than the amount of frames in the animation.
  pub fn get_frame(&self, index: u64) -> Option<&AnimationFrame> {
    self.frames.get(index as usize)
  }

  /// Returns the amount of frames in a single iteration of the animation.
  pub fn frame_count(&self) -> u64 {
    self.frames.len() as u64
  }

  /// Takes an exclusive amount of frames that've been iterated through.
  ///
  /// Returns true if the amount of frames iterated exceeds the animation's duration.
  ///
  /// The duration of an animation would be frames * loop_count.
  pub fn reached_loop_count(&self, frames_iterated_through: u64) -> bool {
    let animation_loops_occurred = frames_iterated_through / self.frame_count();

    self.loop_count.reached_loop_count(animation_loops_occurred)
  }

  /// Returns a reference to the list of frames in the animation.
  pub fn get_frames(&self) -> &Vec<AnimationFrame> {
    &self.frames
  }

  /// Returns the loop count of this animation.
  pub fn get_loop_count(&self) -> &AnimationLoopCount {
    &self.loop_count
  }

  /// Replaces the current resting appearance with the new given one.
  ///
  /// Returns the old resting appearance if it existed.
  pub fn set_resting_appearance(&mut self, new_resting_appearance: Sprite) -> Option<Sprite> {
    std::mem::replace(&mut self.resting_appearance, Some(new_resting_appearance))
  }

  /// Returns a reference to the animation's resting appearance.
  ///
  /// The resting appearance is what the animation would look like if it were to stop running for whatever reason.
  pub fn get_resting_appearance(&self) -> Option<&Sprite> {
    self.resting_appearance.as_ref()
  }

  /// Returns the total duration of the animation in ticks.
  ///
  /// None is returned if the animation runs to infinity.
  pub fn get_total_duration(&self) -> Option<u64> {
    let AnimationLoopCount::Limited(loop_count) = self.loop_count else {
      return None;
    };

    Some(self.get_cycle_duration() * loop_count)
  }

  /// Returns the total ticks for each loop of the animation.
  pub fn get_cycle_duration(&self) -> u64 {
    self
      .frames
      .iter()
      .map(AnimationFrame::get_frame_duration)
      .sum::<u32>() as u64
  }

  /// Returns the frame in the animation based on the amount of ticks.
  ///
  /// Let's say the list of frames in the animation have durations of 3 -> 2 -> 1 ticks.
  /// If you were to pass 4 ticks into this method, the second frame would be returned.
  /// Four ticks means frame 1 is finished, and we're in the middle of frame 2.
  ///
  /// If the amount of ticks surpasses 1 cycle of the animation, passed_in_ticks % cycle_duration is
  /// used to calculate which frame to choose.
  ///
  /// None is returned if the ticks passed in surpasses the total duration of the animation itself.
  pub fn get_frame_based_on_ticks(&self, ticks: u64) -> Option<&AnimationFrame> {
    let total_animation_duration = self.get_total_duration();

    if let Some(total_animation_duration) = total_animation_duration {
      if total_animation_duration <= ticks {
        return None;
      }
    }

    let cycle_duration = self.get_cycle_duration();

    // If there's 0 frames but the loop count is set to 'Forever'.
    if cycle_duration == 0 {
      return None;
    }

    let mut remaining_ticks = ticks % cycle_duration;

    let current_frame = self.frames.iter().position(|frame| {
      let frame_duration = frame.get_frame_duration() as u64;

      if remaining_ticks < frame_duration {
        true
      } else {
        remaining_ticks -= frame_duration;

        false
      }
    })? as u64;

    self.get_frame(current_frame)
  }

  /// Returns an [`AnimationValidityErrorData`](crate::models::animation::errors::AnimationValidityErrorData)
  /// which contains the list of errors for each invalid frame, and the index tied to it.
  ///
  /// Checks every frame in the animation, including the resting frame.
  ///
  /// # Sprite Errors
  ///
  /// - The stored shape isn't rectangular.
  /// - The stored shape doesn't have an anchor.
  /// - The stored shape has multiple anchors.
  /// - The anchor and air characters are the same.
  pub fn validity_check(&self, animation_name: &str) -> Result<(), AnimationValidityErrorData> {
    let resting_appearance_errors = if let Some(Err(ModelError::SpriteValidityChecks(error_list))) =
      self.resting_appearance.as_ref().map(Sprite::validity_check)
    {
      Some(error_list)
    } else {
      None
    };

    let invalid_frame_errors: Vec<(usize, Vec<ModelError>)> = self
      .get_frames()
      .iter()
      .enumerate()
      .filter_map(|(iteration, frame)| {
        if let Err(ModelError::SpriteValidityChecks(error_list)) =
          frame.get_appearance().validity_check()
        {
          Some((iteration, error_list))
        } else {
          None
        }
      })
      .collect();

    if !invalid_frame_errors.is_empty() || resting_appearance_errors.is_some() {
      Err(AnimationValidityErrorData {
        animation_name: animation_name.to_string(),
        resting_appearance_errors,
        invalid_frame_errors,
      })
    } else {
      Ok(())
    }
  }
}

impl AnimationFrame {
  pub fn new(appearance: Sprite, duration: u32) -> Self {
    Self {
      appearance,
      frame_duration: duration,
    }
  }

  pub fn get_frame_duration(&self) -> u32 {
    self.frame_duration
  }

  pub fn get_appearance(&self) -> &Sprite {
    &self.appearance
  }
}

impl AnimationLoopCount {
  pub fn reached_loop_count(&self, current_loop_counter: u64) -> bool {
    match self {
      AnimationLoopCount::Forever => false,
      AnimationLoopCount::Limited(max_loop_count) => max_loop_count == &current_loop_counter,
    }
  }
}

impl From<(AnimationLoopCount, Vec<(u32, Sprite)>, Option<Sprite>)> for AnimationFrames {
  fn from(item: (AnimationLoopCount, Vec<(u32, Sprite)>, Option<Sprite>)) -> Self {
    let (loop_count, frames, resting_appearance) = item;
    let frames: Vec<AnimationFrame> = frames
      .into_iter()
      .map(|(frame_duration, frame)| AnimationFrame::new(frame, frame_duration))
      .collect();

    AnimationFrames::new(frames, loop_count, resting_appearance)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::testing_data::*;

  #[test]
  fn animation_frames_from_logic() {
    let loop_count = AnimationLoopCount::Limited(1);
    let test_animation = TestingData::get_test_animation(['l', 'm', 'n'], loop_count);
    let test_animation_data = ['l', 'm', 'n']
      .into_iter()
      .map(|frame_char| {
        let sprite = Sprite::new(
          TestingData::get_frame_appearance(frame_char),
          'a',
          frame_char,
          '-',
        )
        .unwrap();

        (1, sprite)
      })
      .collect::<Vec<(u32, Sprite)>>();

    let test_animation_from = AnimationFrames::from((loop_count, test_animation_data, None));

    assert_eq!(test_animation, test_animation_from);
  }

  #[test]
  fn get_duration_logic() {
    let loop_count_limited = AnimationLoopCount::Limited(2);
    let loop_count_unlimited = AnimationLoopCount::Forever;
    let limited_animation = TestingData::get_test_animation(['l', 'm', 'n'], loop_count_limited);
    let unlimited_animation =
      TestingData::get_test_animation(['l', 'm', 'n'], loop_count_unlimited);

    assert_eq!(limited_animation.get_cycle_duration(), 3);
    assert_eq!(limited_animation.get_total_duration(), Some(6));

    assert_eq!(unlimited_animation.get_cycle_duration(), 3);
    assert_eq!(unlimited_animation.get_total_duration(), None);
  }

  #[cfg(test)]
  mod get_frame_based_on_ticks_logic {
    use super::*;

    #[test]
    fn limited_runtime() {
      let loop_count_limited = AnimationLoopCount::Limited(2);
      let limited_animation = TestingData::get_test_animation(['l', 'm', 'n'], loop_count_limited);

      let expected_frames = limited_animation.clone();

      assert_eq!(
        limited_animation.get_frame_based_on_ticks(3),
        expected_frames.get_frame(0)
      );
      assert_eq!(
        limited_animation.get_frame_based_on_ticks(4),
        expected_frames.get_frame(1)
      );
      assert_eq!(
        limited_animation.get_frame_based_on_ticks(5),
        expected_frames.get_frame(2)
      );
      assert!(limited_animation.get_frame_based_on_ticks(6).is_none());
    }

    #[test]
    fn unlimited_runtime() {
      let loop_count_unlimited = AnimationLoopCount::Forever;
      let unlimited_animation =
        TestingData::get_test_animation(['l', 'm', 'n'], loop_count_unlimited);

      let expected_frames = unlimited_animation.clone();

      assert_eq!(
        unlimited_animation.get_frame_based_on_ticks(30),
        expected_frames.get_frame(0)
      );
      assert_eq!(
        unlimited_animation.get_frame_based_on_ticks(31),
        expected_frames.get_frame(1)
      );
      assert_eq!(
        unlimited_animation.get_frame_based_on_ticks(32),
        expected_frames.get_frame(2)
      );
      assert_eq!(
        unlimited_animation.get_frame_based_on_ticks(33),
        expected_frames.get_frame(0)
      );
    }

    #[test]
    fn empty_animation() {
      let animation = AnimationFrames::new(vec![], AnimationLoopCount::Forever, None);

      assert!(animation.get_frame_based_on_ticks(0).is_none());
      assert!(animation.get_frame_based_on_ticks(1).is_none());
      assert!(animation.get_frame_based_on_ticks(2).is_none());
      assert!(animation.get_frame_based_on_ticks(3).is_none());
    }
  }

  #[test]
  fn frame_count_logic() {
    let loop_count = AnimationLoopCount::Limited(2);
    let animation = TestingData::get_test_animation(['l', 'm', 'n'], loop_count);

    assert_eq!(animation.frame_count(), 3);
  }

  #[cfg(test)]
  mod reached_loop_count_logic {
    use super::*;

    #[test]
    fn animation_has_limited_loop_count() {
      let loop_count_limited = AnimationLoopCount::Limited(2);
      let limited_animation = TestingData::get_test_animation(['l', 'm', 'n'], loop_count_limited);

      assert!(!limited_animation.reached_loop_count(0));
      assert!(!limited_animation.reached_loop_count(2));
      assert!(!limited_animation.reached_loop_count(3));
      assert!(limited_animation.reached_loop_count(6));
    }

    #[test]
    fn animation_has_unlimited_loop_count() {
      let loop_count_unlimited = AnimationLoopCount::Forever;
      let unlimited_animation =
        TestingData::get_test_animation(['l', 'm', 'n'], loop_count_unlimited);

      assert!(!unlimited_animation.reached_loop_count(9999));
      assert!(!unlimited_animation.reached_loop_count(9999));
    }
  }

  #[test]
  fn get_loop_count_logic() {
    let loop_count_limited = AnimationLoopCount::Limited(2);
    let limited_animation = TestingData::get_test_animation(['l', 'm', 'n'], loop_count_limited);

    let loop_count_unlimited = AnimationLoopCount::Forever;
    let unlimited_animation =
      TestingData::get_test_animation(['l', 'm', 'n'], loop_count_unlimited);

    assert_eq!(limited_animation.get_loop_count(), &loop_count_limited);
    assert_eq!(unlimited_animation.get_loop_count(), &loop_count_unlimited);
  }

  #[cfg(test)]
  mod get_resting_appearance_logic {
    use super::*;

    #[test]
    fn animation_has_no_resting_appearance() {
      let loop_count = AnimationLoopCount::Limited(2);
      let animation = TestingData::get_test_animation(['l', 'm', 'n'], loop_count);

      assert!(animation.get_resting_appearance().is_none());
    }

    #[test]
    fn animation_has_a_resting_appearance() {
      let loop_count = AnimationLoopCount::Limited(2);
      let mut animation = TestingData::get_test_animation(['l', 'm', 'n'], loop_count);
      let resting_appearance =
        Sprite::new(TestingData::get_frame_appearance('l'), 'a', 'l', '-').unwrap();
      let _ = animation.set_resting_appearance(resting_appearance.clone());

      assert_eq!(
        animation.get_resting_appearance(),
        Some(&resting_appearance)
      );
    }
  }

  #[cfg(test)]
  mod validity_check_logic {
    use super::*;

    #[test]
    fn every_frame_is_valid() {
      let loop_count = AnimationLoopCount::Limited(2);
      let animation = TestingData::get_test_animation(['l', 'm', 'n'], loop_count);

      assert!(animation.validity_check("").is_ok());
    }

    #[test]
    fn specific_frame_is_invalid() {
      let loop_count = AnimationLoopCount::Limited(2);
      let animation_frames = vec![
        (1, Sprite::new_unchecked("xxx\nxcx", 'c', 'x', '-', 4)),
        (1, Sprite::new_unchecked("-xx\nxcx", 'c', 'x', '-', 4)),
        (1, Sprite::new_unchecked("x-x\nxcx", 'c', 'x', '-', 4)),
        (1, Sprite::new_unchecked("xx\nxcc", 'c', 'x', 'c', 4)),
      ];
      let animation = AnimationFrames::from((loop_count, animation_frames, None));
      let animation_name = "test_animation".to_string();

      let expected_result = Err(AnimationValidityErrorData {
        animation_name: animation_name.clone(),
        resting_appearance_errors: None,
        invalid_frame_errors: vec![(
          3,
          vec![
            ModelError::NonRectangularShape,
            ModelError::MultipleAnchorsFound(vec![3, 4]),
            ModelError::SpriteAnchorMatchesAirCharacter,
          ],
        )],
      });

      let result = animation.validity_check(&animation_name);

      assert_eq!(result, expected_result);
    }

    #[test]
    fn every_frame_is_invalid() {
      let loop_count = AnimationLoopCount::Limited(2);
      let broken_sprite = Sprite::new_unchecked("xx\nxcc", 'c', 'x', 'c', 4);
      let animation_frames = std::iter::repeat((1, broken_sprite.clone()))
        .take(4)
        .collect();
      let mut animation = AnimationFrames::from((loop_count, animation_frames, None));
      animation.set_resting_appearance(broken_sprite);
      let animation_name = "test_animation".to_string();

      let sprite_errors = vec![
        ModelError::NonRectangularShape,
        ModelError::MultipleAnchorsFound(vec![3, 4]),
        ModelError::SpriteAnchorMatchesAirCharacter,
      ];
      let expected_result = Err(AnimationValidityErrorData {
        animation_name: animation_name.clone(),
        resting_appearance_errors: Some(sprite_errors.clone()),
        invalid_frame_errors: vec![
          (0, sprite_errors.clone()),
          (1, sprite_errors.clone()),
          (2, sprite_errors.clone()),
          (3, sprite_errors),
        ],
      });

      let result = animation.validity_check(&animation_name);

      assert_eq!(result, expected_result);
    }

    #[test]
    fn resting_appearance_is_invalid() {
      let loop_count = AnimationLoopCount::Limited(2);
      let mut animation = TestingData::get_test_animation(['l', 'm', 'n'], loop_count);
      let broken_sprite = Sprite::new_unchecked("xx\nxcc", 'c', 'x', 'c', 4);
      animation.set_resting_appearance(broken_sprite);
      let animation_name = "test_animation".to_string();

      let sprite_errors = vec![
        ModelError::NonRectangularShape,
        ModelError::MultipleAnchorsFound(vec![3, 4]),
        ModelError::SpriteAnchorMatchesAirCharacter,
      ];
      let expected_result = Err(AnimationValidityErrorData {
        animation_name: animation_name.clone(),
        resting_appearance_errors: Some(sprite_errors),
        invalid_frame_errors: vec![],
      });

      let result = animation.validity_check(&animation_name);

      assert_eq!(result, expected_result);
    }
  }
}
