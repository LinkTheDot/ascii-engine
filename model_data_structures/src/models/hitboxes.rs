use crate::errors::*;
use engine_math::rectangle::*;

/// The required data to create a hitbox.
///
/// Takes the shape of the hitbox and the anchor.
///
/// The shape must be a rectangular shape, nothing else will be accepted.
///
/// # Example
/// ```no_run,bash,ignore
/// xxxxx
/// xxaxx
/// xxxxx
/// ```
///
/// The anchor will be the relative placement of the hitbox to the appearance of a model.
/// When creating a model, both the appearance and hitbox are required to have anchors.
///
/// When placed in the world, a hitbox will be placed on it's anchor, and the hitbox's anchor
/// will be placed over that.
#[derive(Debug)]
pub struct HitboxCreationData {
  dimensions: Rectangle,
  anchor_index: usize,
}

impl HitboxCreationData {
  /// Creates a new instance of HitboxCreationData.
  pub fn new(shape: Rectangle, anchor_index: usize) -> Self {
    Self {
      dimensions: shape,
      anchor_index,
    }
  }

  pub fn assign_anchor_index(&mut self, new_index: usize) -> Result<(), ModelError> {
    if !Rectangle::index_is_valid(&self.dimensions, new_index) {
      return Err(ModelError::IndexLargerThanHitboxArea);
    }

    self.anchor_index = new_index;

    Ok(())
  }

  pub fn assign_dimensions(&mut self, new_dimensions: Rectangle) -> Result<(), ModelError> {
    if !Rectangle::index_is_valid(&new_dimensions, self.anchor_index) {
      return Err(ModelError::IndexLargerThanHitboxArea);
    }

    self.dimensions = new_dimensions;

    Ok(())
  }

  /// Converts a [`HitboxCreationData`](HitboxCreationData) into a [`Hitbox`](Hitbox).
  ///
  /// NOTE
  /// "anchor_skin_coordinates" is the internal coordinates of the anchor within the model's current appearance.
  ///
  ///
  /// If the skin string is empty, returns an [`empty hitbox`](Hitbox::create_empty).
  ///
  /// # Errors
  ///
  /// - Returns an error when no anchor was found on the shape of the hitbox.
  /// - Returns an error if multiple anchors were found on the shape of the hitbox.
  fn get_hitbox(self, anchor_skin_coordinates: (isize, isize)) -> Result<Hitbox, ModelError> {
    if self.dimensions.area() == 0 {
      return Ok(Hitbox::create_empty());
    }

    let skin_top_left_to_hitbox_top_left =
      HitboxCreationData::calculate_skin_top_left_to_hitbox_top_left(
        anchor_skin_coordinates,
        self.anchor_index as f32,
        self.dimensions.x as f32,
      );

    Ok(Hitbox {
      skin_top_left_to_hitbox_top_left,
      hitbox_anchor_index: self.anchor_index,
      dimensions: self.dimensions,
      empty_hitbox: false,
    })
  }

  /// This returns the relative position of the skin's top left to the hitbox's top left
  ///
  /// Takes the position of the skin's anchor character interally.
  ///
  /// This method also takes the index of where the anchor is in the hitbox string. Does not count newlines.
  ///
  /// # Example
  ///
  /// Say you have a skin with 'a' as the anchor, that looks like this:
  /// ```no_run,bash,ignore
  /// xxx
  /// xax
  /// xxx
  /// ```
  /// In this case, the first argument would be (1, 1).
  /// This is because the anchor character is within position (1, 1) of the ``model's skin``.
  ///
  /// Now say your hitbox looks the exact same as the skin.
  /// The other arguments would be 4 and 3.
  ///
  /// With this data, this method would return (0, 0).
  ///
  /// ```ignore
  /// use ascii_engine::models::hitboxes::HitboxCreationData;                        
  ///
  /// let skin_relative_anchor: (isize, isize) = (1, 1);
  /// let hitbox_anchor_index: f32 = 4.0;
  /// let hitbox_width: f32 = 3.0;
  ///
  /// let skin_to_hitbox_anchor =
  ///   HitboxCreationData::calculate_skin_top_left_to_hitbox_top_left(
  ///     skin_relative_anchor,
  ///     hitbox_anchor_index,
  ///     hitbox_width
  ///   );
  ///
  /// assert_eq!(skin_to_hitbox_anchor, (0, 0));
  /// ```
  pub(crate) fn calculate_skin_top_left_to_hitbox_top_left(
    skin_anchor_to_top_left: (isize, isize),
    hitbox_anchor_index: f32,
    hitbox_width: f32,
  ) -> (isize, isize) {
    let hitbox_anchor_to_top_left_x = (hitbox_anchor_index % hitbox_width).round() as isize;
    let hitbox_anchor_to_top_left_y = (hitbox_anchor_index / hitbox_width).round() as isize;

    (
      hitbox_anchor_to_top_left_x - skin_anchor_to_top_left.0,
      hitbox_anchor_to_top_left_y - skin_anchor_to_top_left.1,
    )
  }
}

