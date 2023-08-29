use crate::screen::screen_data::*;
use engine_math::hasher::*;
use model_data_structures::errors::*;
use model_data_structures::models::animation::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::sync::oneshot;

/// Starts the thread that will handle model animations
///
// Describe how to handle model animations
///
/// # Errors
///
/// - Returns an error when the model animation thread already exists.
pub(crate) fn start_animation_thread(
  screen_data: &ScreenData,
) -> Result<AnimationThreadConnection, AnimationError> {
  if screen_data.animation_thread_started() {
    return Err(AnimationError::AnimationThreadAlreadyStarted);
  }

  let (sender, mut receiver) = mpsc::unbounded_channel::<AnimationRequest>();
  let (kill_sender, mut kill_receiver) = oneshot::channel::<()>();
  let kill_hash = get_unique_hash();
  let event_sync = screen_data.get_event_sync();

  let animation_thread_handle = std::thread::spawn(move || {
    let mut model_animator_list: HashMap<u64, Arc<Mutex<ModelAnimator>>> = HashMap::new();
    let mut iteration = 0;

    while kill_receiver.try_recv().is_err() {
      iteration += 1;

      event_sync.wait_for_tick();

      if model_animator_list.is_empty() {
        if let Some(request_data) = receiver.blocking_recv() {
          if request_data.model_unique_hash == kill_hash
            && request_data.request == AnimationAction::KillThread
          {
            break;
          }

          run_request(&mut model_animator_list, request_data);
        }
      } else {
        while let Ok(request_data) = receiver.try_recv() {
          run_request(&mut model_animator_list, request_data);
        }
      }

      // TODO Stop the user from making an animation with 0 frames, that will cause a divide by 0.
      // TODO Force at least 1 tick wait times between frames when parsing the animation file.
      model_animator_list.values_mut().for_each(|model_animator| {
        let Ok(mut model_animator) = model_animator.try_lock() else {
              return;
            };

        if !model_animator.has_animations_to_run() {
          return;
        }

        if !model_animator.is_running_an_animation() {
          model_animator
            .overwrite_current_animation_with_first_in_queue()
            .unwrap();
        }

        if model_animator.current_frame_duration_is_finished(iteration) {
          let new_model_frame = match model_animator.next_frame() {
            Some(animation_frame) => animation_frame,
            None => {
              if model_animator
                .overwrite_current_animation_with_first_in_queue()
                .is_err()
              {
                return;
              } else {
                model_animator.next_frame().unwrap()
              }
            }
          };

          model_animator.update_when_last_frame_changed(iteration);
          model_animator.change_model_frame(new_model_frame);
        }
      });
    }

    log::warn!("Animation thread has ended.");
  });

  Ok(AnimationThreadConnection::new(
    animation_thread_handle,
    sender,
    kill_sender,
    kill_hash,
  ))
}

fn run_request(
  model_animator_list: &mut HashMap<u64, Arc<Mutex<ModelAnimator>>>,
  request: AnimationRequest,
) {
  match request.request {
    AnimationAction::AddAnimator(model_animator) => {
      model_animator_list.insert(request.model_unique_hash, model_animator);
    }
    AnimationAction::RemoveAnimator => {
      model_animator_list.remove(&request.model_unique_hash);
    }
    _ => (),
  }
}

