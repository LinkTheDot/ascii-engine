use crate::errors::*;
use crate::general_data::file_logger;
use crate::models::model_data::*;
use crate::screen::models::*;
use crate::CONFIG;
use guard::guard;
use log::error;
use screen_printer::printer::*;
use std::error::Error;
use std::sync::{Arc, MutexGuard, RwLock};
use thread_clock::Clock;

#[allow(unused)]
use log::debug;

pub const GRID_SPACER: &str = "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n";

/// This is in the context of the update_placed_models function
/// but could technically be used anywhere
pub enum Actions {
  Add,
  Subtract,
}

/// Contains all of the data for the screen such as
/// The clock
/// The counter for all models that exist
/// The set of pixels that make up the screen
pub struct ScreenData {
  screen_clock: Clock,
  model_data: Arc<RwLock<Models>>,
  printer: Printer,
  first_print: bool,
  printer_started: bool,

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
    let model_data = Arc::new(RwLock::new(Models::new()));

    screen_clock.start();

    Ok(ScreenData {
      screen_clock,
      model_data,
      printer: Printer::new(CONFIG.grid_width as usize, CONFIG.grid_height as usize),
      first_print: true,
      printer_started: false,
      _cursor_hider: cursor_hider,
    })
  }

  /// Creates a new frame of the world as it currently stands
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

        let model_guard = model.lock().unwrap();

        Self::apply_model_in_frame(model_guard, &mut frame);
      }
    }

    frame
  }

  /// Prints the screen as it currently is.
  ///
  /// # Errors
  ///
  /// Returns an error if;
  /// - The printer hasn't been started yet.
  /// - An object is overlapping on the edge of the grid.
  pub fn print_screen(&mut self) -> Result<(), ScreenError> {
    if !self.printer_started {
      return Err(ScreenError::PrinterNotStarted);
    } else if self.first_print {
      println!("{}", "\n".repeat(CONFIG.grid_height as usize + 10));

      self.first_print = false;
    }

    if let Err(error) = self.printer.dynamic_print(self.display()) {
      return Err(ScreenError::PrintingError(error));
    }

    Ok(())
  }

  /// Prints whitespace over the screen.
  pub fn clear_screen(&mut self) -> Result<(), ScreenError> {
    if let Err(error) = self.printer.clear_grid() {
      return Err(ScreenError::PrintingError(error));
    }

    Ok(())
  }

  /// Prints text at the top of the screen.
  pub fn print_text<T>(&mut self, message: T)
  where
    T: std::fmt::Display + std::ops::Deref,
  {
    print!("\x1B[0;0H");
    println!("{message}");
  }

  pub fn start_printer(&mut self) -> Result<(), ScreenError> {
    if let Err(printing_error) = self.printer.manual_set_origin() {
      return Err(ScreenError::PrintingError(printing_error));
    }

    self.printer_started = true;

    Ok(())
  }

  pub fn printer_started(&self) -> bool {
    self.printer_started
  }

  /// Waits for the input amount of ticks.
  /// The time between ticks is determined by the given value
  /// in the config file.
  // The way the clock is handled should be changed.
  // Pass around a clock_receiver instead of using the screen itself to handle that.
  pub fn wait_for_x_ticks(&mut self, x: u32) {
    // Fix the documentation on how this errors
    let _ = self.screen_clock.wait_for_x_ticks(x);
  }

  pub fn add_model<O: DisplayModel>(&mut self, model: &mut O) -> Result<(), ModelError> {
    model.assign_model_list(self.model_data.clone());

    self
      .model_data
      .write()
      .unwrap()
      .insert(&model.get_unique_hash(), model)
  }

  /// Places the appearance of the model in the given frame.
  fn apply_model_in_frame(model: MutexGuard<ModelData>, current_frame: &mut String) {
    let model_frame_position = *model.top_left();
    let (model_width, _model_height) = model.get_sprite_dimensions();
    let air_character = model.get_air_char().to_owned();

    let model_shape = model.get_sprite().to_string().replace('\n', "");
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

  fn create_blank_frame() -> String {
    // This was the fastest way I found to create a large 2-dimensional string of 1 character.
    let pixel_row = CONFIG.empty_pixel.repeat(CONFIG.grid_width as usize) + "\n";

    let mut frame = pixel_row.repeat(CONFIG.grid_height as usize);
    frame.pop(); // remove new line

    frame
  }
}

#[allow(dead_code)]
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
  use super::*;
  use crate::general_data::coordinates::*;
  use crate::models::hitboxes::HitboxCreationData;

  const SHAPE: &str = "x-x\nxcx\nx-x";
  const ANCHOR_CHAR: char = 'c';
  const ANCHOR_REPLACEMENT_CHAR: char = '-';
  const AIR_CHAR: char = '-';

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
      let model_data = get_model_data((10, 10));
      let find_character = SHAPE.chars().next().unwrap();
      let top_left_index = *model_data.top_left();
      let model_data = Mutex::new(model_data);
      let mut current_frame = ScreenData::create_blank_frame();

      let expected_top_left_character = find_character;
      let expected_left_of_expected_character = CONFIG.empty_pixel.chars().next().unwrap();

      ScreenData::apply_model_in_frame(model_data.lock().unwrap(), &mut current_frame);

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

  fn get_model_data(model_position: (usize, usize)) -> ModelData {
    let sprite = get_sprite();
    let strata = Strata(0);
    let hitbox = get_hitbox();
    let model_name = String::from("model");

    ModelData::new(model_position, sprite, hitbox, strata, model_name).unwrap()
  }

  fn get_sprite() -> Sprite {
    let skin = get_skin();

    Sprite::new(skin).unwrap()
  }

  fn get_skin() -> Skin {
    Skin::new(SHAPE, ANCHOR_CHAR, ANCHOR_REPLACEMENT_CHAR, AIR_CHAR).unwrap()
  }

  fn get_hitbox() -> HitboxCreationData {
    let shape = "xxx\n-c-";

    HitboxCreationData::new(shape, 'c')
  }
}
