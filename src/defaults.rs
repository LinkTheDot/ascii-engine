use crate::screen::{pixel::*, screen_data::ScreenData};

impl Default for Pixel {
  fn default() -> Self {
    Self::new()
  }
}

impl Default for ScreenData {
  fn default() -> Self {
    Self::new()
      .unwrap_or_else(|error| panic!("An error has occured while grabbing ScreenData: '{error}'"))
  }
}
