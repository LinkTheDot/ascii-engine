use crate::models::strata::Strata;
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

  /// A thread holding a copy of a model panicked when trying to obtain the lock of said model.
  FailedToGetLock,

  /// When a model that already exists is attempted to be inserted into the screen.
  ModelAlreadyExists,

  /// When internal model data was attempted to be changed with a model hash that doesn't exist.
  ModelDoesntExist,

  /// There were multiple anchors found in the object's appearance and or hitbox.
  ///
  /// Returns the list of indices the anchor was found in.
  MultipleAnchorsFound(Vec<usize>),

  /// [`ModelData::from_file()`](crate::models::model_data::ModelData::from_file) was called with a path that has the wrong extension.
  NonModelFile,

  /// When something went wrong but it wasn't enough to warrent it's own type.
  ///
  /// Contains a description of what went wrong.
  Other(String),

  /// This error occurrs when attempting to assign an animation to a model that has already been assigned an animation.
  ModelAlreadyHasAnimationData,

  /// A wrapper for the [`ModelCreationError`](ModelCreationError) error type.
  ModelCreationError(ModelCreationError),

  /// A wrapper for the [`AnimationError`](AnimationError) error type.
  AnimationError(AnimationError),

  /// When changing the air or anchor character for a sprite, both matched each other.
  SpriteAnchorMatchesAirCharacter,

  /// Attempted to change the anchor character on a model that already contained
  /// that character in its appearance.
  ModelSpriteContainsNewAnchorCharacter,

  /// Attempted to create/change a hitbox with an index that's larger than it's area.
  IndexLargerThanHitboxArea,
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
pub enum AnimationError {
  /// This is a wrapper for the AnimationParserError.
  AnimationParserError(AnimationParserError),

  /// This is when a model tried starting it's animation thread when it was already started.
  AnimationAlreadyStarted,

  /// This is returned when a model tries to send an animation request when the animation wasn't started yet.
  AnimationNotStarted,

  /// This is when a model attempted to take an action that requires animation_data, but it doesn't have any.
  ModelHasNoAnimationData,

  /// This is returned when AnimationData was called with a filepath that wasn't an animation file.
  NonAnimationFile,

  /// Used internally to signify when the animation queue has run out of animations
  EmptyQueue,

  /// This error happens when trying to request an animation that doesn't exist in the animation list.
  AnimationDoesntExist,

  /// This error happens when attempting to add an animation of a duplicate name into a model's animation_data.
  AnimationAlreadyExists,

  /// This error happens when attempting to assign a connection to an instance of ModelAnimationData that already has one.
  AnimationDataAlreadyHasConnection,

  /// This error happens when attempting to start a model's animation without ever starting the animation thread.
  ///
  /// To start the animation thread use [`screen_data.start_animation_thread()`](crate::screen::screen_data::ScreenData::start_animation_thread).
  AnimationThreadNotStarted,

  /// This error is returned when attempting to start the animation thread when it has already been started.
  AnimationThreadAlreadyStarted,

  /// This error is returned when attempting to increment the changed frames on a model animator that has no current animation.
  NoExistingAnimation,

  /// This error is returned when attempting to parse an animation directory for a model, but the path
  /// defined in the model's file doesn't exist.
  AnimationDirectoryDoesntExist(OsString),

  /// This error is returned when attempting to parse an animation directory for a model, but the path
  /// defined in the model's file leads to a file instead of a directory containing animation files.
  AnimationDirectoryIsFile(OsString),
}

/// Since almost no error is returned to the user from the animation parser, most errors here will only ever
/// be logged during parsing.
/// This means that it is up to the animation parser to log both the file and error if anything goes wrong,
/// and not up to the error to hold a copy of the file's name.
#[derive(Debug, PartialEq, Eq)]
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
}
