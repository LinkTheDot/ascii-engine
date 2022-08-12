use interactable_screen::screen::{run_screen_1::*, screen_data::ScreenData};

// - goals -
//
// be able to move an object that's been placed and have the screen
// react to that
// >
// have objects know when they move over each other

fn main() {
  let screen_data = ScreenData::new()
    .unwrap_or_else(|error| panic!("An error has occured while getting ScreenData: '{error}'"));

  if let Err(error) = run_test_screen(screen_data) {
    eprintln!("An error has occured while running the screen: '{error}'");
  }
}
