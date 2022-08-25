use interactable_screen::screen::{
  screen_data::ScreenData, screen_runs::test_screen::run_test_screen,
};

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

// known bugs
//  - pixels aren't reassigning display data when overwritten then undone
//  - can't go out of bounds in either positive direction
//    - down crashes
//    - right causes some weird bugs
//  - x can never = 0
//  - when an object reaches the far left side of the screen it can't move
//    in any direction but right
