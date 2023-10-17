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
  animation_data: Option<RefCell<ModelAnimationData>>,
}

impl ModelAppearance {
  pub fn new(sprite: Sprite, animation_data: Option<ModelAnimationData>) -> Self {
    let animation_data = animation_data.map(RefCell::new);

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
    self
      .animation_data
      .as_ref()
      .map(|old_data| old_data.replace(new_animation_data))
  }

  pub fn get_appearance(&self) -> &Sprite {
    // Check if there's an animation running, check how long it's been running
    //   since the animation started.
    //   Adjust the queue of animations if that one expired based on how long the
    //   previous animation was running.
    // If no animation is running, get the last idle frame for an animation. If there was
    //   no idle frame, use the base_sprite.
    //
    // The time on an animation should be a RefCell so as to not require &mut self for this method.
    todo!()
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
