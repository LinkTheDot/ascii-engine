use ascii_engine::screen::screen_data::*;
use ascii_engine::CONFIG;

#[test]
fn display_logic() {
  let screen = ScreenData::default();
  // adding the height - 1 is accounting for new lines
  let expected_pixel_count = (CONFIG.grid_width * CONFIG.grid_height) + CONFIG.grid_height - 1;
  let display = screen.display().unwrap();

  assert_eq!(display.len(), expected_pixel_count as usize);
}
