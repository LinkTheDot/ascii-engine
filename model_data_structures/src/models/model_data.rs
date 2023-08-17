// use crate::models::{animation::*, hitboxes::*, sprites::*, strata::*};
// use crate::screen::InternalModels;
// use engine_math::{coordinates::*, hasher};
//
// #[derive(Debug)]
// struct InternalModelData {
//   unique_hash: u64,
//   assigned_name: String,
//   /// The internal coordinates of the sprite's anchor.
//   ///
//   /// Treated as if the appearance of the sprite was a grid, and the anchor was a point on that grid.
//   sprite_internal_anchor_coordinates: (isize, isize),
//   /// counts new lines
//   position_in_frame: usize,
//   strata: Strata,
//   sprite: Sprite,
//   hitbox: Hitbox,
//   /// Exists only when models are placed on the screen
//   existing_models: Option<Arc<RwLock<InternalModels>>>,
//   /// This is created when parsing a model.
//   ///
//   /// None if there was no `.animate` file in the same path of the model, or there was no alternative path given.
//   animation_data: Option<Arc<Mutex<ModelAnimationData>>>,
// }
//
// impl InternalModelData {
//   /// # Errors
//   ///
//   /// - Returns an error when no anchor was found on the shape of the hitbox.
//   /// - Returns an error if multiple anchors were found on the shape of the hitbox.
//   /// - Returns an error when an impossible strata is passed in.
//   fn new(
//     model_position: Coordinates,
//     sprite: Sprite,
//     hitbox_data: HitboxCreationData,
//     strata: Strata,
//     assigned_name: String,
//   ) -> Result<Self, ModelError> {
//     let unique_hash = hasher::get_unique_hash();
//     let position_data = get_position_data(
//       model_position.coordinates_to_index(CONFIG.grid_width as usize + 1),
//       &sprite,
//     );
//     let hitbox = Hitbox::from(hitbox_data, position_data.1)?;
//
//     if !strata.correct_range() {
//       return Err(ModelError::IncorrectStrataRange(strata));
//     }
//
//     Ok(Self {
//       unique_hash,
//       assigned_name,
//       sprite_internal_anchor_coordinates: position_data.1,
//       strata,
//       sprite,
//       position_in_frame: position_data.0,
//       hitbox,
//       existing_models: None,
//       animation_data: None,
//     })
//   }
// }
