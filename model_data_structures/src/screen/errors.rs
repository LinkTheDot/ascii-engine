use crate::models::errors::*;

/// The list of errors that could happen when dealing with the screen.
///
/// Includes wrappers for both
/// [`ModelError`](crate::models::errors::ModelError) and
/// [`PrintingError`](screen_printer::printer::PrintingError).
#[derive(Debug, PartialEq, Eq)]
pub enum ScreenError {
  /// A wrapper for [`ModelError`](crate::models::errors::ModelError).
  ModelError(ModelError),

  /// A wrapper for [`PrintingError`](screen_printer::printer::PrintingError).
  PrintingError(screen_printer::printer::PrintingError),

  /// A wrapper for [`AnimationError`](crate::models::errors::AnimationError).
  AnimationError(AnimationError),

  /// Generally a wrapper around other crate's error types.
  Other(String),

  /// Attempted to read a file that did not exist.
  FileDoesNotExist,

  /// There was an error when attempting to serialize the data for a world.
  /// Contains the error that caused this.
  FailedToLoadWorld(String),
}
