use ascii_engine::prelude::*;
use std::sync::{Arc, Mutex};

const SHAPE: &str = "x-x\nxcx\nx-x";
const ANCHOR_CHAR: char = 'c';
const ANCHOR_REPLACEMENT_CHAR: char = '-';
const AIR_CHAR: char = '-';
const MODEL_NAME: &str = "rectangle";

#[test]
fn display_logic() {
  let screen = ScreenData::default();
  // adding the height - 1 is accounting for new lines
  let expected_pixel_count =
    ((CONFIG.grid_width * CONFIG.grid_height) + CONFIG.grid_height - 1) as usize;
  let display = screen.display().unwrap();

  assert_eq!(display.chars().count(), expected_pixel_count);
}

#[cfg(test)]
mod model_storage_tests {
  use super::*;

  #[test]
  fn insert_valid_model_data() {
    let model_data = Arc::new(Mutex::new(get_model_data((5, 5))));
    let model = Square::new(model_data);
    let mut model_storage = Models::new();

    let insert_result = model_storage.insert(&model.get_unique_hash(), &model);

    assert!(insert_result.is_ok());
  }

  #[test]
  fn insert_invalid_model_data() {
    let model_data = Arc::new(Mutex::new(get_model_data((5, 5))));
    model_data.lock().unwrap().change_strata(Strata(101));
    let model = Square::new(model_data);
    let mut model_storage = Models::new();

    let expected_result = Err(ModelError::IncorrectStrataRange(Strata(101)));

    let insert_result = model_storage.insert(&model.get_unique_hash(), &model);

    assert_eq!(insert_result, expected_result);
  }

  #[test]
  #[ignore]
  fn get_logic() {
    // let model_data = Arc::new(Mutex::new(get_model_data((5, 5))));
    // let model = Square::new(model_data);
    // let mut model_storage = Models::new();
    // model_storage
    //   .insert(model.get_unique_hash(), &model)
    //   .unwrap();
    //
    // let expected_data = get_model_data((5, 5));
    //
    // // Gets just the model data inside from all the nesting.
    // // This is required because neither Mutex or MutexGuard implement Eq.
    // let inside_data = model_storage
    //   .get_strata_keys(&Strata(0))
    //   .unwrap()
    //   .get(&model.get_unique_hash())
    //   .unwrap()
    //   .lock()
    //   .unwrap();
    //
    // assert_eq!(*inside_data, expected_data);
  }
}

//
// -- Data for tests below --
//

#[derive(DisplayModel)]
struct Square {
  model_data: Arc<Mutex<ModelData>>,
}

impl Square {
  pub fn new(model_data: Arc<Mutex<ModelData>>) -> Self {
    Self { model_data }
  }
}

fn get_model_data(world_position: (usize, usize)) -> ModelData {
  let sprite = get_sprite();
  let hitbox = get_hitbox();
  let strata = Strata(0);

  ModelData::new(
    world_position,
    sprite,
    hitbox,
    strata,
    MODEL_NAME.to_string(),
  )
  .unwrap()
}

fn get_sprite() -> Sprite {
  let skin = get_skin();

  Sprite::new(skin).unwrap()
}

fn get_skin() -> Skin {
  Skin::new(SHAPE, ANCHOR_CHAR, ANCHOR_REPLACEMENT_CHAR, AIR_CHAR).unwrap()
}

fn get_hitbox() -> HitboxCreationData {
  let shape = "xxx\n-c-";

  HitboxCreationData::new(shape, 'c')
}
