// Will most likely be replaced with a Z axis on models
/// The Strata will be the priority on the screen.
/// That which has a lower Strata, will be behind those with a higher strata.
///
/// The strata is a range from 0-100, any number outside of that range will
/// not be accepted.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Strata(pub usize);

impl Strata {
  /// Returns true if the given strata is withing 0-100.
  pub fn correct_range(&self) -> bool {
    self.0 <= 100
  }
}
