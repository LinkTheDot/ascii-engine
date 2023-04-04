use crate::errors::*;
use crate::general_data::file_logger;
use crate::models::model_data::*;
use crate::screen::models::*;
use crate::CONFIG;
use guard::guard;
use log::error;
use screen_printer::printer::*;
use std::sync::{Arc, RwLock};
use thread_clock::Clock;

#[allow(unused)]
use log::debug;

/// ScreenData is where all the internal information required to create frames is held.
///
/// # Creation
///
/// ```
///  use ascii_engine::prelude::*;
///
///  let screen_data = ScreenData::new();
/// ```
///
/// # Usage
///
/// ```ignore
///  use ascii_engine::prelude::*;
///
///  let mut screen_data = ScreenData::new();
///  screen_data.start_printer().unwrap();
///
///  // Add models to be printed to the screen.
///
///  if let Err(error) = screen_data.print_screen() {
///    log::error!("An error has occurred while printing the screen: {error:?}");
///  }
/// ```
///
/// To create your own models refer to [`ModelData`](crate::models::model_data::ModelData).
/// For adding them to the screen look to [add_model()](crate::screen::screen_data::ScreenData::add_model()).
pub struct ScreenData {
  screen_clock: Clock,
  model_data: Arc<RwLock<InternalModels>>,
  printer: Printer,
  first_print: bool,
  printer_started: bool,

  /// Hides the cursor as long as this lives
  _cursor_hider: termion::cursor::HideCursor<std::io::Stdout>,
}

impl ScreenData {
  /// Creates the screen.
  ///
  /// # Creation
  ///
  /// ```
  ///  use ascii_engine::prelude::*;
  ///
  ///  let screen_data = ScreenData::new();
  /// ```
  ///
  /// # Usage
  ///
  /// ```ignore
  ///  use ascii_engine::prelude::*;
  ///
  ///  let mut screen_data = ScreenData::new();
  ///  screen_data.start_printer().unwrap();
  ///
  ///  // Add models to be printed to the screen.
  ///
  ///  if let Err(error) = screen_data.print_screen() {
  ///    log::error!("An error has occurred while printing the screen: {error:?}");
  ///  }
  /// ```
  ///
  /// To create your own models refer to [`ModelData`](crate::models::model_data::ModelData).
  /// For adding them to the screen look to [add_model()](crate::screen::screen_data::ScreenData::add_model()).
  pub fn new() -> ScreenData {
    // The handle for the file logger, isn't needed right now
    let _ = file_logger::setup_file_logger();
    let cursor_hider = termion::cursor::HideCursor::from(std::io::stdout());
    let mut screen_clock = Clock::custom(CONFIG.tick_duration).unwrap_or_else(|error| {
      panic!("An error has occurred while spawning a clock thread: '{error}'")
    });
    let model_data = Arc::new(RwLock::new(InternalModels::new()));

    screen_clock.start();

    ScreenData {
      screen_clock,
      model_data,
      printer: Printer::new(CONFIG.grid_width as usize, CONFIG.grid_height as usize),
      first_print: true,
      printer_started: false,
      _cursor_hider: cursor_hider,
    }
  }

  /// Creates a new frame of the world as it currently stands.
  ///
  /// This method will build out a frame for the world and return it.
  /// This could be used for when you don't want to use the built in printer and maybe want to
  /// send the data somewhere else other than a terminal.
  ///
  /// If you want to print to a terminal it's best to use the
  /// [`print_screen()`](crate::screen::screen_data::ScreenData::print_screen) method for that.
  ///
  /// Using this does not require you to start the printer as it just returns a frame the printer would've used itself.
  pub fn display(&self) -> String {
    let mut frame = Self::create_blank_frame();

    for strata_number in 0..=100 {
      let model_data = self.model_data.read().unwrap();

      guard!( let Some(strata_keys) = model_data.get_strata_keys(&Strata(strata_number)) else { continue } );

      for model in strata_keys
        .iter()
        .map(|key| self.model_data.read().unwrap().get_model(key))
      {
        guard!( let Some(model) = model else {
          error!("A model in strata {strata_number} that doesn't exist was attempted to be run.");

          continue;
        });

        Self::apply_model_in_frame(model, &mut frame);
      }
    }

    frame
  }

