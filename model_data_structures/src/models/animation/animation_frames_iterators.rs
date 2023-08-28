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

  pub fn get_current_frame(&self) -> Option<AnimationFrame> {
    self
      .frames
      .get_frame(self.current_loop_counter % self.frames.frame_count())
      .cloned()
  }
}

#[cfg(test)]
mod frame_tests {
  use super::*;
  use crate::models::sprites::Sprite;

  #[test]
  fn iter_logic() {
    let animation_frames = get_test_animation(2);
    let [frame_1, frame_2, frame_3] = get_test_frames();

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

  fn get_test_animation(loop_count: u64) -> AnimationFrames {
    let frames = get_test_frames().to_vec();

    AnimationFrames::new(frames, AnimationLoopCount::Limited(loop_count))
  }

  fn get_test_frames() -> [AnimationFrame; 3] {
    let mut base_frame = Sprite::new();
    base_frame.change_anchor_character('a').unwrap();

    base_frame
      .change_shape("lllll\nllall\nlllll".to_string(), None, Some('l'))
      .unwrap();
    let frame_one = base_frame.clone();

    base_frame
      .change_shape("mmmmm\nmmamm\nmmmmm".to_string(), None, Some('m'))
      .unwrap();
    let frame_two = base_frame.clone();

    base_frame
      .change_shape("nnnnn\nnnann\nnnnnn".to_string(), None, Some('n'))
      .unwrap();
    let frame_three = base_frame;

    [
      AnimationFrame::new(frame_one, 1),
      AnimationFrame::new(frame_two, 1),
      AnimationFrame::new(frame_three, 1),
    ]
  }
}
