use crate::models::animation::*;
use crate::models::errors::*;
use sprites::*;
use std::cell::RefCell;

pub mod sprites;

#[derive(Debug, Clone)]
pub struct ModelAppearance {
  /// The default appearance the model will use if there's no animation data.
  default_sprite: Sprite,
  /// This is created when parsing a model.
  ///
  /// None if there was no `.animate` file in the same path of the model, or there was no alternative path given.
  animation_data: Option<ModelAnimationData>,
}

impl ModelAppearance {
  pub fn new(sprite: Sprite, animation_data: Option<ModelAnimationData>) -> Self {
    Self {
      default_sprite: sprite,
      animation_data,
    }
  }

  /// Replaces the currently stored animation data, returns the previous one if it existed.
  pub fn add_animation_data(
    &mut self,
    new_animation_data: ModelAnimationData,
  ) -> Option<ModelAnimationData> {
    std::mem::replace(&mut self.animation_data, Some(new_animation_data))
  }

  pub fn get_appearance(&self) -> &Sprite {
    if let Some(animation_data) = &self.animation_data {
      if let Some(current_appearance) = animation_data.get_current_appearance() {
        return current_appearance;
      }
    }

    &self.default_sprite
  }

  /// Gets the default sprite, ignoring all animation data that may or may not be running.
  ///
  /// If there is no animation data, the default will be returned from [`get_appearance`](ModelAppearance::get_appearance).
  pub fn get_default_appearance(&self) -> &Sprite {
    &self.default_sprite
  }

  /// Replaces the default sprite and returns the old one.
  ///
  /// Default sprites will be used when:
  ///   - There's no animation_data
  ///   - No animations have been run since creation of the animation_data.
  ///   - The model_animator has been refreshed, clearing all data about previously run animations.
  pub fn update_default_sprite(&mut self, new_sprite: Sprite) -> Sprite {
    std::mem::replace(&mut self.default_sprite, new_sprite)
  }

  pub fn queue_model_animation(&mut self, _animation_name: &str) -> Result<(), AnimationError> {
    todo!()
  }

  pub fn overwrite_current_model_animation(
    &mut self,
    _new_animation_name: &str,
  ) -> Result<(), AnimationError> {
    todo!()
  }

  /// If an animation with that name already exists, it is returned.
  pub fn add_animation_to_model(
    &mut self,
    _new_animation: AnimationFrames,
    _animation_name: String,
  ) -> Option<AnimationFrames> {
    todo!()
  }

  pub fn clear_model_animation_queue(&mut self) -> Result<(), AnimationError> {
    todo!()
  }
}

#[cfg(test)]
mod tests {
  // use super::*;
  // use crate::models::testing_data::*;
  //
  // #[test]
  // fn api_testing() {
  // let sprite_appearance = TestingData::get_frame_appearance('x');
  // let sprite = Sprite::new(sprite_appearance, 'a', 'x', '-').unwrap();
  // let animation_data = TestingData::get_test_model_animation_data();
  // let mut model_appearance = ModelAppearance::new(sprite, Some(animation_data));
  // }
}
