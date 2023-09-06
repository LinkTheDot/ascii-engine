pub use crate::models::animation::animation_frames_iterators::*;
use crate::models::sprites::Sprite;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AnimationFrames {
  frames: Vec<AnimationFrame>,
  /// Determines if this animation should loop forever or a set amount of times.
  loop_count: AnimationLoopCount,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AnimationFrame {
  appearance: Sprite,
  /// This is how many ticks this frame should live for.
  frame_duration: u32,
}

#[derive(Debug, Clone, Eq, PartialEq)]
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