/// The hitbox will be how objects know the space they take up in the world.
///
/// You will not need to manually create a hitbox, rather, you will add a field called "Hitbox_Dimensions"
/// to your model file.
///
/// # Example
///
/// The "a" character represents the assigned "anchor_character" under the "Skin" Header.
/// ```no_run,bash,ignore
/// * other data above *
/// -=--=-
/// HitboxDimensions
/// xxxxx
/// xxaxx
/// xxxxx
/// ```
///
/// Refer to [`ModelData`](crate::models::model_data::ModelData) for more information on model creation.
///
/// # Manual Creation
///
/// If for some reason you still want to manually create a hitbox through code (which is not recommended and you should make your own model file).
///
/// First you much create [`HitboxCreationData`](HitboxCreationData).
/// From there, you can create a hitbox with that and the relative anchor to the skin using the [`Hitbox::from()`](Hitbox::from) method.
#[derive(Debug, Eq, PartialEq)]
pub struct Hitbox {
  // TODO: Is it really the hitbox's job to know this?
  skin_top_left_to_hitbox_top_left: (isize, isize),
  hitbox_anchor_index: usize,
  dimensions: Rectangle,
  empty_hitbox: bool,
}

impl Hitbox {
  /// Creates a new hitbox from the passed in data and anchor to the skin.
  ///
  /// NOTE
  /// "skin_anchor_coordinates" is the internal coordinates of the anchor within the model's current appearance.
  ///
  /// That would mean if you had a skin like such:
  /// ```no_run,bash,ignore
  /// xxx
  /// xax
  /// xxx
  /// ```
  /// you would pass in (1, 1).
  ///
  /// # Errors
  ///
  /// - Returns an error when no anchor was found on the shape of the hitbox.
  /// - Returns an error if multiple anchors were found on the shape of the hitbox.
  pub fn from(
    hitbox_data: HitboxCreationData,
    skin_anchor_coordinates: (isize, isize),
  ) -> Result<Self, ModelError> {
    hitbox_data.get_hitbox(skin_anchor_coordinates)
  }

  /// Returns an empty hitbox.
  ///
  /// An empty hitbox will have the 'empty_hitbox' field labeled as true.
  /// This will stop any checks from being run on this hitbox instance.
  ///
  /// This means an object with an "empty hitbox" will never interact with the world.
  fn create_empty() -> Self {
    Self {
      skin_top_left_to_hitbox_top_left: (0, 0),
      hitbox_anchor_index: 0,
      dimensions: Rectangle::default(),
      empty_hitbox: true,
    }
  }

  pub fn assign_anchor_index(&mut self, new_index: usize) -> Result<(), ModelError> {
    if !Rectangle::index_is_valid(&self.dimensions, new_index) {
      return Err(ModelError::IndexLargerThanHitboxArea);
    }

    self.hitbox_anchor_index = new_index;

    Ok(())
  }

  pub fn assign_dimensions(&mut self, new_dimensions: Rectangle) -> Result<(), ModelError> {
    if !Rectangle::index_is_valid(&new_dimensions, self.hitbox_anchor_index) {
      return Err(ModelError::IndexLargerThanHitboxArea);
    }

    self.dimensions = new_dimensions;

    Ok(())
  }
}