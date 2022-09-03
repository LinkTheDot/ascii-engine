use interactable_screen::screen::{
  screen_data::ScreenData, screen_runs::test_screen::run_test_screen,
};

fn main() {
  let screen_data = ScreenData::new()
    .unwrap_or_else(|error| panic!("An error has occured while getting ScreenData: '{error}'"));

  if let Err(error) = run_test_screen(screen_data) {
    eprintln!("An error has occured while running the screen: '{error}'");
  }
}