  /// Prints the screen as it currently is.
  ///
  /// This will use a built in printer to efficiently print to the screen.
  /// This prevents any flickers that normally appear in the terminal when printing a lot in a given time frame.
  ///
  /// To use this method you must first call the [`start_printer()`](crate::screen::screen_data::ScreenData::start_printer) method
  /// to activate the printer.
  /// It's recommended to do this right after creating the screen, and before you do anything else.
  ///
  /// # Usage
  ///
  /// ```ignore
  ///  use ascii_engine::prelude::*;
  ///
  ///  let mut screen_data = ScreenData::new();
  ///  screen_data.start_printer().unwrap();
  ///
  ///  // Add models to the screen.
  ///
  ///  if let Err(error) = screen_data.print_screen() {
  ///    log::error!("An error has occurred while printing the screen: {error:?}");
  ///  }
  /// ```
  ///
  /// # Errors
  ///
  /// Returns an error if;
  /// - The printer hasn't been started yet.
  /// - A model is overlapping on the edge of the grid.
  pub fn print_screen(&mut self) -> Result<(), ScreenError> {
    if !self.printer_started {
      return Err(ScreenError::PrinterNotStarted);
    } else if self.first_print {
      self.first_print = false;
    }

    let frame = self.display();

    if let Err(error) = self.printer.dynamic_print(frame) {
      return Err(ScreenError::PrintingError(error));
    }

    Ok(())
  }

  /// Prints whitespace over the screen.
  ///
  /// This can be used to reset the grid if things get desynced from possible bugs.
  ///
  /// # Errors
  ///
  /// - Returns an error if the printer hasn't been started with [`start_printer()`](crate::screen::screen_data::ScreenData::start_printer).
  pub fn clear_screen(&mut self) -> Result<(), ScreenError> {
    if !self.printer_started() {
      return Err(ScreenError::PrinterNotStarted);
    }

    // Any errors this returns shouldn't be able to happen with a started screen.
    self.printer.clear_grid().unwrap();

    Ok(())
  }

  /// This shouldn't be a main way of printing text, but it's here if needed.
  ///
  /// Prints a message at the top of the terminal.
  pub fn print_text<T>(&mut self, message: T)
  where
    T: std::fmt::Display + std::ops::Deref,
  {
    print!("\x1B[1;1H");
    println!("{message}");
  }

  /// Starts the printer allowing you to use the
  /// [`print_screen()`](crate::screen::screen_data::ScreenData::print_screen) method.
  ///
  /// This should be called before starting the user_input thread or any instance of blocking stdin.
  ///
  /// # Usage
  /// ```ignore
  /// use ascii_engine::prelude::*;
  ///
  /// let mut screen_data = ScreenData::new();
  /// screen_data.start_printer().unwrap();
  /// ```
  ///
  /// # Errors
  ///
  /// - An error is returned when stdin is being block upon calling this method.
  pub fn start_printer(&mut self) -> Result<(), ScreenError> {
    if self.printer_started {
      return Err(ScreenError::PrinterAlreadyStarted);
    }

    println!("{}", termion::clear::All);
    println!("{}", "\n".repeat(CONFIG.grid_height as usize + 10));

    if let Err(printing_error) = self.printer.manual_set_origin() {
      return Err(ScreenError::PrintingError(printing_error));
    }

    self.printer_started = true;

    Ok(())
  }

  /// Returns true if the printer has been started or not.
  ///
  /// # Example
  ///
  /// ```
  /// use ascii_engine::prelude::*;
  ///
  /// let screen_data = ScreenData::new();
  ///
  /// assert!(!screen_data.printer_started());
  /// ```
  pub fn printer_started(&self) -> bool {
    self.printer_started
  }

