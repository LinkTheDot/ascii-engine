use crate::animation_actor::*;
use crate::player::*;
use crate::result_traits::ResultTraits;
use ascii_engine::prelude::*;
use std::path::Path;
use std::path::PathBuf;

pub const WORLD_PATH: &str = "examples/animation_tests/world.wrld";

pub fn initialize() -> (ScreenData, Player) {
  // Create screen before everything to start logging.
  let mut screen = ScreenData::new();

  let world = if let Ok(world) = StoredWorld::load(Path::new(WORLD_PATH)) {
    world
  } else {
    create_world()
  };

  screen.load_world(world);

  let model_manager = screen.get_model_manager();

  let player_hash = model_manager
    .get_models_with_tags(vec![Player::NAME])
    .remove(0);
  let player = Player::new(player_hash);

  (screen, player)
}

fn create_world() -> StoredWorld {
  let model_list: Vec<ModelData> = vec![
    Player::new_model((10, 10)),
    AnimationActor::new_model(vec![Player::ANIMATION_NAME_SPIN.to_string()], (50, 20)),
    AnimationActor::new_model(
      vec![AnimationActor::TAG_CLEAR_ANIMATIONS.to_string()],
      (50, 30),
    ),
  ];

  StoredWorld::new(model_list)
}

pub fn save_world(screen: &ScreenData) {
  screen
    .copy_current_world()
    .save(PathBuf::from(WORLD_PATH))
    .log_if_err();
}

pub fn delete_world_save() {
  std::fs::remove_file(WORLD_PATH).log_if_err();
}
