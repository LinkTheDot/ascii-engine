use ascii_engine::prelude::*;
use std::sync::{Arc, Mutex};

const SHAPE: &str = "x-x\nxcx\nx-x";
const CENTER_CHAR: char = 'c';
const CENTER_REPLACEMENT_CHAR: char = '-';
const AIR_CHAR: char = '-';
const OBJECT_NAME: &str = "rectangle";

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
mod object_storage_tests {
  use super::*;

  #[test]
  fn insert_valid_object_data() {
    let object_data = Arc::new(Mutex::new(get_object_data((5, 5))));
    let object = Square::new(object_data);
    let mut object_storage = Objects::new();

    let insert_result = object_storage.insert(&object.get_unique_hash(), &object);

    assert!(insert_result.is_ok());
  }

  #[test]
  fn insert_invalid_object_data() {
    let object_data = Arc::new(Mutex::new(get_object_data((5, 5))));
    object_data.lock().unwrap().change_strata(Strata(101));
    let object = Square::new(object_data);
    let mut object_storage = Objects::new();

    let expected_result = Err(ObjectError::IncorrectStrataRange(Strata(101)));

    let insert_result = object_storage.insert(&object.get_unique_hash(), &object);

    assert_eq!(insert_result, expected_result);
  }

  #[test]
  #[ignore]
  fn get_logic() {
    // let object_data = Arc::new(Mutex::new(get_object_data((5, 5))));
    // let object = Square::new(object_data);
    // let mut object_storage = Objects::new();
    // object_storage
    //   .insert(object.get_unique_hash(), &object)
    //   .unwrap();
    //
    // let expected_data = get_object_data((5, 5));
    //
    // // Gets just the object data inside from all the nesting.
    // // This is required because neither Mutex or MutexGuard implement Eq.
    // let inside_data = object_storage
    //   .get_strata_keys(&Strata(0))
    //   .unwrap()
    //   .get(&object.get_unique_hash())
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

#[derive(Object)]
struct Square {
  object_data: Arc<Mutex<ObjectData>>,
}

impl Square {
  pub fn new(object_data: Arc<Mutex<ObjectData>>) -> Self {
    Self { object_data }
  }
}

fn get_object_data(object_position: (usize, usize)) -> ObjectData {
  let sprite = get_sprite();
  let hitbox = get_hitbox();
  let strata = Strata(0);

  ObjectData::new(
    object_position,
    sprite,
    hitbox,
    strata,
    OBJECT_NAME.to_string(),
  )
  .unwrap()
}

fn get_sprite() -> Sprite {
  let skin = get_skin();

  Sprite::new(skin).unwrap()
}

fn get_skin() -> Skin {
  Skin::new(SHAPE, CENTER_CHAR, CENTER_REPLACEMENT_CHAR, AIR_CHAR).unwrap()
}

fn get_hitbox() -> HitboxCreationData {
  let shape = "xxx\n-c-";

  HitboxCreationData::new(shape, 'c')
}
