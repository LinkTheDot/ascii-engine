pub use crate::models::animation::animation_frames_iterators::*;
use crate::models::sprites::Sprite;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AnimationFrames {
  frames: Vec<AnimationFrame>,
  /// Determines if this animation should loop forever or a set amount of times.
  loop_count: AnimationLoopCount,
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
  pub fn new(animation_frames: Vec<AnimationFrame>, loop_count: AnimationLoopCount) -> Self {
    Self {
      frames: animation_frames,
      loop_count,
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

impl From<(AnimationLoopCount, Vec<(u32, Sprite)>)> for AnimationFrames {
  fn from(item: (AnimationLoopCount, Vec<(u32, Sprite)>)) -> Self {
    let (loop_count, frames) = item;
    let frames: Vec<AnimationFrame> = frames
      .into_iter()
      .map(|(frame_duration, frame)| AnimationFrame::new(frame, frame_duration))
      .collect();

    AnimationFrames::new(frames, loop_count)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::testing_data::*;

  #[test]
  fn animation_frame_get_logic() {
    let test_animation =
      TestingData::get_test_animation(['l', 'm', 'n'], AnimationLoopCount::Limited(1));
    let test_frame = test_animation.into_iter().next().unwrap();

    let expected_appearance =
      Sprite::new(TestingData::get_frame_appearance('l'), 'a', 'l', '-').unwrap();

    assert_eq!(test_frame.get_appearance(), &expected_appearance);
    assert_eq!(test_frame.get_frame_duration(), 1);
  }

  #[test]
  fn animation_frames_get_logic() {
    let loop_count = AnimationLoopCount::Limited(1);
    let test_animation = TestingData::get_test_animation(['l', 'm', 'n'], loop_count);
    let test_frame_list: Vec<AnimationFrame> = test_animation.clone().into_iter().collect();

    assert_eq!(test_animation.get_frames(), &test_frame_list);
    assert_eq!(test_animation.get_loop_count(), &loop_count);
  }

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

    let test_animation_from = AnimationFrames::from((loop_count, test_animation_data));

    assert_eq!(test_animation, test_animation_from);
  }
}
