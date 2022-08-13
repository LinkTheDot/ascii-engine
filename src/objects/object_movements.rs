#[derive(PartialEq)]
pub enum ObjectMovements {
  Up,
  Down,
  Left,
  Right,
}

pub trait ObjectMovement {
  fn is_horizontal(&self) -> bool;
  fn is_vertical(&self) -> bool;

  fn moves_in_negative_direction(&self) -> bool;
  fn moves_in_positive_direction(&self) -> bool;
}

impl ObjectMovement for ObjectMovements {
  fn is_horizontal(&self) -> bool {
    self == &ObjectMovements::Right || self == &ObjectMovements::Left
  }

  fn is_vertical(&self) -> bool {
    self == &ObjectMovements::Up || self == &ObjectMovements::Down
  }

  fn moves_in_negative_direction(&self) -> bool {
    self == &ObjectMovements::Up || self == &ObjectMovements::Left
  }

  fn moves_in_positive_direction(&self) -> bool {
    self == &ObjectMovements::Down || self == &ObjectMovements::Right
  }
}
