use crate::errors::*;
use crate::general_data::file_logger;
use crate::models::animation_thread;
use crate::screen::model_manager::*;
use crate::screen::model_storage::*;
use crate::CONFIG;
use event_sync::EventSync;
use log::error;
use model_data_structures::models::{animation::*, model_data::*, strata::*};
use screen_printer::printer::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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
  printer: Printer,
  event_sync: EventSync,
  model_storage: Arc<RwLock<ModelStorage>>,

  /// Hides the cursor as long as this lives
  _cursor_hider: termion::cursor::HideCursor<std::io::Stdout>,

  animation_thread_connection: Option<AnimationThreadConnection>,
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
    print!("{}", termion::clear::All);

    // The handle for the file logger, isn't needed right now
    let _ = file_logger::setup_file_logger();
    let cursor_hider = termion::cursor::HideCursor::from(std::io::stdout());
    let printing_position =
      PrintingPosition::new(XPrintingPosition::Middle, YPrintingPosition::Middle);
    let model_storage: Arc<RwLock<ModelStorage>> = Default::default();

    ScreenData {
      printer: Printer::new_with_printing_position(printing_position),
      event_sync: EventSync::new(CONFIG.tick_duration),
      model_storage,
      _cursor_hider: cursor_hider,
      animation_thread_connection: None,
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
      let existing_models = self.model_storage.read().unwrap();

      let Some(strata_keys) = existing_models.get_strata_keys(&Strata(strata_number)) else { continue };

      for model in strata_keys.iter().map(|key| existing_models.get_model(key)) {
        let Some(model) = model else {
          error!("A model in strata {strata_number} that doesn't exist was attempted to be run.");

          continue;
        };

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
  /// - Returns an error if a model is overlapping on the edge of the grid.
  pub fn print_screen(&mut self) -> Result<(), ScreenError> {
    let frame = self.display();

    if let Err(error) = self.printer.dynamic_print(frame) {
      return Err(ScreenError::PrintingError(error));
    }

    Ok(())
  }

  /// Prints whitespace over the screen.
  ///
  /// This can be used to reset the grid if things get desynced from possible bugs.
  pub fn clear_screen(&mut self) {
    self.printer.clear_grid().unwrap();
  }

  /// Adds the passed in model to the list of all models in the world.
  ///
  /// Returns the hash of that model
  ///
  /// Refer to [`ModelData`](crate::models::model_data::ModelData) on how to create your own model.
  ///
  /// # Errors
  ///
  /// - An error is returned when attempting to add a model that already exists.
  pub fn add_model(&mut self, model: ModelData) -> Result<(), ModelError> {
    self.model_storage.write().unwrap().insert(model)
  }

  /// Removes the ModelData of the given key and returns it.
  ///
  /// Returns None if there's no model with the given key.
  pub fn remove_model(&mut self, key: &u64) -> Option<ModelData> {
    self.model_storage.write().unwrap().remove(key)
  }

  /// Replaces the currently existing list of all models that exist in the world with a new, empty list.
  ///
  /// Returns the list of all models that existed prior to calling this method.
  pub fn reset_world(&mut self) -> HashMap<u64, ModelData> {
    let mut existing_models = self.model_storage.write().unwrap();

    let old_world_models = std::mem::take(&mut *existing_models);

    old_world_models.extract_model_list()
  }

  pub fn get_model_manager(&self) -> ModelManager {
    ModelManager::new(self.model_storage.clone())
  }

  pub fn connect_model_manager_to_animation_thread(&self, model_manager: &mut ModelManager) {
    let Some(animation_thread_connection) = &self.animation_thread_connection else {
      return;
    };

    model_manager.add_animation_connection(animation_thread_connection.clone_sender());
  }

  /// Starts the animation thread for the screen.
  ///
  /// This allows for the use of animation methods on Models.
  ///
  /// # Errors
  ///
  /// - An error is returned if the animation thread was already started.
  pub fn start_animation_thread(&mut self) -> Result<(), ScreenError> {
    match animation_thread::start_animation_thread(self) {
      Ok(animation_connection) => self.animation_thread_connection = Some(animation_connection),
      Err(animation_error) => return Err(ScreenError::AnimationError(animation_error)),
    }

    Ok(())
  }

  pub fn stop_animation_thread(&mut self) -> Result<(), ScreenError> {
    if !self.animation_thread_started() {
      return Err(ScreenError::AnimationError(
        AnimationError::AnimationThreadNotStarted,
      ));
    }

    let animation_thread_connection = self.animation_thread_connection.take().unwrap();

    animation_thread_connection.kill_thread();

    Ok(())
  }

  pub fn animation_thread_started(&self) -> bool {
    self.animation_thread_connection.is_some()
  }

  pub fn get_event_sync(&self) -> EventSync {
    self.event_sync.clone()
  }

  /// Places the appearance of the model in the given frame.
  fn apply_model_in_frame(model: ModelData, current_frame: &mut String) {
    let model_frame_position = model.get_frame_position();
    let model_sprite = model.get_sprite();
    let model_sprite = model_sprite.read().unwrap();
    let sprite_width = model_sprite.get_dimensions().x;
    let air_character = model_sprite.air_character();

    let model_shape = model_sprite.get_appearance().replace('\n', "");
    let model_characters = model_shape.chars();

    drop(model_sprite);

    for (index, character) in model_characters.enumerate() {
      if character != air_character {
        let current_row_count = index / sprite_width;

        // (top_left_index + (row_adder + column_adder)) - column_correction
        let character_index = (model_frame_position
          + (((CONFIG.grid_width as usize + 1) * current_row_count) + index))
          - (current_row_count * sprite_width);

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
    frame.pop(); // Removes the new line left at the end.

    frame
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const WORLD_POSITION: (usize, usize) = (10, 10);
  const SHAPE: &str = "xxxxx\nxxaxx\nxxxxx";

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
    // Places the model on the screen.
    //
    // Checks if the first character in the model is equal to the first character
    // of where the model was expected to be in the frame.
    fn correct_input() {
      let model_data = new_test_model();
      let find_character = SHAPE.chars().next().unwrap();
      let top_left_index = model_data.get_frame_position();
      let mut current_frame = ScreenData::create_blank_frame();

      let expected_top_left_character = find_character;
      let expected_left_of_expected_character = CONFIG.empty_pixel.chars().next().unwrap();

      ScreenData::apply_model_in_frame(model_data, &mut current_frame);

      let model_top_left_character_in_frame = current_frame.chars().nth(top_left_index);
      let left_of_index_in_frame = current_frame.chars().nth(top_left_index - 1);

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

  #[cfg(test)]
  mod get_animation_connection_logic {
    use super::*;

    #[test]
    fn animation_not_started() {
      let screen = ScreenData::new();

      let result = screen.get_animation_connection();

      assert!(result.is_none());
    }

    #[test]
    fn animation_is_started() {
      let mut screen = ScreenData::new();
      screen.start_animation_thread().unwrap();

      let result = screen.get_animation_connection();

      assert!(result.is_some());
    }
  }

  //
  // -- Data for tests below --
  //

  fn new_test_model() -> ModelData {
    let test_model_path = std::path::Path::new("tests/models/test_square.model");
    ModelData::from_file(test_model_path, WORLD_POSITION).unwrap()
  }
}