  // The way the clock is handled should be changed.
  // Pass around a clock_receiver instead of using the screen itself to handle that.
  /// Not useful at the moment, soon the be depricated.
  pub fn wait_for_x_ticks(&mut self, x: u32) {
    // Fix the documentation on how this errors
    let _ = self.screen_clock.wait_for_x_ticks(x);
  }

  /// This is how you let the screen know a model exists.
  ///
  /// Refer to [`ModelData`](crate::models::model_data::ModelData) on how to create your own model.
  ///
  /// # Usage
  /// ```
  /// use ascii_engine::prelude::*;
  /// use std::path::Path;
  ///
  /// #[derive(DisplayModel)]
  /// struct MyModel {
  ///   model_data: ModelData,
  /// }
  ///
  /// impl MyModel {
  ///   fn new(world_position: (usize, usize)) -> Self {
  ///     let model_path = Path::new("examples/models/square.model");
  ///
  ///     Self {
  ///       model_data: ModelData::from_file(model_path, world_position).unwrap(),
  ///     }
  ///   }
  /// }
  ///
  /// let mut screen_data = ScreenData::new();
  /// // screen_data.start_printer().unwrap(); crashes when running in tests
  ///
  /// let my_model = MyModel::new((10, 10));
  ///
  /// screen_data.add_model(&my_model).unwrap();
  /// ```
  ///
  /// # Errors
  ///
  /// - An error is returned when attempting to add a model that already exists.
  pub fn add_model<M: DisplayModel>(&mut self, model: &M) -> Result<(), ModelError> {
    let mut model_data = model.get_model_data();

    model_data.assign_model_list(self.model_data.clone());

    self.model_data.write().unwrap().insert(model_data)
  }

  /// Removes the ModelData of the given key.
  ///
  /// Returns The ModelData if it existed, otherwise returns None.
  ///
  /// # Errors (yes there's technically an error)
  ///
  /// Returns None when any existing model somehow has an impossible strata.
  pub fn remove_model(&mut self, key: &u64) -> Option<ModelData> {
    self.model_data.write().unwrap().remove(key)
  }

  /// Places the appearance of the model in the given frame.
  fn apply_model_in_frame(model: ModelData, current_frame: &mut String) {
    let model_frame_position = model.top_left();
    let (model_width, _model_height) = model.get_sprite_dimensions();
    let air_character = model.get_air_char();

    let model_shape = model.get_sprite().replace('\n', "");
    drop(model); // Drops the model lock early since it's no longer needed.
    let model_characters = model_shape.chars();

    // Error returned here to prevent the program from crashing when a model is found to be out of bounds.
    // Uncomment when it's fully implemented.
    // out_of_bounds_check(model_position, model_width, model_height)?;

    for (index, character) in model_characters.enumerate() {
      if character != air_character {
        let current_row_count = index / model_width;

        // (top_left_index + (row_adder + column_adder)) - column_correction
        let character_index = (model_frame_position
          + (((CONFIG.grid_width as usize + 1) * current_row_count) + index))
          - (current_row_count * model_width);

        current_frame.replace_range(
          character_index..(character_index + 1),
          &character.to_string(),
        );
      }
    }
  }

  /// Returns a 2D string of the assigned air character in the config file.
  ///
  /// 2D meaning, rows of characters separated by newlines "creating a second dimension.
  fn create_blank_frame() -> String {
    // This was the fastest way I found to create a large 2-dimensional string of 1 character.
    let pixel_row = CONFIG.empty_pixel.repeat(CONFIG.grid_width as usize) + "\n";

    let mut frame = pixel_row.repeat(CONFIG.grid_height as usize);
    frame.pop(); // remove new line

    frame
  }
}

