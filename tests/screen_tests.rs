use ascii_engine::prelude::*;

#[cfg(test)]
mod display_logic {
  use super::*;

  #[tokio::test]
  async fn empty_screen() {
    let screen = ScreenData::default();
    // adding the height - 1 is accounting for new lines
    let expected_pixel_count =
      ((CONFIG.grid_width * CONFIG.grid_height) + CONFIG.grid_height - 1) as usize;
    let display = screen.display();

    assert_eq!(display.chars().count(), expected_pixel_count);
  }

  #[tokio::test]
  async fn with_model() {
    let mut screen = ScreenData::new();
    let test_model = TestModel::new();

    let expected_pixel_count =
      ((CONFIG.grid_width * CONFIG.grid_height) + CONFIG.grid_height - 1) as usize;

    screen.add_model(&test_model).unwrap();

    let display = screen.display();

    assert_eq!(display.chars().count(), expected_pixel_count);
  }
}

#[tokio::test]
async fn add_and_remove_model() {
  let mut screen = ScreenData::new();
  let test_model = TestModel::new();

  screen.add_model(&test_model).unwrap();

  let test_model_hash = test_model.get_unique_hash();

  let result_data = screen.remove_model(&test_model_hash).unwrap();

  assert_eq!(result_data.get_unique_hash(), test_model_hash);
}

#[tokio::test]
async fn printer_started() {
  let screen = ScreenData::new();

  assert!(!screen.printer_started());
}

#[cfg(test)]
mod get_animation_connection_logic {
  use super::*;

  #[tokio::test]
  async fn animation_not_started() {
    let screen = ScreenData::new();

    let result = screen.get_animation_connection();

    assert!(result.is_none());
  }

  #[tokio::test]
  async fn animation_is_started() {
    let mut screen = ScreenData::new();
    screen.start_animation_thread().await.unwrap();

    let result = screen.get_animation_connection();

    assert!(result.is_some());
  }
}

#[cfg(test)]
mod start_animation_thread_logic {
  use super::*;

  #[tokio::test]
  async fn starting_once() {
    let mut screen = ScreenData::new();

    let result = screen.start_animation_thread().await;

    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn starting_multiple_times() {
    let mut screen = ScreenData::new();
    screen.start_animation_thread().await.unwrap();

    let expected_result = Err(ScreenError::AnimationError(
      AnimationError::AnimationThreadAlreadyStarted,
    ));

    let result = screen.start_animation_thread().await;

    assert_eq!(result, expected_result);
  }
}

#[tokio::test]
async fn get_event_sync_logic() {
  let screen = ScreenData::new();

  let expected_elapsed_time_low = 23500;
  let expected_elapsed_time_high = 24500;

  let event_sync = screen.get_event_sync();

  // run the test 50 times
  for _ in 0..50 {
    event_sync.wait_for_tick();
    let now = std::time::Instant::now();

    event_sync.wait_for_tick();

    let elapsed_time = now.elapsed().as_micros();

    // check if the elapsed time is 24ms +- 0.5ms;
    assert!(
      expected_elapsed_time_low <= elapsed_time && expected_elapsed_time_high >= elapsed_time
    );
  }
}

//
// -- Data for tests below --
//

const WORLD_POSITION: (usize, usize) = (10, 10);

#[derive(DisplayModel)]
struct TestModel {
  model_data: ModelData,
}

impl TestModel {
  fn new() -> Self {
    let test_model_path = std::path::Path::new("tests/models/test_square.model");
    let model_data = ModelData::from_file(test_model_path, WORLD_POSITION).unwrap();

    Self { model_data }
  }
}
