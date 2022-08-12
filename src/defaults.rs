use crate::clock::clock_struct::ScreenClock;
use crate::screen::screen_data::{Pixel, ScreenData};

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

impl Default for ScreenClock {
  fn default() -> Self {
    Self::new()
      .unwrap_or_else(|error| panic!("An error has occured while creating a new clock: '{error}'"))
  }
}
