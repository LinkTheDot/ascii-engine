use crate::models::model_data::Strata;
use std::ffi::OsString;

#[derive(Debug, PartialEq, Eq)]
/// This is the list of possible errors that could occurr while handling models.
///
/// Includes a wrapper for [`ModelCreationError`](crate::models::errors::ModelCreationError).
pub enum ModelError {
  /// While creating the model, no anchor point was found.
  NoAnchor,

  /// While creating the model, no every row
  /// contained the same amount of characters.
  NonRectangularShape,

  /// This error is returned when a strata that wasn't 0-100 was passed in.
  IncorrectStrataRange(Strata),

  /// The model has attempted to move out of bounds.
  OutOfBounds(Direction),

  /// A thread holding a copy of a model panicked when trying to obtain the lock of said model.
  FailedToGetLock,

  /// When a model that already exists is attempted to be inserted into the screen.
  ModelAlreadyExists,

  /// When internal model data was attempted to be changed with a model hash that doesn't exist.
  ModelDoesntExist,

  /// There were multiple anchors found in the object's appearance and or hitbox.
  ///
  /// Returns the list of indexes the anchor was found in.
  MultipleAnchorsFound(Vec<usize>),

  /// [`ModelData::from_file()`](crate::models::ModelData::from_file) was called with a path that has the wrong extension.
  NonModelFile,

  /// When something went wrong but it wasn't enough to warrent it's own type.
  ///
  /// Contains a description of what went wrong.
  Other(String),

  /// A wrapper for [`ModelCreationError`](crate::models::errors::ModelCreationError).
  ModelCreationError(ModelCreationError),
}

/// This is the list of possible errors that could happen when parsing a model file.
#[derive(Debug, PartialEq, Eq)]
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

  /// Failed to find the model file with the given path.
  ///
  /// Contains the path that was passed in.
  ModelFileDoesntExist(Option<OsString>),

  /// The model file exists, but has no content inside of it.
  ModelFileIsEmpty,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Direction {
  Up,
  Left,
  Right,
  Down,
}
