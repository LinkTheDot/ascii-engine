// use engine_math::rectangle::*;
// use model_data_structures::models::errors::*;
//
// /// The ``Sprite`` contains the data for how a model looks on the screen.
// #[derive(Debug, PartialEq, Eq)]
// pub struct Sprite {
//   pub shape: String,
//   pub anchor_character: char,
//   pub anchor_replacement_character: char,
//   pub air_character: char,
//   /// Doesn't count new lines
//   anchor_character_index: usize,
// }
//
// impl Sprite {
//   /// Converts a Skin into a Sprite
//   pub fn new(
//     shape: &str,
//     anchor_character: char,
//     anchor_replacement_character: char,
//     air_character: char,
//   ) -> Result<Self, ModelError> {
//     if !Rectangle::string_is_valid_rectangle(shape) {
//       return Err(ModelError::NonRectangularShape);
//     }
//
//     let mut shape = shape.to_string();
//
//     let anchor_character_index = Self::get_anchor_index(&shape, anchor_character)?;
//     Self::fix_shape(&mut shape, anchor_character, anchor_replacement_character);
//
//     Ok(Self {
//       shape,
//       anchor_character,
//       anchor_replacement_character,
//       air_character,
//       anchor_character_index,
//     })
//   }
//
//   pub(crate) fn replace_appearance(
//     &mut self,
//     mut new_appearance: String,
//     new_anchor_replacement: Option<char>,
//   ) -> Result<(), ModelError> {
//     let anchor_replacement_character = match new_anchor_replacement {
//       Some(anchor_replacement) => anchor_replacement,
//       None => self.anchor_replacement_character,
//     };
//     valid_rectangle_check(&new_appearance)?;
//     if !Rectangle::string_is_valid_rectangle(&new_appearance) {
//       return Err(ModelError::NonRectangularShape);
//     }
//
//
//     let new_anchor_character_index =
//       Self::get_anchor_index(&new_appearance, self.anchor_character)?;
//
//     Self::fix_shape(
//       &mut new_appearance,
//       self.anchor_character,
//       anchor_replacement_character,
//     );
//
//     self.shape = new_appearance.to_owned();
//     self.anchor_character_index = new_anchor_character_index;
//
//     Ok(())
//   }
//
//   /// Returns the index of the anchor character in the sprite's appearance.
//   pub fn get_anchor_character_index(&self) -> usize {
//     self.anchor_character_index
//   }
//
//   /// Returns a reference to the skin's appearance
//   pub fn get_shape(&self) -> &str {
//     &self.shape
//   }
//
//   /// Returns the character labeled as air in the sprite's appearance.
//   pub fn air_character(&self) -> char {
//     self.air_character
//   }
//
//   fn fix_shape(shape: &mut String, anchor_character: char, anchor_replacement_character: char) {
//     *shape = shape.replace(
//       &anchor_character.to_string(),
//       &anchor_replacement_character.to_string(),
//     )
//   }
//
//   fn get_anchor_index(shape: &str, anchor_character: char) -> Result<usize, ModelError> {
//     shape
//       .replace('\n', "")
//       .chars()
//       .position(|pixel| pixel == anchor_character)
//       .ok_or(ModelError::NoAnchor)
//   }
// }
//
// #[cfg(test)]
// mod tests {
//   use super::*;
//
//   #[cfg(test)]
//   mod get_anchor_index_logic {
//     use super::*;
//
//     #[test]
//     fn expected_data() {
//       let shape = "aaaaa\naaxaa";
//       let anchor = 'x';
//
//       let expected_index = Ok(7);
//
//       let anchor_index = Sprite::get_anchor_index(shape, anchor);
//
//       assert_eq!(anchor_index, expected_index);
//     }
//
//     #[test]
//     fn no_anchor() {
//       let shape = "aaaaa\naaaaa";
//       let anchor = 'x';
//
//       let expected_result = Err(ModelError::NoAnchor);
//
//       let result = Sprite::get_anchor_index(shape, anchor);
//
//       assert_eq!(result, expected_result);
//     }
//   }
// }
