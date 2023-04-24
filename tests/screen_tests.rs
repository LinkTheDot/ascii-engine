use ascii_engine::prelude::*;

const WORLD_POSITION: (usize, usize) = (10, 10);

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

//
// -- Data for tests below --
//

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
