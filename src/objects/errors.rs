use crate::objects::object_data::Strata;

#[derive(Debug, PartialEq, Eq)]
/// This is the list of possible errors that could occurr
/// while handling objects.
pub enum ObjectError {
  /// While creating the object, no center point
  /// was found.
  NoCenter,

  /// While creating the object, no every row
  /// contained the same amount of characters.
  NonRectangularShape,

  /// While building out the hitbox, the string was
  /// found to be empty.
  // possibly just make this mean there's no hitbox
  // period
  EmptyHitboxString,

  /// This error is returned when a strata that wasn't 0-100 was passed in.
  IncorrectStrataRange(Strata),

  /// The object has attempted to move out of bounds.
  OutOfBounds(Direction),

  /// A thread holding a copy of an object panicked when trying to obtain the lock of said object.
  FailedToGetLock,

  /// When an object that already exists is attempted to be inserted into the screen.
  ObjectAlreadyExists,

  /// When internal object data was attempted to be changed with an object hash that doesn't exist.
  ObjectDoesntExist,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Direction {
  Up,
  Left,
  Right,
  Down,
}
