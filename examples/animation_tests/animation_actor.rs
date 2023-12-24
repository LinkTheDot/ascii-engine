#![allow(unused)]

use ascii_engine::prelude::*;

use crate::result_traits::ResultTraits;

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

    Self::apply_animations(&mut model);
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
    let animations = vec![Self::get_interacted_animation()];
    let model_appearance = model.get_appearance_data();
    let mut model_appearance = model_appearance.lock().unwrap();

    animations.into_iter().for_each(|(name, frames)| {
      model_appearance.add_animation_to_model(name, frames);
    });
  }

  fn get_interacted_animation() -> (String, AnimationFrames) {
    let frame_duration = 5; // ticks

    let animation_frames: Vec<(String, char, u32)> = vec![
      (
        "/=============\\\n|*************|\n|*-----------*|\n|*-----c-----*|\n|*-----------*|\n|*************|\n\\=============/".to_string(), 
        '-',
        frame_duration
      ),
      (
        "/-------------\\\n|=============|\n|*-----------*|\n|*-----c-----*|\n|*-----------*|\n|*************|\n\\=============/".to_string(), 
        '-',
        frame_duration
      ),
      (
        "/-------------\\\n|-------------|\n|=============|\n|*-----c-----*|\n|*-----------*|\n|*************|\n\\=============/".to_string(), 
        '-',
        frame_duration
      ),
      (
        "/-------------\\\n|-------------|\n|=============|\n|*-----c-----*|\n|*-----------*|\n|*************|\n\\=============/".to_string(), 
        '-',
        frame_duration
      ),
      (
        "/-------------\\\n|-------------|\n|-------------|\n|======c======|\n|*-----------*|\n|*************|\n\\=============/".to_string(),
        '=',
        frame_duration
      ),
      (
        "/-------------\\\n|-------------|\n|-------------|\n|------c------|\n|=============|\n|*************|\n\\=============/".to_string(),
        '-',
        frame_duration
      ),
      (
        "/-------------\\\n|-------------|\n|-------------|\n|------c------|\n|-------------|\n|=============|\n\\=============/".to_string(),
        '-',
        frame_duration
      ),
      (
        "/-------------\\\n|-------------|\n|-------------|\n|------c------|\n|-------------|\n|-------------|\n\\=============/".to_string(),
        '-',
        10
      ),
    ];

    let mut animation_frames: Vec<AnimationFrame> = animation_frames
      .into_iter()
      .map(|(frame, replacement_center, frame_duration)| {
        let appearance = Sprite::new(frame, 'c', replacement_center, '-').unwrap();

        AnimationFrame::new(appearance, frame_duration)
      })
      .collect();

    let mut reversed_animation_frames: Vec<AnimationFrame> =
      animation_frames.clone().into_iter().skip(1).rev().collect();

    animation_frames.append(&mut reversed_animation_frames);

    let loop_count = AnimationLoopCount::Limited(1);
    let animation_frames = AnimationFrames::new(animation_frames, loop_count, None);

    (Self::ANIMATION_NAME_BASE.into(), animation_frames)
  }

  pub fn activate_animation(model_hash: &u64, model_manager: &mut ModelManager) {
    model_manager
      .queue_model_animation(model_hash, Self::ANIMATION_NAME_BASE, false)
      .log_if_err();
  }
}
