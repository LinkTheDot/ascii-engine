pub use crate::models::animation::animation_frames_iterators::*;
use crate::models::model_appearance::sprites::Sprite;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AnimationFrames {
  frames: Vec<AnimationFrame>,
  /// Determines if this animation should loop forever or a set amount of times.
  loop_count: AnimationLoopCount,
  /// The appearance of the model when there are no animations running.
  resting_appearance: Option<Sprite>,
  running: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AnimationFrame {
  appearance: Sprite,
  /// This is how many ticks this frame should live for.
  frame_duration: u32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum AnimationLoopCount {
  Forever,
  /// Contains how many times an animation should loop for.
  Limited(u64),
}

impl AnimationFrames {
  pub fn new(
    animation_frames: Vec<AnimationFrame>,
    loop_count: AnimationLoopCount,
    resting_appearance: Option<Sprite>,
  ) -> Self {
    Self {
      frames: animation_frames,
      loop_count,
      resting_appearance,
      running: true,
    }
  }

  pub fn get_frame(&self, index: u64) -> Option<&AnimationFrame> {
    self.frames.get(index as usize)
  }

  pub fn frame_count(&self) -> u64 {
    self.frames.len() as u64
  }

  pub fn reached_loop_count(&self, frames_iterated_through: u64) -> bool {
    let animation_loops_occurred = frames_iterated_through / self.frame_count();

    self.loop_count.reached_loop_count(animation_loops_occurred)
  }

  pub fn get_frames(&self) -> &Vec<AnimationFrame> {
    &self.frames
  }

  pub fn get_loop_count(&self) -> &AnimationLoopCount {
    &self.loop_count
  }

  pub fn pause(&mut self) {
    self.running = false;
  }

  pub fn start(&mut self) {
    self.running = true;
  }

  pub fn is_running(&self) -> bool {
    self.running
  }

  pub fn get_resting_appearance(&self) -> Option<&Sprite> {
    self.resting_appearance.as_ref()
  }

  /// Returns the total duration of the animation.
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

    if total_animation_duration.is_some() && total_animation_duration.unwrap() <= ticks {
      return None;
    }

    let mut remaining_ticks = ticks % self.get_cycle_duration();

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

  // #[test]
  // fn animation_frame_get_logic() {
  //   let test_animation =
  //     TestingData::get_test_animation(['l', 'm', 'n'], AnimationLoopCount::Limited(1));
  //   let test_frame = test_animation.into_iter().next().unwrap();
  //
  //   let expected_appearance =
  //     Sprite::new(TestingData::get_frame_appearance('l'), 'a', 'l', '-').unwrap();
  //
  //   assert_eq!(test_frame.get_appearance(), &expected_appearance);
  //   assert_eq!(test_frame.get_frame_duration(), 1);
  // }
  //
  // #[test]
  // fn animation_frames_get_logic() {
  //   let loop_count = AnimationLoopCount::Limited(1);
  //   let test_animation = TestingData::get_test_animation(['l', 'm', 'n'], loop_count);
  //   let test_frame_list: Vec<AnimationFrame> = test_animation.clone().into_iter().collect();
  //
  //   assert_eq!(test_animation.get_frames(), &test_frame_list);
  //   assert_eq!(test_animation.get_loop_count(), &loop_count);
  // }

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
  }
}
