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
  use crate::models::testing_data::TestingData;

  #[test]
  fn iter_logic() {
    let animation_frames =
      TestingData::get_test_animation(['l', 'm', 'n'], AnimationLoopCount::Limited(2));
    let frame_data = vec![
      ("lllll\nllall\nlllll".to_string(), 1, 'l'),
      ("mmmmm\nmmamm\nmmmmm".to_string(), 1, 'm'),
      ("nnnnn\nnnann\nnnnnn".to_string(), 1, 'n'),
    ];
    let frame_data = TestingData::get_test_frames(frame_data);

    let mut iter = animation_frames.into_iter();

    // Loop 1
    assert_eq!(iter.next(), Some(frame_data[0].clone()));
    assert_eq!(iter.next(), Some(frame_data[1].clone()));
    assert_eq!(iter.next(), Some(frame_data[2].clone()));

    // Loop 2
    assert_eq!(iter.next(), Some(frame_data[0].clone()));
    assert_eq!(iter.next(), Some(frame_data[1].clone()));
    assert_eq!(iter.next(), Some(frame_data[2].clone()));

    // Run count hit its limit.
    assert_eq!(iter.next(), None);
  }

  #[test]
  fn get_from_iteration_logic() {}
}