#[allow(dead_code)]
// This will be remade completely once a "collision check" method is added to models.
fn out_of_bounds_check(
  model_frame_position: usize,
  model_width: usize,
  model_height: usize,
) -> Result<(), ScreenError> {
  // Implement all directions
  // possibly just calculate the x value for this
  // for out of bounds left,
  //   if x == grid_width then say it went out of bounds left

  if model_width + (model_frame_position % (CONFIG.grid_width as usize + 1))
    >= CONFIG.grid_width as usize + 1
  {
    return Err(ScreenError::ModelError(ModelError::OutOfBounds(
      Direction::Right,
    )));
  } else if model_height + (model_frame_position / (CONFIG.grid_width as usize + 1))
    >= CONFIG.grid_height as usize
  {
    return Err(ScreenError::ModelError(ModelError::OutOfBounds(
      Direction::Down,
    )));
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  #![allow(unused)]

  use super::*;
  use crate::general_data::coordinates::*;

  const WORLD_POSITION: (usize, usize) = (10, 10);
  const SHAPE: &str = "xxxxx\nxxaxx\nxxxxx";
  const ANCHOR_CHAR: char = 'a';
  const ANCHOR_REPLACEMENT_CHAR: char = 'x';
  const AIR_CHAR: char = '-';
  const MODEL_NAME: &str = "Test_Model";

  #[test]
  fn change_position_out_of_bounds_right() {
    let frame_position = ((CONFIG.grid_width - 1) as usize, 15);
    let frame_position = frame_position.coordinates_to_index(CONFIG.grid_width as usize + 1);
    let (width, height) = (3, 3);

    let expected_result = Err(ScreenError::ModelError(ModelError::OutOfBounds(
      Direction::Right,
    )));

    let check_result = out_of_bounds_check(frame_position, width, height);

    assert_eq!(check_result, expected_result);
  }

  #[test]
  fn change_position_out_of_bounds_down() {
    let frame_position = (15, CONFIG.grid_width as usize + 1);
    let frame_position = frame_position.coordinates_to_index(CONFIG.grid_width as usize);
    let (width, height) = (3, 3);

    let expected_result = Err(ScreenError::ModelError(ModelError::OutOfBounds(
      Direction::Down,
    )));

    let check_result = out_of_bounds_check(frame_position, width, height);

    assert_eq!(check_result, expected_result);
  }

  #[test]
  fn create_blank_frame() {
    let expected_pixel_count =
      ((CONFIG.grid_width * CONFIG.grid_height) + CONFIG.grid_height - 1) as usize;

    let blank_frame = ScreenData::create_blank_frame();

    assert!(blank_frame.chars().count() == expected_pixel_count);
  }

  #[cfg(test)]
  mod apply_row_in_frame_logic {
    use super::*;

    #[test]
    fn correct_input() {
      let model = TestModel::new();
      let model_data = model.get_model_data();
      let find_character = SHAPE.chars().next().unwrap();
      let top_left_index = model_data.top_left();
      let mut current_frame = ScreenData::create_blank_frame();

      let expected_top_left_character = find_character;
      let expected_left_of_expected_character = CONFIG.empty_pixel.chars().next().unwrap();

      ScreenData::apply_model_in_frame(model_data, &mut current_frame);

      let model_top_left_character_in_frame = current_frame.chars().nth(top_left_index);
      let left_of_index_in_frame = current_frame.chars().nth(top_left_index - 1);

      println!("\n\n{current_frame:?}\n\n");
      println!("top_left: {top_left_index}");

      assert_eq!(
        model_top_left_character_in_frame.unwrap(),
        expected_top_left_character
      );
      assert_eq!(
        left_of_index_in_frame.unwrap(),
        expected_left_of_expected_character
      );
    }
  }

  //
  // -- Data for tests below --
  //

  #[derive(DisplayModel)]
  struct TestModel {
    model_data: ModelData,
  }

  impl TestModel {
    fn new() -> Self {
      let test_model_path = std::path::Path::new("tests/models/test_square.model");
      let model_data = ModelData::from_file(test_model_path, WORLD_POSITION).unwrap();

      Self { model_data }
    }
  }
}
