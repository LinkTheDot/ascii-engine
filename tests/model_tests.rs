use ascii_engine::prelude::*;

const WORLD_POSITION: (usize, usize) = (10, 10);
#[test]
fn move_to_logic() {
  let mut screen = ScreenData::new();
  let mut test_model = TestModel::new();

  screen.add_model(&test_model).unwrap();

  let expected_collisions = 0;
  let expected_position = ((CONFIG.grid_width + 1) as usize * 11) + 11;

  let collisions = test_model.move_to((11, 11));

  let new_model_position = test_model.get_position();

  assert_eq!(collisions.len(), expected_collisions);
  assert_eq!(new_model_position, expected_position);
}

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
