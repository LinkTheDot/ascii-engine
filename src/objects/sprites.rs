use crate::objects::errors::*;

#[allow(unused)]
use log::debug;

/// The Sprite is data about the display and hitbox side of an object.
///
/// The Sprite will contain how an object will look, where it's Hitbox will be, and
/// what character in the skin of the object should be classified as "air".
#[derive(Debug, PartialEq, Eq)]
pub struct Sprite {
  skin: Skin,
}

/// The Skin is how an object will appear on the screen.
///
/// When creating a skin's shape, center and air characters will need to be designated.
/// The center character will be replaced with the 'center_replacement_character' field when
/// building the shape of the Skin.
#[derive(Debug, PartialEq, Eq)]
pub struct Skin {
  pub shape: String,
  pub center_character: char,
  pub center_replacement_character: char,
  pub air_character: char,
  /// Doesn't count new lines
  center_character_index: usize,
}

impl Sprite {
  // OBJECT CREATION WILL CHANGE TO A FILE FORMAT
  pub fn new(mut skin: Skin) -> Result<Self, ObjectError> {
    skin.fix_skin();

    Ok(Self { skin })
  }

  pub fn get_center_character_index(&self) -> usize {
    self.skin.center_character_index
  }

  /// Returns a reference to the skin's shape
  pub fn get_shape(&self) -> &str {
    &self.skin.shape
  }

  /// Returns a mutable reference to the skin's shape
  pub fn get_mut_shape(&mut self) -> &mut String {
    &mut self.skin.shape
  }

  // /// Returns a reference to the relative points of the hitbox to
  // /// the designated center point of the object's skin.
  // pub fn get_hitbox(&self) -> &Vec<(isize, isize)> {
  //   &self.hitbox
  // }

  // /// Replaces the object's hitbox with a new one
  // pub fn change_hitbox(&mut self, new_hitbox: HitboxCreationData) -> Result<(), ObjectError> {
  //   match new_hitbox.get_hitbox_data() {
  //     Ok(hitbox_data) => self.hitbox = hitbox_data,
  //     Err(error) => return Err(error),
  //   }
  //
  //   Ok(())
  // }

  pub fn air_character(&self) -> char {
    self.skin.air_character
  }
}

impl Skin {
  // OBJECT CREATION WILL CHANGE TO A FILE FORMAT
  pub fn new(
    shape: &str,
    center_character: char,
    center_replacement_character: char,
    air_character: char,
  ) -> Result<Self, ObjectError> {
    let cleaned_shape = shape.replace('\n', "");
    let center_character_index = cleaned_shape
      .chars()
      .position(|pixel| pixel == center_character);

    match center_character_index {
      None => Err(ObjectError::NoCenter),
      Some(center_character_index) => Ok(Self {
        shape: shape.to_string(),
        center_character,
        center_replacement_character,
        air_character,
        center_character_index,
      }),
    }
  }

  /// Replaces the center character in the skin's shape with the given
  /// replacement character.
  fn fix_skin(&mut self) {
    self.shape = self.shape.replace(
      &self.center_character.to_string(),
      &self.center_replacement_character.to_string(),
    );
  }

  pub fn get_center_character_index(&self) -> usize {
    self.center_character_index
  }
}

#[cfg(test)]
mod skin_logic {
  use super::*;

  #[test]
  fn fix_skin_logic() {
    let mut skin = Skin::new("x-x\nxcx\nx-x", 'c', '-', '-').unwrap();

    let expected_shape = "x-x\nx-x\nx-x";

    skin.fix_skin();

    assert_eq!(skin.shape, expected_shape)
  }
}

// #[cfg(test)]
// mod get_hitbox_data_logic {
//   use super::*;
//
//   #[test]
//   fn valid_data_center_is_hitbox() {
//     let hitbox = get_hitbox_data(true);
//
//     // xxx
//     //  x  < this x is the center character
//     let expected_hitbox_data = Ok(vec![(-1, -1), (0, -1), (1, -1), (0, 0)]);
//
//     let hitbox_data = hitbox.get_hitbox_data();
//
//     assert_eq!(hitbox_data, expected_hitbox_data);
//   }
//
//   #[test]
//   fn valid_data_center_is_not_hitbox() {
//     let hitbox = get_hitbox_data(false);
//
//     // xxx
//     //  c  < this center character is not apart of the hitbox
//     let expected_hitbox_data = Ok(vec![(-1, -1), (0, -1), (1, -1)]);
//
//     let hitbox_data = hitbox.get_hitbox_data();
//
//     assert_eq!(hitbox_data, expected_hitbox_data);
//   }
//
//   #[test]
//   fn invalid_shape() {
//     let mut hitbox = get_hitbox_data(true);
//     hitbox.shape = "a-s-d-qwf-e-ff\n\n\nwe-gwe-w-vwea\nasd\n".to_string();
//
//     let expected_error = Err(ObjectError::NonRectangularShape);
//
//     let hitbox_data = hitbox.get_hitbox_data();
//
//     assert_eq!(hitbox_data, expected_error);
//   }
//
//   #[test]
//   fn no_center_character() {
//     let mut hitbox = get_hitbox_data(true);
//     hitbox.shape = "".to_string();
//
//     let expected_error = Err(ObjectError::EmptyHitboxString);
//
//     let hitbox_data = hitbox.get_hitbox_data();
//
//     assert_eq!(hitbox_data, expected_error);
//   }

// fn get_hitbox_data(center_is_hitbox: bool) -> HitboxCreationData {
//   let shape = "xyz\n-c-";
//   let center_character = 'c';
//   let air_character = '-';
//
//   HitboxCreationData::new(shape, center_character, air_character, center_is_hitbox)
// }
// }
