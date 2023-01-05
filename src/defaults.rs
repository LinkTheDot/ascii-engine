use crate::screen::objects::Objects;
use crate::screen::screen_data::ScreenData;

impl Default for ScreenData {
  fn default() -> Self {
    Self::new()
      .unwrap_or_else(|error| panic!("An error has occured while grabbing ScreenData: '{error}'"))
  }
}

impl Default for Objects {
  fn default() -> Self {
    Self::new()
  }
}
