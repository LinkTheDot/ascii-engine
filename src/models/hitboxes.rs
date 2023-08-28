// use crate::models::errors::*;
// use log::error;
// use std::cmp::Ordering;
//
// /// The hitbox will be how objects know the space they take up in the world.
// ///
// /// You will not need to manually create a hitbox, rather, you will add a field called "Hitbox_Dimensions"
// /// to your model file.
// ///
// /// # Example
// ///
// /// The "a" character represents the assigned "anchor_character" under the "Skin" Header.
// /// ```no_run,bash,ignore
// /// * other data above *
// /// -=--=-
// /// HitboxDimensions
// /// xxxxx
// /// xxaxx
// /// xxxxx
// /// ```
// ///
// /// Refer to [`ModelData`](crate::models::model_data::ModelData) for more information on model creation.
// ///
// /// # Manual Creation
// ///
// /// If for some reason you still want to manually create a hitbox through code (which is not recommended and you should make your own model file).
// ///
// /// First you much create [`HitboxCreationData`](HitboxCreationData).
// /// From there, you can create a hitbox with that and the relative anchor to the skin using the [`Hitbox::from()`](Hitbox::from) method.
// #[derive(Debug, Eq, PartialEq)]
// pub struct Hitbox {
//   skin_top_left_to_hitbox_top_left: (isize, isize),
//   hitbox_anchor_index: usize,
//   width: isize,
//   height: isize,
//   empty_hitbox: bool,
// }
//
// /// The required data to create a hitbox.
// ///
// /// Takes the shape of the hitbox and the anchor.
// ///
// /// The shape must be a rectangular shape, nothing else will be accepted.
// ///
// /// # Example
// /// ```no_run,bash,ignore
// /// xxxxx
// /// xxaxx
// /// xxxxx
// /// ```
// ///
// /// The anchor will be the relative placement of the hitbox to the appearance of a model.
// /// When creating a model, both the appearance and hitbox are required to have anchors.
// ///
// /// When placed in the world, a hitbox will be placed on it's anchor, and the hitbox's anchor
// /// will be placed over that.
// #[derive(Debug)]
// pub struct HitboxCreationData {
//   pub shape: String,
//   pub anchor_character: char,
// }
//
// impl Hitbox {
//   /// Creates a new hitbox from the passed in data and anchor to the skin.
//   ///
//   /// NOTE
//   /// "skin_anchor_coordinates" is the internal coordinates of the anchor within the model's current appearance.
//   ///
//   /// That would mean if you had a skin like such:
//   /// ```no_run,bash,ignore
//   /// xxx
//   /// xax
//   /// xxx
//   /// ```
//   /// you would pass in (1, 1).
//   ///
//   /// # Errors
//   ///
//   /// - Returns an error when no anchor was found on the shape of the hitbox.
//   /// - Returns an error if multiple anchors were found on the shape of the hitbox.
//   pub fn from(
//     hitbox_data: HitboxCreationData,
//     skin_anchor_coordinates: (isize, isize),
//   ) -> Result<Self, ModelError> {
//     hitbox_data.get_hitbox(skin_anchor_coordinates)
//   }
//
//   /// Returns an empty hitbox.
//   ///
//   /// An empty hitbox will have the 'empty_hitbox' field labeled as true.
//   /// This will stop any checks from being run on this hitbox instance.
//   ///
//   /// This means an object with an "empty hitbox" will never interact with the world.
//   fn create_empty() -> Self {
//     Self {
//       skin_top_left_to_hitbox_top_left: (0, 0),
//       hitbox_anchor_index: 0,
//       width: 0,
//       height: 0,
//       empty_hitbox: true,
//     }
//   }
//
//   /// Returns the (width, height) of the hitbox.
//   pub(crate) fn get_dimensions(&self) -> (isize, isize) {
//     (self.width, self.height)
//   }
//
//   /// Returns true if the hitbox is labeled as empty.
//   pub(crate) fn is_empty(&self) -> bool {
//     self.empty_hitbox
//   }
//
//   #[allow(unused)]
//   pub(crate) fn get_relative_top_left(&self) -> (isize, isize) {
//     self.skin_top_left_to_hitbox_top_left
//   }
//
//   pub(crate) fn recalculate_relative_top_left(&mut self, skin_anchor_coordinates: (isize, isize)) {
//     let new_relative_distance = HitboxCreationData::calculate_skin_top_left_to_hitbox_top_left(
//       skin_anchor_coordinates,
//       self.hitbox_anchor_index as f32,
//       self.width as f32,
//     );
//
//     self.skin_top_left_to_hitbox_top_left = new_relative_distance;
//   }
//
//   pub(crate) fn get_anchor_index(&self) -> usize {
//     self.hitbox_anchor_index
//   }
// }
//
// impl HitboxCreationData {
//   /// Creates a new instance of HitboxCreationData.
//   ///
//   /// This should not be used over model files.
//   /// Refer to [`ModelData`](crate::models::model_data::ModelData) for information of creating a model file.
//   pub fn new(shape: &str, anchor_character: char) -> Self {
//     Self {
//       shape: shape.to_string(),
//       anchor_character,
//     }
//   }
//
//   /// Converts a [`HitboxCreationData`](HitboxCreationData) into a [`Hitbox`](Hitbox).
//   ///
//   /// NOTE
//   /// "anchor_skin_coordinates" is the internal coordinates of the anchor within the model's current appearance.
//   ///
//   ///
//   /// If the skin string is empty, returns an [`empty hitbox`](Hitbox::create_empty).
//   ///
//   /// # Errors
//   ///
//   /// - Returns an error when no anchor was found on the shape of the hitbox.
//   /// - Returns an error if multiple anchors were found on the shape of the hitbox.
//   fn get_hitbox(self, anchor_skin_coordinates: (isize, isize)) -> Result<Hitbox, ModelError> {
//     if self.shape.trim() == "" {
//       return Ok(Hitbox::create_empty());
//     }
//
//     let (hitbox_width, hitbox_height) = valid_rectangle_check(&self.shape)?;
//     let hitbox = &self.shape.split('\n').collect::<String>();
//     let hitbox_anchor_indices: Vec<usize> = hitbox
//       .chars()
//       .enumerate()
//       .filter(|(_, character)| character == &self.anchor_character)
//       .map(|(index, _)| index)
//       .collect();
//
//     let hitbox_anchor_index = match hitbox_anchor_indices.len().cmp(&1) {
//       Ordering::Equal => hitbox_anchor_indices[0],
//       Ordering::Greater => {
//         error!("Multiple anchors were found when attempting to make a hitbox.");
//
//         return Err(ModelError::MultipleAnchorsFound(hitbox_anchor_indices));
//       }
//       Ordering::Less => {
//         error!("No anchors were found when attempting to make a hitbox.");
//
//         return Err(ModelError::NoAnchor);
//       }
//     };
//
//     let skin_top_left_to_hitbox_top_left =
//       HitboxCreationData::calculate_skin_top_left_to_hitbox_top_left(
//         anchor_skin_coordinates,
//         hitbox_anchor_index as f32,
//         hitbox_width as f32,
//       );
//
//     Ok(Hitbox {
//       skin_top_left_to_hitbox_top_left,
//       hitbox_anchor_index,
//       width: hitbox_width as isize,
//       height: hitbox_height as isize,
//       empty_hitbox: false,
//     })
//   }
//
//   /// This returns the relative position of the skin's top left to the hitbox's top left
//   ///
//   /// Takes the position of the skin's anchor character interally.
//   ///
//   /// This method also takes the index of where the anchor is in the hitbox string. Does not count newlines.
//   ///
//   /// # Example
//   ///
//   /// Say you have a skin with 'a' as the anchor, that looks like this:
//   /// ```no_run,bash,ignore
//   /// xxx
//   /// xax
//   /// xxx
//   /// ```
//   /// In this case, the first argument would be (1, 1).
//   /// This is because the anchor character is within position (1, 1) of the ``model's skin``.
//   ///
//   /// Now say your hitbox looks the exact same as the skin.
//   /// The other arguments would be 4 and 3.
//   ///
//   /// With this data, this method would return (0, 0).
//   ///
//   /// ```ignore
//   /// use ascii_engine::models::hitboxes::HitboxCreationData;
//   ///
//   /// let skin_relative_anchor: (isize, isize) = (1, 1);
//   /// let hitbox_anchor_index: f32 = 4.0;
//   /// let hitbox_width: f32 = 3.0;
//   ///
//   /// let skin_to_hitbox_anchor =
//   ///   HitboxCreationData::calculate_skin_top_left_to_hitbox_top_left(
//   ///     skin_relative_anchor,
//   ///     hitbox_anchor_index,
//   ///     hitbox_width
//   ///   );
//   ///
//   /// assert_eq!(skin_to_hitbox_anchor, (0, 0));
//   /// ```
//   pub(crate) fn calculate_skin_top_left_to_hitbox_top_left(
//     skin_anchor_to_top_left: (isize, isize),
//     hitbox_anchor_index: f32,
//     hitbox_width: f32,
//   ) -> (isize, isize) {
//     let hitbox_anchor_to_top_left_x = (hitbox_anchor_index % hitbox_width).round() as isize;
//     let hitbox_anchor_to_top_left_y = (hitbox_anchor_index / hitbox_width).round() as isize;
//
//     (
//       hitbox_anchor_to_top_left_x - skin_anchor_to_top_left.0,
//       hitbox_anchor_to_top_left_y - skin_anchor_to_top_left.1,
//     )
//   }
// }
//
// /// Returns
// /// Result<(width, height)>.
// ///
// /// # Errors
// ///
// /// - An error is returned when the shape isn't a rectangle.
// /// - An error is returned when the shape is empty.
// // Move this and other similar functions that are lying around into their own general_data module.
// //
// // Possibly change this to two different methods
// // valid_rectangle() and get_dimensions()
// // which both will rely on the same logic, just return different things
// pub(crate) fn valid_rectangle_check(rectangle_shape: &str) -> Result<(usize, usize), ModelError> {
//   if rectangle_shape.is_empty() {
//     return Err(ModelError::NonRectangularShape);
//   }
//
//   let rows: Vec<&str> = rectangle_shape.split('\n').collect();
//   let model_width = rows[0].chars().count();
//
//   let rows_have_same_lengths = rows.iter().all(|row| row.chars().count() == model_width);
//
//   if rows_have_same_lengths {
//     Ok((model_width, rows.len()))
//   } else {
//     Err(ModelError::NonRectangularShape)
//   }
// }
//
// #[cfg(test)]
// mod tests {
//   use super::*;
//   // use crate::prelude::*;
//
//   #[cfg(test)]
//   mod valid_rectangle_check_logic {
//     use super::*;
//
//     #[test]
//     fn valid_rectangle() {
//       let dimensions = "xxx\nxxx\nxxx";
//
//       let expected_dimensions = Ok((3, 3));
//
//       let rectangle_dimensions = valid_rectangle_check(dimensions);
//
//       assert_eq!(rectangle_dimensions, expected_dimensions);
//     }
//
//     #[test]
//     fn invalid_rectangle() {
//       let dimensions = "xx\nxxx\nx\nxxxxxx";
//
//       let expected_error = Err(ModelError::NonRectangularShape);
//
//       let returned_data = valid_rectangle_check(dimensions);
//
//       assert_eq!(returned_data, expected_error);
//     }
//   }
//
//   #[test]
//   fn empty_hitbox_logic() {
//     let hitbox_creation_data = HitboxCreationData::new("", 'a');
//     let hitbox = Hitbox::from(hitbox_creation_data, (0, 0)).unwrap();
//
//     assert!(hitbox.is_empty());
//   }
//
//   // #[test]
//   // fn get_hitbox_position_logic() {
//   //   let hitbox_creation_data = HitboxCreationData::new("xxx\nxax", 'a');
//   //   // With a skin of the same shape: "xxx\nxax".
//   //   let hitbox = Hitbox::from(hitbox_creation_data, (1, 1)).unwrap();
//   //   // where the model is at (11, 11).
//   //   let model_frame_position = 10 + (10 * (CONFIG.grid_width as usize + 1));
//   //
//   //   let expected_hitbox_frame_position = (10, 10);
//   //
//   //   let hitbox_frame_position = hitbox.get_hitbox_position(model_frame_position);
//   //
//   //   assert_eq!(hitbox_frame_position, expected_hitbox_frame_position);
//   // }
//
//   #[test]
//   fn get_dimensions_logic() {
//     let hitbox_creation_data = HitboxCreationData::new("xxxxx\nxxaxx\nxxxxx", 'a');
//     let hitbox = Hitbox::from(hitbox_creation_data, (1, 1)).unwrap();
//
//     let expected_dimensions: (isize, isize) = (5, 3);
//
//     let hitbox_dimensions = hitbox.get_dimensions();
//
//     assert_eq!(hitbox_dimensions, expected_dimensions);
//   }
//
//   #[test]
//   fn hitbox_data_no_anchor() {
//     let hitbox_creation_data = HitboxCreationData::new("x", 'a');
//
//     let expected_result = Err(ModelError::NoAnchor);
//
//     let result = Hitbox::from(hitbox_creation_data, (0, 0));
//
//     assert_eq!(result, expected_result);
//   }
//
//   #[test]
//   fn hitbox_data_multiple_anchors() {
//     let hitbox_creation_data = HitboxCreationData::new("xaax", 'a');
//
//     let expected_result = Err(ModelError::MultipleAnchorsFound(vec![1, 2]));
//
//     let result = Hitbox::from(hitbox_creation_data, (0, 0));
//
//     assert_eq!(result, expected_result);
//   }
// }
