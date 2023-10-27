use crate::models::errors::ModelError;
use std::ffi::OsString;
use thiserror::Error;

/// Contains data for when frame(s) in an animation had invalid sprites, or if the resting frame was invalid.
///
/// The list of invalid frames and their errors is contained.
#[derive(Debug, PartialEq, Eq)]
pub struct AnimationValidityErrorData {
  /// The name of the invalid animation.
  pub animation_name: String,
  /// Contains the errors for if the resting frame was invalid.
  pub resting_appearance_errors: Option<Vec<ModelError>>,
  /// Contains which frame(s) contain an invalid Sprite.
  pub invalid_frame_errors: Vec<(usize, Vec<ModelError>)>,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum AnimationError {
  /// This is a wrapper for the AnimationParserError.
  #[error("Deprecated {:?}", .0)]
  AnimationParserError(#[from] AnimationParserError),

  /// This error happens when trying to request an animation that doesn't exist in the animation list.
  #[error(
    "Attempted to run an animation that doesn't exist. animation_name: {:?}",
    invalid_animation_name
  )]
  AnimationDoesntExist { invalid_animation_name: String },

  /// An instance of [`ModelAnimationData`](crate::models::animation::ModelAnimationData) contained animations with frames containing invalid Sprites.
  #[error("Invalid sprites found in a model's animation data: {:?}", .0)]
  AnimationValidityCheckFailed(Vec<AnimationValidityErrorData>),
}

/// Since almost no error is returned to the user from the animation parser, most errors here will only ever
/// be logged during parsing.
/// This means that it is up to the animation parser to log both the file and error if anything goes wrong,
/// and not up to the error to hold a copy of the file's name.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum AnimationParserError {
  /// This error is returned when the animation file parser failed to get a handle on an animation file in a
  /// model's directory.
  CouldntGetAnimationPath(OsString),

  /// This error is returned when the animation file parser attempts to parse an empty animation file.
  AnimationFileIsEmpty,

  /// This error is returned when an error was returned after attempting to read the contents of an
  /// animation file. This could've been caused by something like invalid UTF-8 being contained in the file.
  CouldntReadAnimationFile,

  /// Invalid syntax was found with the line it was on being contained in the error.
  InvalidSyntax(usize),

  /// This error is returned when the contents of a variable were incorrect.
  /// The line in which this happened is contained in the error.
  InvalidLineContents(usize),

  /// This error is returned when a duplicate variable was found during animation file parsing.
  /// The line in which this happened is contained in the error.
  DuplicateVariable(usize),

  /// This error is returned when an animation hasn't defined how many times it should run.
  MissingLoopCount,

  /// This error is returned when an animation has a frame defined with a duration of 0.
  /// The line this happens is contained in the error.
  FrameDurationOfZero(usize),

  /// This error is returned when an animation has a frame with an invalid shape.
  /// This means the animation's frame is NOT a rectangle.
  InvalidFrameDimensions(usize),

  /// This error is returned when the first animation has defined it's duration two times.
  FrameDurationDefinedTwice(usize),

  /// This error is returned when attempting to initialize a frame that has no appearance.
  FrameHasNoAppearance,

  /// This error is returned when attempting to parse an animation directory for a model, but the path
  /// defined in the model's file doesn't exist.
  AnimationDirectoryDoesntExist(OsString),

  /// This error is returned when attempting to parse an animation directory for a model, but the path
  /// defined in the model's file leads to a file instead of a directory containing animation files.
  AnimationDirectoryIsFile(OsString),
}

impl std::fmt::Display for AnimationParserError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    write!(f, "{:?}", self)
  }
}
