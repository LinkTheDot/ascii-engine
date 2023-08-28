// pub use crate::models::animation::animation_frames_iterators::*;
//
// #[derive(Debug, Clone, Eq, PartialEq)]
// pub struct AnimationFrames {
//   frames: Vec<AnimationFrame>,
//   /// Determines if this animation should loop forever or a set amount of times.
//   loop_count: AnimationLoopCount,
// }
//
// #[derive(Debug, Clone, Eq, PartialEq)]
// pub struct AnimationFrame {
//   appearance: String,
//   /// This is how many ticks this frame should live for.
//   frame_duration: u32,
//   anchor_replacement_character: Option<char>,
// }
//
// #[derive(Debug, Clone, Eq, PartialEq)]
// pub enum AnimationLoopCount {
//   Forever,
//   /// Contains how many times an animation should loop for.
//   Limited(u64),
// }
//
// impl AnimationFrames {
//   pub fn new(animation_frames: Vec<AnimationFrame>, loop_count: AnimationLoopCount) -> Self {
//     Self {
//       frames: animation_frames,
//       loop_count,
//     }
//   }
//
//   pub fn get_frame(&self, index: u64) -> Option<&AnimationFrame> {
//     self.frames.get(index as usize)
//   }
//
//   pub fn frame_count(&self) -> u64 {
//     self.frames.len() as u64
//   }
//
//   pub fn reached_loop_count(&self, frames_iterated_through: u64) -> bool {
//     let animation_loops_occurred = frames_iterated_through / self.frame_count();
//
//     self.loop_count.reached_loop_count(animation_loops_occurred)
//   }
//
//   pub fn get_frames(&self) -> &Vec<AnimationFrame> {
//     &self.frames
//   }
//
//   pub fn get_loop_count(&self) -> &AnimationLoopCount {
//     &self.loop_count
//   }
// }
//
// impl AnimationFrame {
//   pub fn new(
//     appearance: String,
//     duration: u32,
//     anchor_replacement_character: Option<char>,
//   ) -> Self {
//     Self {
//       appearance,
//       frame_duration: duration,
//       anchor_replacement_character,
//     }
//   }
//
//   pub fn get_frame_duration(&self) -> u32 {
//     self.frame_duration
//   }
//
//   pub fn get_appearance(&self) -> &str {
//     &self.appearance
//   }
//
//   pub fn get_anchor_replacement_char(&self) -> Option<char> {
//     self.anchor_replacement_character
//   }
// }
//
// impl AnimationLoopCount {
//   pub fn reached_loop_count(&self, current_loop_counter: u64) -> bool {
//     match self {
//       AnimationLoopCount::Forever => false,
//       AnimationLoopCount::Limited(max_loop_count) => max_loop_count == &current_loop_counter,
//     }
//   }
// }
//
// impl std::convert::From<(AnimationLoopCount, Vec<(u32, String)>)> for AnimationFrames {
//   fn from(item: (AnimationLoopCount, Vec<(u32, String)>)) -> Self {
//     let (loop_count, frames) = item;
//     let frames: Vec<AnimationFrame> = frames
//       .into_iter()
//       .map(|(frame_duration, frame)| AnimationFrame::new(frame, frame_duration, None))
//       .collect();
//
//     AnimationFrames::new(frames, loop_count)
//   }
// }
//
// #[cfg(test)]
// mod tests {
//   use super::*;
//
//   #[test]
//   fn get_frame_logic() {
//     let test_animation = get_test_animation(999);
//
//     let expected_frame = AnimationFrame::new("rrrrr\nrrarr\nrrrrr".to_string(), 1, None);
//
//     let obtained_frame = test_animation.get_frame(0);
//
//     assert_eq!(obtained_frame, Some(&expected_frame));
//   }
//
//   #[test]
//   fn frame_count_logic() {
//     let test_animation = get_test_animation(999);
//
//     let test_animation_frame_count = test_animation.frame_count();
//
//     assert_eq!(test_animation_frame_count, 3);
//   }
//
//   fn get_test_animation(loop_count: u64) -> AnimationFrames {
//     let frames = vec![
//       AnimationFrame::new("rrrrr\nrrarr\nrrrrr".to_string(), 1, None),
//       AnimationFrame::new("sssss\nssass\nsssss".to_string(), 1, None),
//       AnimationFrame::new("ttttt\nttatt\nttttt".to_string(), 1, None),
//     ];
//
//     AnimationFrames::new(frames, AnimationLoopCount::Limited(loop_count))
//   }
// }
