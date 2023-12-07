#![allow(unused)]

use ascii_engine::prelude::*;

pub struct AnimationActor;

impl AnimationActor {
  pub const NAME: &str = "animation_actor";
  pub const ANIMATION_NAME_BASE: &str = "basic_animation";
  pub const TAG_CLEAR_ANIMATIONS: &str = "clear_animations";

  pub const BASE_DIMENSIONS: (usize, usize) = (15, 7);

  pub fn new_model(animation_names: Vec<String>, position: (usize, usize)) -> ModelData {
    /*
      /=============\
      |*************|
      |*-----------*|
      |*-----c-----*|
      |*-----------*|
      |*************|
      \=============/
    */
    let base_appearance = "/=============\\\n|*************|\n|*-----------*|\n|*-----c-----*|\n|*-----------*|\n|*************|\n\\=============/";
    let air_character = '-';
    let anchor_character = 'c';
    let anchor_replacement_character = air_character;
    let base_appearance = Sprite::new(
      base_appearance,
      anchor_character,
      anchor_replacement_character,
      air_character,
    )
    .unwrap();
    let hitbox = Self::get_hitbox();
    let mut model = ModelData::new(
      position,
      base_appearance,
      hitbox,
      Strata(0),
      Self::NAME.to_string(),
    )
    .unwrap();

    // Self::apply_animations(&mut model);
    Self::apply_tags(&mut model, animation_names);

    model
  }

  fn get_hitbox() -> Hitbox {
    let hitbox_dimensions = Rectangle::from((
      Self::BASE_DIMENSIONS.0 as u16,
      Self::BASE_DIMENSIONS.1 as u16,
    ));
    let index = 52;

    Hitbox::new(hitbox_dimensions, index)
  }

  fn apply_tags(model: &mut ModelData, mut animation_tags: Vec<String>) {
    animation_tags.push(Self::NAME.to_string());

    model.add_tags(animation_tags);
  }

  fn apply_animations(model: &mut ModelData) {
    todo!()
  }

  fn get_base_animation() -> (String, AnimationFrames) {
    todo!()
  }
}
