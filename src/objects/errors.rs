#[derive(Debug, PartialEq, Eq)]
pub enum ObjectError {
  NoCenter,
  NonRectangularShape,
  EmptyHitboxString,
}
