use interactable_screen::screen::screen_data::*;

#[test]
fn generate_pixel_grid_logic() {
  let pixel_grid = generate_pixel_grid();
  let expected_grid_length = GRID_WIDTH * GRID_HEIGHT;

  assert_eq!(pixel_grid.len(), expected_grid_length);
}

#[test]
fn display_works() {
  let screen = ScreenData::new()
    .unwrap_or_else(|error| panic!("An error has occured while grabbing ScreenData: '{error}'"));
  let expected_screen_size = GRID_WIDTH * GRID_HEIGHT + GRID_HEIGHT;
  let display = screen.display();

  println!("{}", &display);

  assert_eq!(display.len(), expected_screen_size);
}
