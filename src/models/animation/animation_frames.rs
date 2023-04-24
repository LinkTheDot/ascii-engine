#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AnimationFrames {
  frames: Vec<AnimationFrame>,
  /// Determines if this animation should loop forever or a set amount of times.
  loop_count: AnimationLoopCount,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AnimationFrame {
  appearance: String,
  /// This is how many ticks this frame should live for.
  frame_duration: u32,
  anchor_replacement_character: Option<char>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AnimationLoopCount {
  Forever,
  /// Contains how many times an animation should loop for.
  Limited(u32),
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

    self
      .loop_count
      .reached_loop_count(animation_loops_occurred as u32)
  }
}

impl AnimationFrame {
  pub fn new(
    appearance: String,
    duration: u32,
    anchor_replacement_character: Option<char>,
  ) -> Self {
    Self {
      appearance,
      frame_duration: duration,
      anchor_replacement_character,
    }
  }

  pub fn get_frame_duration(&self) -> u32 {
    self.frame_duration
  }

  pub fn get_appearance(&self) -> &str {
    &self.appearance
  }

  pub fn get_anchor_replacement_char(&self) -> Option<char> {
    self.anchor_replacement_character
  }
}

impl AnimationLoopCount {
  pub fn reached_loop_count(&self, current_loop_counter: u32) -> bool {
    match self {
      AnimationLoopCount::Forever => false,
      AnimationLoopCount::Limited(max_loop_count) => max_loop_count == &current_loop_counter,
    }
  }
}
