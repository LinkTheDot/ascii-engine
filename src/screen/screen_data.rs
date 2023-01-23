use crate::errors::*;
use crate::general_data::file_logger;
use crate::objects::object_data::*;
use crate::screen::objects::*;
use crate::CONFIG;
use log::debug;
use screen_printer::printer::*;
use std::error::Error;
use std::sync::MutexGuard;
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
  pub fn display(&self) -> Result<String, ScreenError> {
    let mut frame = Self::create_blank_frame();

    for strata_number in 0..101 {
      if let Some(objects) = self.object_data.get(&Strata(strata_number)) {
        for (_, object) in objects.iter() {
          let object_guard = match object.lock() {
            Ok(guard) => guard,
            Err(_) => return Err(ScreenError::ObjectError(ObjectError::FailedToGetLock)),
          };

          Self::apply_rows_in_frame(object_guard, &mut frame)?;
        }
      }
    }

    Ok(frame)
  }

  /// Prints the screen as it currently is.
  pub fn print_screen(&mut self) -> Result<(), ScreenError> {
    if self.first_print {
      println!("{}", "\n".repeat(CONFIG.grid_height as usize + 10));

      self.first_print = false;
    }

    let screen = self.display()?;

    debug!("current_screen: \n{screen}");
    // debug!("printer:\n{:?}", self.printer);

    // match self.printer.dynamic_print(screen) {
    //   Ok(_) => Ok(()),
    //   Err(error) => Err(ScreenError::PrintingError(error)),
    // }
    println!("{GRID_SPACER}");
    println!("{screen}");

    Ok(())
  }

  /// Prints whitespace over the screen.
  pub fn clear_screen(&mut self) -> Result<(), ScreenError> {
    match self.printer.clear_grid() {
      Ok(_) => Ok(()),
      Err(error) => Err(ScreenError::PrintingError(error)),
    }
  }

  /// Prints text at the top of the screen.
  pub fn print_text<T>(&mut self, message: T)
  where
    T: std::fmt::Display + std::ops::Deref,
  {
    print!("\x1B[0;0H");
    println!("{message}");
  }

  /// Waits for the input amount of ticks.
  /// The time between ticks is determined by the given value
  /// in the config file.
  pub fn wait_for_x_ticks(&mut self, x: u32) {
    let _ = self.screen_clock.wait_for_x_ticks(x);
  }

  pub fn add_object<O: Object>(&mut self, object: &O) -> Result<(), ObjectError> {
    self.object_data.insert(object.get_unique_hash(), object)
  }

  fn apply_rows_in_frame(
    object: MutexGuard<ObjectData>,
    current_frame: &mut String,
  ) -> Result<(), ScreenError> {
    let mut object_position = *object.top_left();
    debug!("object_position: {:?}", object_position);
    debug!("object_data:\n{:?}", object);
    object_position += object_position / CONFIG.grid_height as usize;
    let (object_width, object_height) = object.get_sprite_dimensions();

    // This is an issue, I don't want to print air at all
    // Might not be able to replace rows at a time, rather replace each
    // character at a time
    let object_shape = object
      .get_sprite()
      .replace(object.get_air_char(), &CONFIG.empty_pixel);
    let object_rows = object_shape.split('\n');

    out_of_bounds_check(object_position, object_width, object_height)?;

    for (row_number, row) in object_rows.enumerate() {
      debug!("row number: {}", row_number);
      let object_position = object_position + ((CONFIG.grid_width as usize + 1) * row_number);

      debug!("{:?}", object_position..(object_position + object_width));
      let (x, y) = (
        object_position % CONFIG.grid_width as usize,
        object_position / CONFIG.grid_width as usize,
      );
      debug!("(x, y): {:?}", (x, y));
      debug!("row shape: {}", row);

      current_frame.replace_range(object_position..(object_position + object_width), row);
    }

    Ok(())
  }

  fn create_blank_frame() -> String {
    let pixel_row = CONFIG.empty_pixel.repeat(CONFIG.grid_width as usize) + "\n";

    pixel_row
      .repeat(CONFIG.grid_height as usize)
      .trim()
      .to_string()
  }
}

fn out_of_bounds_check(
  object_position: usize,
  object_width: usize,
  object_height: usize,
) -> Result<(), ScreenError> {
  if object_width + (object_position % CONFIG.grid_width as usize) >= CONFIG.grid_width as usize {
    return Err(ScreenError::ObjectError(ObjectError::OutOfBounds(
      Direction::Right,
    )));
  } else if object_height + (object_position / CONFIG.grid_width as usize)
    >= CONFIG.grid_height as usize
  {
    return Err(ScreenError::ObjectError(ObjectError::OutOfBounds(
      Direction::Down,
    )));
  }

  Ok(())
}
