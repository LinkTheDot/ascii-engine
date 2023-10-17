// use crate::errors::{AnimationError, ModelError};
// use crate::models::animation::ModelAnimator;
// use std::sync::{Arc, Mutex};
// use std::thread::JoinHandle;
// use tokio::sync::mpsc;
// use tokio::sync::oneshot;
//
// /// A connection to the animation thread.
// ///
// /// This will only live in the screen, nowhere else.
// pub struct AnimationThreadConnection {
//   _handle: JoinHandle<()>,
//   request_sender: mpsc::UnboundedSender<AnimationRequest>,
//   kill_sender: oneshot::Sender<()>,
//   kill_hash: u64,
// }
//
// #[derive(Debug)]
// pub struct AnimationRequest {
//   pub model_unique_hash: u64,
//   pub request: AnimationAction,
// }
//
// #[derive(Debug, Clone)]
// pub enum AnimationAction {
//   AddAnimator(Arc<Mutex<ModelAnimator>>),
//   RemoveAnimator,
//   KillThread,
// }
//
// impl PartialEq for AnimationAction {
//   fn eq(&self, other: &Self) -> bool {
//     // Because AddAnimator contains data, the enum can't be cast directly into an integer.
//     let self_int: u8 = match self {
//       AnimationAction::AddAnimator(_) => 0,
//       AnimationAction::RemoveAnimator => 1,
//       AnimationAction::KillThread => 2,
//     };
//     let other_int: u8 = match other {
//       AnimationAction::AddAnimator(_) => 0,
//       AnimationAction::RemoveAnimator => 1,
//       AnimationAction::KillThread => 2,
//     };
//
//     self_int == other_int
//   }
// }
//
// impl AnimationThreadConnection {
//   pub fn new(
//     thread_handle: JoinHandle<()>,
//     request_sender: mpsc::UnboundedSender<AnimationRequest>,
//     kill_sender: oneshot::Sender<()>,
//     kill_hash: u64,
//   ) -> Self {
//     Self {
//       _handle: thread_handle,
//       request_sender,
//       kill_sender,
//       kill_hash,
//     }
//   }
//
//   pub fn clone_sender(&self) -> mpsc::UnboundedSender<AnimationRequest> {
//     self.request_sender.clone()
//   }
//
//   // TODO: Have the screen create and store the hash once the animation thread is spawned
//   /// # Errors
//   ///
//   /// - An error is returned when the animation thread doesn't exist.
//   pub fn kill_thread(self) {
//     self.kill_sender.send(()).unwrap();
//
//     let kill_request = AnimationRequest {
//       model_unique_hash: self.kill_hash,
//       request: AnimationAction::KillThread,
//     };
//
//     self.request_sender.send(kill_request).unwrap();
//   }
//
//   // TODO: List the errors.
//   pub fn send_request(&self, request: AnimationRequest) -> Result<(), ModelError> {
//     if self.request_sender.send(request).is_err() {
//       Err(ModelError::AnimationError(
//         AnimationError::AnimationThreadClosed,
//       ))
//     } else {
//       Ok(())
//     }
//   }
// }
//
// impl AnimationRequest {
//   pub fn new(hash: u64, action: AnimationAction) -> Self {
//     Self::from((hash, action))
//   }
// }
//
// impl From<(u64, AnimationAction)> for AnimationRequest {
//   fn from((model_unique_hash, request): (u64, AnimationAction)) -> Self {
//     Self {
//       model_unique_hash,
//       request,
//     }
//   }
// }
