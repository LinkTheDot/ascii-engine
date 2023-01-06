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
}
