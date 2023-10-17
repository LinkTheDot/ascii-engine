// #![cfg(test)]
//
// use ascii_engine::prelude::*;
// use model_data_structures::models::testing_data::TestingData;
//
// const WORLD_POSITION: (usize, usize) = (10, 10);
//
// #[cfg(test)]
// mod display_logic {
//   use super::*;
//
//   #[test]
//   fn empty_screen() {
//     let screen = ScreenData::default();
//     // adding the height - 1 is accounting for new lines
//     let expected_pixel_count =
//       ((CONFIG.grid_width * CONFIG.grid_height) + CONFIG.grid_height - 1) as usize;
//     let display = screen.display();
//
//     assert_eq!(display.chars().count(), expected_pixel_count);
//   }
//
//   #[test]
//   fn with_model() {
//     let mut screen = ScreenData::new();
//     let test_model = TestingData::new_test_model(WORLD_POSITION);
//
//     let expected_pixel_count =
//       ((CONFIG.grid_width * CONFIG.grid_height) + CONFIG.grid_height - 1) as usize;
//
//     screen.add_model(test_model).unwrap();
//
//     let display = screen.display();
//
//     assert_eq!(display.chars().count(), expected_pixel_count);
//   }
//
//   #[test]
//   fn get_screen_printer_logic() {
//     let screen = ScreenData::new();
//     let _screen_printer = screen.get_screen_printer();
//   }
// }
//
// #[test]
// fn add_and_remove_model() {
//   let mut screen = ScreenData::new();
//   let test_model = TestingData::new_test_model(WORLD_POSITION);
//   let test_model_hash = test_model.get_hash();
//
//   screen.add_model(test_model).unwrap();
//
//   let result_data = screen.remove_model(&test_model_hash).unwrap();
//
//   assert_eq!(result_data.get_hash(), test_model_hash);
// }
//
// #[cfg(test)]
// mod start_animation_thread_logic {
//   use super::*;
//
//   #[test]
//   fn starting_once() {
//     let mut screen = ScreenData::new();
//
//     let result = screen.start_animation_thread();
//
//     assert!(result.is_ok());
//   }
//
//   #[test]
//   fn starting_multiple_times() {
//     let mut screen = ScreenData::new();
//     screen.start_animation_thread().unwrap();
//
//     let expected_result = Err(ScreenError::AnimationError(
//       AnimationError::AnimationThreadAlreadyStarted,
//     ));
//
//     let result = screen.start_animation_thread();
//
//     assert_eq!(result, expected_result);
//   }
// }
//
// #[test]
// fn get_event_sync_logic() {
//   const RUN_COUNT: usize = 50;
//
//   let screen = ScreenData::new();
//
//   let expected_elapsed_time_low = 23500;
//   let expected_elapsed_time_high = 24500;
//
//   let event_sync = screen.get_event_sync();
//
//   let mut sucess_count = 0;
//
//   // run the test 50 times
//   for _ in 0..RUN_COUNT {
//     event_sync.wait_for_tick();
//     let now = std::time::Instant::now();
//
//     event_sync.wait_for_tick();
//
//     let elapsed_time = now.elapsed().as_micros();
//
//     // check if the elapsed time is 24ms +- 0.5ms;
//     if expected_elapsed_time_low <= elapsed_time && expected_elapsed_time_high >= elapsed_time {
//       sucess_count += 1;
//     }
//   }
//
//   // check for a 90% sucess rate
//   assert!(sucess_count >= (RUN_COUNT as f32 * 0.9) as usize);
// }
//
// #[test]
// fn start_and_stop_animation_thread() {
//   let mut screen = ScreenData::new();
//
//   screen.start_animation_thread().unwrap();
//
//   let started_state = screen.animation_thread_started();
//
//   screen.stop_animation_thread().unwrap();
//
//   let stopped_state = screen.animation_thread_started();
//
//   assert!(started_state);
//   assert!(!stopped_state);
// }
//
// #[test]
// #[should_panic]
// fn stop_stopped_animation_thread() {
//   let mut screen = ScreenData::new();
//
//   screen.stop_animation_thread().unwrap();
// }
//
// #[cfg(test)]
// mod world_management_logic {
//   use std::path::PathBuf;
//
//   use ascii_engine::screen::stored_worlds::StoredWorld;
//
//   use super::*;
//
//   #[test]
//   fn reset_world_logic() {
//     let test_world = StoredWorld::load(PathBuf::from("tests/worlds/test_template.world")).unwrap();
//     let test_world_hashes = test_world.get_model_hashes();
//     let mut screen_data = ScreenData::from_world(test_world);
//
//     let reset_world = screen_data.reset_world();
//     let reset_world_hashes = reset_world.get_model_hashes();
//
//     assert_eq!(test_world_hashes, reset_world_hashes)
//   }
//
//   #[test]
//   fn load_world_logic() {
//     let test_world = StoredWorld::load(PathBuf::from("tests/worlds/test_template.world")).unwrap();
//     let mut screen_data = ScreenData::new();
//     let model_manager = screen_data.get_model_manager();
//
//     let empty_world = screen_data.load_world(test_world);
//     assert!(empty_world.get_model_hashes().is_empty());
//
//     model_manager.get_model_list(|model_list| {
//       assert!(model_list.keys().count() == 5);
//     });
//   }
// }
