use crate::objects::object_data::*;
use crate::screen::objects::Objects;
use crate::screen::screen_data::ScreenData;

impl<'a, O> Default for ScreenData<'a, O>
where
  O: Object,
{
  fn default() -> Self {
    Self::new()
      .unwrap_or_else(|error| panic!("An error has occured while grabbing ScreenData: '{error}'"))
  }
}

impl<'a, O> Default for Objects<'a, O>
where
  O: Object,
{
  fn default() -> Self {
    Self::new()
  }
}
