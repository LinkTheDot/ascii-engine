use crate::general_data::file_logger;
use crate::screen::objects::*;
use crate::CONFIG;
use guard::guard;
use screen_printer::printer::*;
use std::error::Error;
use thread_clock::Clock;

pub const GRID_SPACER: &str = "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n";

/// This is in the context of the update_placed_objects function
/// but could technically be used anywhere
pub enum Actions {
  Add,
  Subtract,
}

/// Contains all of the data for the screen such as
/// The clock
/// The counter for all objects that exist
/// The set of pixels that make up the screen
pub struct ScreenData {
  screen_clock: Clock,
  object_data: Objects,
  printer: Printer,
  first_print: bool,

  /// Hides the cursor as long as this lives
  _cursor_hider: termion::cursor::HideCursor<std::io::Stdout>,
}

impl ScreenData {
  /// Creates a new screen and starts all processes required for the engine.
  /// These include the file logger, clock, and cursor hider.
  pub fn new() -> Result<ScreenData, Box<dyn Error>> {
    // The handle for the file logger, isn't needed right now
    let _ = file_logger::setup_file_logger();
    let cursor_hider = termion::cursor::HideCursor::from(std::io::stdout());
    let mut screen_clock = Clock::custom(CONFIG.tick_duration).unwrap_or_else(|error| {
      panic!("An error has occurred while spawning a clock thread: '{error}'")
    });

    screen_clock.start();

    Ok(ScreenData {
      screen_clock,
      object_data: Objects::new(),
      printer: Printer::new(CONFIG.grid_width as usize, CONFIG.grid_height as usize),
      first_print: true,
      _cursor_hider: cursor_hider,
    })
  }

  /// Returns the screen as a string depending on what each pixel
  /// is assigned
  pub fn display(&self) -> Result<String, PrintingError> {
    // Printer::create_grid_from_full_character_list(&self.screen, GRID_WIDTH, GRID_HEIGHT)
    todo!()
  }

  // return an error when those are added
  /// Prints the screen as it currently is.
  pub fn print_screen(&mut self) {
    if self.first_print {
      println!("{}", "\n".repeat(CONFIG.grid_height as usize + 10));

      self.first_print = false;
    }

    guard!( let Ok(screen) = self.display() else { return; } );

    let _ = self.printer.dynamic_print(screen);
  }

  /// Replaces every pixel with whitespace, does not overwrite assignment.
  pub fn clear_screen(&mut self) -> Result<(), PrintingError> {
    self.printer.clear_grid()?;

    Ok(())
  }

  pub fn print_text<T>(&mut self, message: T)
  where
    T: std::fmt::Display + std::ops::Deref,
  {
    print!("\x1B[0;0H");
    println!("{message}");
  }

  pub fn wait_for_x_ticks(&mut self, x: u32) {
    let _ = self.screen_clock.wait_for_x_ticks(x);
  }
}

// /// Generates a 1-Dimensional grid of Pixels
// pub fn generate_pixel_grid() -> Vec<Pixel> {
//   (0..(GRID_WIDTH * GRID_HEIGHT)) //
//     .fold(Vec::new(), |mut pixel_vec, pixel_index| {
//       pixel_vec.push(Pixel::new(pixel_index));
//
//       pixel_vec
//     })
// }
