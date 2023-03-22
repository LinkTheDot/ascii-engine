use crate::models::model_data::Strata;

#[derive(Debug, PartialEq, Eq)]
/// This is the list of possible errors that could occurr
/// while handling models.
pub enum ModelError {
  /// While creating the model, no center point
  /// was found.
  NoCenter,

  /// While creating the model, no every row
  /// contained the same amount of characters.
  NonRectangularShape,

  /// While building out the hitbox, the string was
  /// found to be empty.
  // possibly just make this mean there's no hitbox
  // period
  EmptyHitboxString,

  /// This error is returned when a strata that wasn't 0-100 was passed in.
  IncorrectStrataRange(Strata),

  /// The model has attempted to move out of bounds.
  OutOfBounds(Direction),

  /// A thread holding a copy of an model panicked when trying to obtain the lock of said model.
  FailedToGetLock,

  /// When an model that already exists is attempted to be inserted into the screen.
  ModelAlreadyExists,

  /// When internal model data was attempted to be changed with an model hash that doesn't exist.
  ModelDoesntExist,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Direction {
  Up,
  Left,
  Right,
  Down,
}
