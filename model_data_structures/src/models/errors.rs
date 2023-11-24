use crate::models::animation::errors::*;
use crate::models::strata::Strata;
use std::ffi::OsString;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
/// This is the list of possible errors that could occurr while handling models.
///
/// Includes a wrapper for [`ModelCreationError`](crate::models::errors::ModelCreationError).
pub enum ModelError {
  /// There was no anchor found in the appearance of a [`Sprite`](crate::models::model_appearance::sprites::Sprite).
  #[error("A model's sprite was missing an anchor point for world and hitbox placement.")]
  NoAnchor,

  /// A model's appearance was found to be non-rectangular.
  /// (Every row must contain the same amount of characters to be rectangular)
  #[error("A model's sprite was found to be a non-rectangular shape.")]
  NonRectangularShape,

  /// A model's [`Strata`](crate::models::strata::Strata) wasn't within 0-100.
  #[error("A model was found to contain an invalid strata: {:?}", .0)]
  IncorrectStrataRange(Strata),

  /// When a model that already exists is attempted to be inserted into the screen.
  #[error("Attempted to insert an already existing model into the world.")]
  ModelAlreadyExists,

  /// When internal model data was attempted to be changed with a model hash that doesn't exist.
  #[error("Attempted to get a model that doesn't exist.")]
  ModelDoesntExist,

  /// A model contained multiple anchors in its [`Sprite`](crate::models::model_appearance::sprites::Sprite).
  #[error("A model was found to have multiple anchor points in its appearance.")]
  MultipleAnchorsFound(Vec<usize>),

  /// A wrapper for the [`ModelCreationError`](ModelCreationError) error type.
  #[error("Failed to create a model. Reason: {:?}", .0)]
  ModelCreationError(#[from] ModelCreationError),

  /// A wrapper for the [`AnimationError`](AnimationError) error type.
  #[error("An error occurred while animating a model. Reason: {:?}", .0)]
  AnimationError(#[from] AnimationError),

  /// When changing the air or anchor character for a sprite, both matched each other..
  #[error("Attempted to change a sprite's anchor/air character to be the same.")]
  SpriteAnchorMatchesAirCharacter,

  /// Attempted to change the anchor character on a model that already contained
  /// that character in its appearance.
  #[error("Attempted to change a sprite's anchor character, but the character already exists in the appearance.")]
  ModelSpriteContainsNewAnchorCharacter,

  /// Attempted to create/change a hitbox with an index that's larger than it's area.
  #[error("Attempted to change a Hitbox to dimensions too small for its stored anchor index.")]
  IndexLargerThanHitboxArea,

  /// Attempted to place a model out of bounds of the world.
  #[error("Model has been place of of bounds of the screen.")]
  ModelOutOfBounds,

  /// A stored list of errors returned when checking if a sprite has any issues with it's data.
  #[error("A sprite was found to be invalid. Reason(s): {:?}", .0)]
  SpriteValidityChecks(Vec<Self>),

  /// When something went wrong but it wasn't enough to warrent it's own type.
  ///
  /// Contains a description of what went wrong.
  #[error("An error has occurred. Reason: {:?}", .0)]
  Other(String),
}

/// This is the list of possible errors that could happen when parsing a model file.
#[derive(Error, Debug, PartialEq, Eq, Clone)]
// This error won't be implemented for this as model files are going to be replaced in the near future.
pub enum ModelCreationError {
  /// Invalid syntax was found with the line it was on being contained in the error.
  InvalidSyntax(usize),

  /// A string that was suppose to be 1 character was found to be more or less than 1.
  /// The line in which this happened is contained in the error.
  InvalidStringSizeAtLine(usize),

  /// A Model has a strata range that's impossible.
  ///
  /// Returns the given strata range.
  InvalidStrataRange(usize),

  /// When parsing the appearance of the model, it was found to be non-rectangular.
  InvalidSkinShape,

  /// When parsing the hitbox of the model, it was found to be non-rectangular.
  InvalidHitboxShape,

  /// One or more fields of data were missing from the model file.
  ///
  /// Contains a list of everything that was missing.
  MissingData(Vec<String>),

  /// [`ModelData::from_file()`](crate::models::model_data::ModelData::from_file) was called with a path that has the wrong extension.
  NonModelFile,

  /// Failed to find the model file with the given path.
  ///
  /// Contains the path that was passed in.
  ModelFileDoesntExist(Option<OsString>),

  /// The model file exists, but has no content inside of it.
  ModelFileIsEmpty,
}
impl std::fmt::Display for ModelCreationError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    write!(f, "{:?}", self)
  }
}
