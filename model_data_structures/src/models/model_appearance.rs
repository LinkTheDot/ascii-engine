use crate::models::animation::*;
use crate::models::errors::*;
use serde::{Deserialize, Serialize};
use sprites::*;

pub mod sprites;

#[derive(Debug, Clone, Deserialize, Serialize)]
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

  pub fn queue_model_animation(&mut self, animation_name: &str) -> Result<(), AnimationError> {
    self
      .get_mut_animation_data()
      .queue_animation(animation_name)
  }

  /// Stops the currently running animation, and replaces it with the one passed in.
  ///
  /// # Errors
  ///
  /// - An animation of that name doesn't exist, and therefore cannot be added to the queue.
  pub fn overwrite_current_model_animation(
    &mut self,
    new_animation_name: &str,
  ) -> Result<(), AnimationError> {
    self
      .get_mut_animation_data()
      .overwrite_current_model_animation(new_animation_name)
  }

  /// If an animation with that name already exists, it is returned.
  pub fn add_animation_to_model(
    &mut self,
    animation_name: String,
    new_animation: AnimationFrames,
  ) -> Option<AnimationFrames> {
    self
      .get_mut_animation_data()
      .add_new_animation_to_list(animation_name, new_animation)
  }

  /// Removes the animation of the given name and returns its name and data if it existed.
  pub fn remove_animation_from_list(&mut self, animation_name: &str) -> Option<AnimationFrames> {
    self
      .get_mut_animation_data()
      .remove_animation_from_list(animation_name)
  }

  /// Removes every animation running in the animation queue.
  ///
  /// The last existing animation to be run in the list is assigned to the last_run_animation.
  pub fn clear_model_animation_queue(&mut self) {
    self.get_mut_animation_data().clear_animation_queue();
  }

  /// Removes the currently running animation from the queue.
  pub fn stop_current_model_animation(&mut self) {
    self
      .get_mut_animation_data()
      .remove_current_model_animation_from_queue()
  }

  /// Checks every sprite in every animation of self and ensures they have no errors.
  ///
  /// If any errors are found, the animation names and data about what's wrong with them is returned.
  /// If the default sprite contains errors, its name will be in the list of errors as "Default Sprite".
  ///
  /// # Sprite Errors
  ///
  /// - The stored shape isn't rectangular.
  /// - The stored shape doesn't have an anchor.
  /// - The stored shape has multiple anchors.
  /// - The anchor and air characters are the same.
  pub fn full_validity_check(&mut self) -> Result<(), ModelError> {
    if let Some(animation_data) = self.animation_data.as_ref() {
      let mut errors: Vec<AnimationValidityErrorData> = animation_data
        .get_animation_list()
        .iter()
        .filter_map(|(animation_name, animation)| {
          if let Err(error_data) = animation.validity_check(animation_name) {
            Some(error_data)
          } else {
            None
          }
        })
        .collect();

      if let Err(ModelError::SpriteValidityChecks(error_list)) =
        self.default_sprite.validity_check()
      {
        let default_sprite_error_data = AnimationValidityErrorData {
          animation_name: "Default Sprite".into(),
          resting_appearance_errors: None,
          invalid_frame_errors: vec![(0, error_list)],
        };

        errors.push(default_sprite_error_data);
      }

      if !errors.is_empty() {
        return Err(AnimationError::AnimationValidityCheckFailed(errors).into());
      }
    }

    Ok(())
  }

  /// Returns a mutable reference to the contained [`ModelAnimationData`](crate::models::animation::ModelAnimationData)
  ///
  /// If there was none stored, it's created.
  fn get_mut_animation_data(&mut self) -> &mut ModelAnimationData {
    if self.animation_data.is_none() {
      self.animation_data = Some(ModelAnimationData::default());
    }

    self.animation_data.as_mut().unwrap()
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
