use crate::models::animation::animation_frames::*;

#[derive(Debug, Clone)]
pub struct AnimationFramesIntoIter {
  frames: AnimationFrames,
  current_loop_counter: u64,
}

impl IntoIterator for AnimationFrames {
  type Item = AnimationFrame;
  type IntoIter = AnimationFramesIntoIter;

  fn into_iter(self) -> Self::IntoIter {
    AnimationFramesIntoIter {
      frames: self,
      current_loop_counter: 0,
    }
  }
}

impl Iterator for AnimationFramesIntoIter {
  type Item = AnimationFrame;

  fn next(&mut self) -> Option<Self::Item> {
    if self.frames.reached_loop_count(self.current_loop_counter) {
      return None;
    }

    let item = self
      .frames
      .get_frame(self.current_loop_counter % self.frames.frame_count());

    self.current_loop_counter += 1;

    item.cloned()
  }
}

impl AnimationFramesIntoIter {
  pub fn current_frame_duration(&self) -> Option<u64> {
    Some(
      self
        .frames
        .get_frame(self.current_loop_counter % self.frames.frame_count())?
        .get_frame_duration() as u64,
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn iter_logic() {
    let animation_frames = get_test_animation();
    let frame_1 = AnimationFrame::new("lllll\nllall\nlllll".to_string(), 1, None);
    let frame_2 = AnimationFrame::new("mmmmm\nmmamm\nmmmmm".to_string(), 1, None);
    let frame_3 = AnimationFrame::new("nnnnn\nnnann\nnnnnn".to_string(), 1, None);

    let mut iter = animation_frames.into_iter();

    // Loop 1
    assert_eq!(iter.next(), Some(frame_1.clone()));
    assert_eq!(iter.next(), Some(frame_2.clone()));
    assert_eq!(iter.next(), Some(frame_3.clone()));

    // Loop 2
    assert_eq!(iter.next(), Some(frame_1));
    assert_eq!(iter.next(), Some(frame_2));
    assert_eq!(iter.next(), Some(frame_3));

    // Run count hit it's limit.
    assert_eq!(iter.next(), None);
  }

  // Test Data

  fn get_test_animation() -> AnimationFrames {
    let frames = vec![
      AnimationFrame::new("lllll\nllall\nlllll".to_string(), 1, None),
      AnimationFrame::new("mmmmm\nmmamm\nmmmmm".to_string(), 1, None),
      AnimationFrame::new("nnnnn\nnnann\nnnnnn".to_string(), 1, None),
    ];

    AnimationFrames::new(frames, AnimationLoopCount::Limited(2))
  }
}