// #[cfg(test)]
// mod tests {
//   use super::*;
//
//   #[cfg(test)]
//   mod start_animation_thread_logic {
//     use super::*;
//
//     #[test]
//     fn start_thread_once() {
//       let screen_data = ScreenData::new();
//
//       ModelAnimationData::start_animation_thread(&screen_data).unwrap();
//     }
//
//     #[test]
//     #[should_panic]
//     fn start_thread_multiple_times() {
//       let mut screen_data = ScreenData::new();
//
//       screen_data.start_animation_thread().unwrap();
//       ModelAnimationData::start_animation_thread(&screen_data).unwrap();
//     }
//   }
//
//   // how to test the animation thread
//   //   check if it's waiting properly
//   //     > start the thread
//   //     > add 2 models
//   //     > wait 2 ticks for it to sit and wait
//   //     > send in 2 animation requests for both models at the same time
//   //     > if only one of the models changed after that, it waited properly
//   //   check if it's running all animations
//   //     > start the thread
//   //     > add 2 models
//   //     > send requests to add animations
//   //     > if they both changed, it's running requests properly
//
//   #[cfg(test)]
//   mod model_animator_methods {
//     use super::*;
//
//     #[cfg(test)]
//     mod run_request_logic {
//       use super::*;
//
//       #[test]
//       fn add_to_queue_no_running_animation() {
//         let model_data = get_test_model_data();
//         let mut model_animator = ModelAnimatorData::new(model_data);
//         let animation_frames = get_test_animation_limited_run_count();
//         let request = AnimationAction::AddToQueue(animation_frames);
//
//         let result = model_animator.run_request(request);
//
//         assert!(!result);
//         assert!(model_animator.animation_queue.is_empty());
//         assert!(model_animator.current_animation.is_some());
//       }
//
//       #[test]
//       fn add_to_queue_currently_running_animation() {
//         let model_data = get_test_model_data();
//         let mut model_animator = ModelAnimatorData::new(model_data);
//         let animation_frames = get_test_animation_limited_run_count();
//         model_animator.overwrite_current_animation(animation_frames.clone());
//         let request = AnimationAction::AddToQueue(animation_frames);
//
//         let result = model_animator.run_request(request);
//
//         assert!(!result);
//         assert!(!model_animator.animation_queue.is_empty());
//         assert!(model_animator.current_animation.is_some());
//       }
//
//       #[test]
//       fn overwrite_current_animation() {
//         let model_data = get_test_model_data();
//         let mut model_animator = ModelAnimatorData::new(model_data);
//         let animation_frames = get_test_animation_limited_run_count();
//         let initial_appearance_request = AnimationAction::AddToQueue(animation_frames);
//         model_animator.run_request(initial_appearance_request);
//         let replacing_animation_frames = get_test_animation_unlimited_run_count();
//         let overwrite_request =
//           AnimationAction::OverwriteCurrentAnimation(replacing_animation_frames);
//
//         let expected_frame_appearance = "ooooo\nooaoo\nooooo".to_string();
//
//         let result = model_animator.run_request(overwrite_request);
//         let frame_0_appearance = model_animator
//           .next_frame()
//           .unwrap()
//           .get_appearance()
//           .to_owned();
//
//         assert!(!result);
//         assert!(model_animator.animation_queue.is_empty());
//         assert_eq!(frame_0_appearance, expected_frame_appearance);
//       }
//
//       #[test]
//       fn clear_queue() {
//         let model_data = get_test_model_data();
//         let mut model_animator = ModelAnimatorData::new(model_data);
//         let animation_frames = get_test_animation_limited_run_count();
//         let appearance_request = AnimationAction::AddToQueue(animation_frames);
//         model_animator.run_request(appearance_request.clone());
//         model_animator.run_request(appearance_request);
//
//         assert!(!model_animator.animation_queue.is_empty());
//
//         let request = AnimationAction::ClearQueue;
//         let result = model_animator.run_request(request);
//
//         assert!(!result);
//         assert!(!model_animator.has_animations_queued());
//       }
//
//       #[test]
//       // Nothing should happen other than an error being logged
//       fn add_animator() {
//         let model_data = get_test_model_data();
//         let other_model_data = get_test_model_data();
//         let mut model_animator = ModelAnimatorData::new(model_data);
//         let request = AnimationAction::AddAnimator(other_model_data);
//
//         let result = model_animator.run_request(request);
//
//         assert!(!result);
//       }
//
//       #[test]
//       fn remove_animator() {
//         let model_data = get_test_model_data();
//         let mut model_animator = ModelAnimatorData::new(model_data);
//         let request = AnimationAction::RemoveAnimatior;
//
//         let result = model_animator.run_request(request);
//
//         assert!(result);
//       }
//     }
//
//     #[test]
//     #[should_panic]
//     fn kill_thread_through_model() {
//       let model_data = get_test_model_data();
//       let mut model_animator = ModelAnimatorData::new(model_data);
//
//       let request = AnimationAction::KillThread;
//
//       model_animator.run_request(request);
//     }
//   }
//
//   #[cfg(test)]
//   mod model_animation_data_tests {
//     use super::*;
//
//     #[cfg(test)]
//     mod from_file_logic {
//       // use super::*;
//
//       #[test]
//       #[ignore]
//       fn empty_path() {}
//
//       #[test]
//       #[ignore]
//       fn invalid_extension() {}
//     }
//
//     #[cfg(test)]
//     mod animation_thread_logic {
//       use super::*;
//
//       // This test is a bit complicated.
//       //
//       // Basically, we start the thread and add two models to the animator list.
//       // If no animations are running, the thread should be sitting and waiting for a request.
//       // During this state, once it receives a request it only runs one of the requests queued.
//       //
//       // This means if we send in two animation requests while it's waiting, only one gets run.
//       // So after getting it to a state of waiting, we send an animation request for each model.
//       // If only one of the models changed, then we know that the thread was waiting because
//       //   it only ran one of those animation requests.
//       #[test]
//       // Chance of failing due to time sensitivity
//       fn thread_is_awaiting_requests() {
//         let mut screen = ScreenData::new();
//         let mut model_one = get_test_model_data();
//         let mut model_two = get_test_model_data();
//         let model_animation = get_test_animation(5);
//         log::info!("I am running test \"thread is awaiting requests\"");
//
//         // THIS IS TEMPORARY UNTIL MODEL ANIMATION FILES ARE IMPLEMENTED
//         {
//           model_one
//             .assign_model_animation(ModelAnimationData::new())
//             .unwrap();
//           model_two
//             .assign_model_animation(ModelAnimationData::new())
//             .unwrap();
//
//           model_one
//             .add_new_animation_to_list("test_animation".into(), model_animation.clone())
//             .unwrap();
//           model_two
//             .add_new_animation_to_list("test_animation".into(), model_animation)
//             .unwrap();
//         }
//         // THIS IS TEMPORARY UNTIL MODEL ANIMATION FILES ARE IMPLEMENTED
//
//         screen.start_animation_thread().unwrap();
//
//         model_one.start_animation(&screen).unwrap();
//         model_two.start_animation(&screen).unwrap();
//
//         screen.get_event_sync().wait_for_x_ticks(3);
//
//         model_one
//           .queue_next_animation("test_animation".into())
//           .unwrap();
//         model_two
//           .queue_next_animation("test_animation".into())
//           .unwrap();
//         // Wait for it to process the first animation request before killing the thread.
//         screen.get_event_sync().wait_for_x_ticks(1);
//
//         screen.stop_animation_thread().unwrap();
//
//         let model_one_appearance = model_one.get_sprite();
//         let model_two_appearance = model_two.get_sprite();
//
//         assert_ne!(model_one_appearance, model_two_appearance);
//       }
//
//       #[test]
//       // Chance of failing due to time sensitivity
//       fn thread_is_running_animations() {
//         let mut screen = ScreenData::new();
//         let mut model_one = get_test_model_data();
//         let mut model_two = get_test_model_data();
//         let model_animation = get_test_animation(5);
//         log::info!("I am running test \"thread is running animations\"");
//
//         let base_model_appearance = model_one.get_sprite();
//
//         // THIS IS TEMPORARY UNTIL MODEL ANIMATION FILES ARE IMPLEMENTED
//         {
//           model_one
//             .assign_model_animation(ModelAnimationData::new())
//             .unwrap();
//           model_two
//             .assign_model_animation(ModelAnimationData::new())
//             .unwrap();
//
//           model_one
//             .add_new_animation_to_list("test_animation".into(), model_animation.clone())
//             .unwrap();
//           model_two
//             .add_new_animation_to_list("test_animation".into(), model_animation)
//             .unwrap();
//         }
//         // THIS IS TEMPORARY UNTIL MODEL ANIMATION FILES ARE IMPLEMENTED
//
//         screen.start_animation_thread().unwrap();
//
//         model_one.start_animation(&screen).unwrap();
//         model_two.start_animation(&screen).unwrap();
//
//         model_one
//           .queue_next_animation("test_animation".into())
//           .unwrap();
//         model_two
//           .queue_next_animation("test_animation".into())
//           .unwrap();
//         // Wait for it to process the animation requests before checking for changes.
//         screen.get_event_sync().wait_for_x_ticks(5);
//
//         let model_one_appearance = model_one.get_sprite();
//         let model_two_appearance = model_two.get_sprite();
//
//         println!("{}", model_one_appearance);
//         println!("{}", model_two_appearance);
//
//         assert_ne!(model_one_appearance, base_model_appearance);
//         assert_ne!(model_two_appearance, base_model_appearance);
//
//         screen.stop_animation_thread().unwrap();
//       }
//     }
//   }
//
//   // data for tests
//
//   const WORLD_POSITION: (usize, usize) = (10, 10);
//
//   fn get_test_model_data() -> ModelData {
//     let test_model_path = std::path::Path::new("tests/models/test_square.model");
//
//     ModelData::from_file(test_model_path, WORLD_POSITION).unwrap()
//   }
//
//   // This is temporary until animation file parsers are a thing.
//   fn get_test_animation_limited_run_count() -> AnimationFrames {
//     let frames = vec![
//       AnimationFrame::new("lllll\nllall\nlllll".to_string(), 1, None),
//       AnimationFrame::new("mmmmm\nmmamm\nmmmmm".to_string(), 1, None),
//       AnimationFrame::new("nnnnn\nnnann\nnnnnn".to_string(), 1, None),
//     ];
//
//     AnimationFrames::new(frames, AnimationLoopCount::Limited(5))
//   }
//
//   // This is temporary until animation file parsers are a thing.
//   fn get_test_animation_unlimited_run_count() -> AnimationFrames {
//     let frames = vec![
//       AnimationFrame::new("ooooo\nooaoo\nooooo".to_string(), 1, None),
//       AnimationFrame::new("ppppp\nppapp\nppppp".to_string(), 1, None),
//       AnimationFrame::new("qqqqq\nqqaqq\nqqqqq".to_string(), 1, None),
//     ];
//
//     AnimationFrames::new(frames, AnimationLoopCount::Forever)
//   }
//
//   // This is temporary until animation file parsers are a thing.
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
