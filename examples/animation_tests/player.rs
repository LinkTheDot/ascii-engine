use crate::{animation_actor::*, result_traits::ResultTraits};
use ascii_engine::prelude::*;

pub struct Player {
  key: u64,
}

impl Player {
  pub const ANIMATION_NAME_SPIN: &str = "spin_animation";
  pub const NAME: &str = "player";
  pub const BASE_DIMENSIONS: (usize, usize) = (11, 5);

  pub fn new(key: u64) -> Self {
    Self { key }
  }

  pub fn new_model(position: (usize, usize)) -> ModelData {
    /*
      -----x-----
      ---x-x-x---
      --x-x-x-x--
      -xx-xcx-xx-
      xxxxxxxxxxx
    */
    let base_appearance = "-----x-----\n---x-x-x---\n--x-x-x-x--\n-xx-xcx-xx-\nxxxxxxxxxxx";
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
      Strata(100),
      Self::NAME.to_string(),
    )
    .unwrap();

    Self::apply_animations(&mut model);
    Self::apply_tags(&mut model);

    model
  }

  fn get_hitbox() -> Hitbox {
    let hitbox_dimensions = Rectangle::from((
      Self::BASE_DIMENSIONS.0 as u16,
      Self::BASE_DIMENSIONS.1 as u16,
    ));
    let index = 38;

    Hitbox::new(hitbox_dimensions, index)
  }

  fn apply_tags(model: &mut ModelData) {
    model.add_tags(vec![Self::NAME.to_string()]);
  }

  fn apply_animations(model: &mut ModelData) {
    let animations = Self::get_animations();
    let model_appearance = model.get_appearance_data();
    let mut model_appearance = model_appearance.lock().unwrap();

    animations.into_iter().for_each(|(name, frames)| {
      model_appearance.add_animation_to_model(name, frames);
    });
  }

  fn get_animations() -> Vec<(String, AnimationFrames)> {
    let spin_animation = Self::get_spin_animation();

    vec![spin_animation]
  }

  fn get_spin_animation() -> (String, AnimationFrames) {
    let frame_duration = 5; // ticks

    let animation_frame_appearances_left: Vec<(String, char)> = vec![
      "-----------\n-----------\n===========\n-----c-----\n-----------".to_string(),
      "-----------\n====-------\n----===----\n-----c-====\n-----------".to_string(),
      "==---------\n--==-------\n----===----\n-----c-==--\n---------==".to_string(),
      "-\\---------\n---\\-------\n----\\=\\----\n-----c-\\---\n---------\\-".to_string(),
      "---|-------\n---|-------\n----\\=\\----\n-----c-|---\n-------|---".to_string(),
    ]
    .into_iter()
    .map(|s| (s, '-'))
    .collect();
    let animation_frame_appearance_middle = (
      "-----|-----\n-----|-----\n-----|-----\n-----c-----\n-----|-----".to_string(),
      '|',
    );
    let mut animation_frame_appearances_right: Vec<(String, char)> =
      animation_frame_appearances_left
        .iter()
        .skip(1)
        .map(|(appearance, center_replacement)| {
          let appearance = appearance
            .split('\n')
            .map(|row| row.to_owned().reversed())
            .collect::<Vec<String>>()
            .join("\n")
            .replace('\\', "/");

          (appearance, *center_replacement)
        })
        .rev()
        .collect();
    let mut animation_frames: Vec<(String, char)> = animation_frame_appearances_left;
    animation_frames.push(animation_frame_appearance_middle);
    animation_frames.append(&mut animation_frame_appearances_right);
    log::info!("{:#?}", animation_frames);

    let loop_count = AnimationLoopCount::Forever;
    let animation_frames: Vec<AnimationFrame> = animation_frames
      .into_iter()
      .map(|(frame, replacement_center)| {
        let appearance = Sprite::new(frame, 'c', replacement_center, '-').unwrap();

        AnimationFrame::new(appearance, frame_duration)
      })
      .collect();

    let animation_frames = AnimationFrames::new(animation_frames, loop_count, None);

    (Self::ANIMATION_NAME_SPIN.to_string(), animation_frames)
  }

  pub fn get_key(&self) -> u64 {
    self.key
  }

  pub fn movement(&self, model_manager: &mut ModelManager, movement_input: &str) {
    let mut movement = (0, 0);

    match movement_input {
      "w" => movement.1 -= 1,
      "s" => movement.1 += 1,
      "a" => movement.0 -= 1,
      "d" => movement.0 += 1,
      _ => return,
    }

    let movement = ModelMovement::Relative(movement);

    let Some(collisions) = model_manager.move_model(&self.get_key(), movement).unwrap() else {
      return;
    };

    for model_key in collisions.collision_list {
      let Some(mut model_tags) = model_manager.get_tags_of_model(model_key) else {
        continue;
      };

      if model_tags.remove(AnimationActor::NAME) {
        let animation_names: Vec<String> = model_tags.into_iter().collect();
        let Some(animation_name) = animation_names.get(0) else {
          continue;
        };

        if animation_name == AnimationActor::TAG_CLEAR_ANIMATIONS {
          model_manager
            .clear_model_animation_queue(&self.get_key())
            .log_if_err();

          continue;
        }

        model_manager
          .queue_model_animation(&self.get_key(), animation_name, false)
          .log_if_err();
      }
    }
  }
}

trait ReverseString {
  fn reversed(&self) -> Self;
}

impl ReverseString for String {
  fn reversed(&self) -> Self {
    self.chars().rev().collect()
  }
}
