use crate::errors::*;
use crate::general_data::file_logger;
use crate::objects::object_data::*;
use crate::screen::objects::*;
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
  object_data: Arc<RwLock<Objects>>,
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
    let object_data = Arc::new(RwLock::new(Objects::new()));

    screen_clock.start();

    Ok(ScreenData {
      screen_clock,
      object_data,
      printer: Printer::new(CONFIG.grid_width as usize, CONFIG.grid_height as usize),
      first_print: true,
      _cursor_hider: cursor_hider,
    })
  }

  /// Returns the screen as a string depending on what each pixel
  /// is assigned
  pub fn display(&self) -> Result<String, ScreenError> {
    let mut frame = Self::create_blank_frame();

    for strata_number in 0..=100 {
      let object_data = self.object_data.read().unwrap();
      guard!( let Some(strata_keys) = object_data.get_strata_keys(&Strata(strata_number)) else { continue } );

      for object in strata_keys
        .iter()
        .map(|key| self.object_data.read().unwrap().get_object(key))
      {
        guard!( let Some(object) = object else {
          error!("An object in strata {strata_number} that doesn't exist was attempted to be run.");

          continue;
        });

        let object_guard = object.lock().unwrap();

        // Instead of returning any errors here just do nothing instead.
        //
        // Returning an error here would be a problem for objects that aren't
        // suppose to be on screen.
        Self::apply_object_in_frame(object_guard, &mut frame)?;
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

    if let Err(error) = self.printer.dynamic_print(screen) {
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

  /// Waits for the input amount of ticks.
  /// The time between ticks is determined by the given value
  /// in the config file.
  // The way the clock is handled should be changed.
  // Pass around a clock_receiver instead of using the screen itself to handle that.
  pub fn wait_for_x_ticks(&mut self, x: u32) {
    // Fix the documentation on how this errors
    let _ = self.screen_clock.wait_for_x_ticks(x);
  }

  pub fn add_object<O: Object>(&mut self, object: &mut O) -> Result<(), ObjectError> {
    object.assign_object_list(self.object_data.clone());

    self
      .object_data
      .write()
      .unwrap()
      .insert(&object.get_unique_hash(), object)
  }

  fn apply_object_in_frame(
    object: MutexGuard<ObjectData>,
    current_frame: &mut String,
  ) -> Result<(), ScreenError> {
    let object_position = *object.top_left();
    let (object_width, _object_height) = object.get_sprite_dimensions();
    let air_character = object.get_air_char().to_owned();

    let object_shape = object.get_sprite().to_string().replace('\n', "");
    drop(object); // Drops the object lock early since it's no longer needed.
    let object_characters = object_shape.chars();

    // Error returned here to prevent the program from crashing when an object is found to be out of bounds.
    // Uncomment when it's fully implemented.
    // out_of_bounds_check(object_position, object_width, object_height)?;

    for (index, character) in object_characters.enumerate() {
      if character != air_character {
        let current_row_count = index / object_width;

        // (top_left_index + (row_adder + column_adder)) - column_correction
        let character_index = (object_position
          + (((CONFIG.grid_width as usize + 1) * current_row_count) + index))
          - (current_row_count * object_width);

        current_frame.replace_range(
          character_index..(character_index + 1),
          &character.to_string(),
        );
      }
    }

    Ok(())
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
  object_position: usize,
  object_width: usize,
  object_height: usize,
) -> Result<(), ScreenError> {
  // Implement all directions
  // possibly just calculate the x value for this
  // for out of bounds left,
  //   if x == grid_width then say it went out of bounds left

  if object_width + (object_position % (CONFIG.grid_width as usize + 1))
    >= CONFIG.grid_width as usize + 1
  {
    return Err(ScreenError::ObjectError(ObjectError::OutOfBounds(
      Direction::Right,
    )));
  } else if object_height + (object_position / (CONFIG.grid_width as usize + 1))
    >= CONFIG.grid_height as usize
  {
    return Err(ScreenError::ObjectError(ObjectError::OutOfBounds(
      Direction::Down,
    )));
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::general_data::coordinates::*;
  use crate::objects::hitboxes::HitboxCreationData;

  const SHAPE: &str = "x-x\nxcx\nx-x";
  const CENTER_CHAR: char = 'c';
  const CENTER_REPLACEMENT_CHAR: char = '-';
  const AIR_CHAR: char = '-';

  #[test]
  fn change_position_out_of_bounds_right() {
    let position = ((CONFIG.grid_width - 1) as usize, 15);
    let position = position.coordinates_to_index(CONFIG.grid_width as usize + 1);
    let (width, height) = (3, 3);

    let expected_result = Err(ScreenError::ObjectError(ObjectError::OutOfBounds(
      Direction::Right,
    )));

    let check_result = out_of_bounds_check(position, width, height);

    assert_eq!(check_result, expected_result);
  }

  #[test]
  fn change_position_out_of_bounds_down() {
    let position = (15, CONFIG.grid_width as usize + 1);
    let position = position.coordinates_to_index(CONFIG.grid_width as usize);
    let (width, height) = (3, 3);

    let expected_result = Err(ScreenError::ObjectError(ObjectError::OutOfBounds(
      Direction::Down,
    )));

    let check_result = out_of_bounds_check(position, width, height);

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
      let object_data = get_object_data((10, 10));
      let find_character = SHAPE.chars().next().unwrap();
      let top_left_index = *object_data.top_left();
      let object_data = Mutex::new(object_data);
      let mut current_frame = ScreenData::create_blank_frame();

      let expected_top_left_character = find_character;
      let expected_left_of_expected_character = CONFIG.empty_pixel.chars().next().unwrap();

      ScreenData::apply_object_in_frame(object_data.lock().unwrap(), &mut current_frame).unwrap();

      let object_top_left_character_in_frame = current_frame.chars().nth(top_left_index);
      let left_of_index_in_frame = current_frame.chars().nth(top_left_index - 1);

      println!("\n\n{current_frame:?}\n\n");
      println!("top_left: {top_left_index}");

      assert_eq!(
        object_top_left_character_in_frame.unwrap(),
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

  fn get_object_data(object_position: (usize, usize)) -> ObjectData {
    let sprite = get_sprite();
    let strata = Strata(0);
    let hitbox = get_hitbox();
    let object_name = String::from("object");

    ObjectData::new(object_position, sprite, hitbox, strata, object_name).unwrap()
  }

  fn get_sprite() -> Sprite {
    let skin = get_skin();

    Sprite::new(skin).unwrap()
  }

  fn get_skin() -> Skin {
    Skin::new(SHAPE, CENTER_CHAR, CENTER_REPLACEMENT_CHAR, AIR_CHAR).unwrap()
  }

  fn get_hitbox() -> HitboxCreationData {
    let shape = "xxx\n-c-";

    HitboxCreationData::new(shape, 'c')
  }
}
