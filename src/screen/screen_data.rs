use crate::errors::*;
use crate::general_data::file_logger;
use crate::models::animation::{AnimationConnection, AnimationRequest, ModelAnimationData};
use crate::models::model_data::*;
use crate::screen::model_storage::*;
use crate::CONFIG;
use event_sync::EventSync;
use log::error;
use screen_printer::printer::*;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

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
  model_data: Arc<RwLock<InternalModels>>,
  printer: Printer,
  event_sync: EventSync,

  /// Hides the cursor as long as this lives
  _cursor_hider: termion::cursor::HideCursor<std::io::Stdout>,

  animation_thread_connection: Option<AnimationConnection>,
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
    let model_data = Arc::new(RwLock::new(InternalModels::new()));
    let printing_position =
      PrintingPosition::new(XPrintingPosition::Middle, YPrintingPosition::Middle);

    ScreenData {
      model_data,
      printer: Printer::new_with_printing_position(printing_position),
      event_sync: EventSync::new(CONFIG.tick_duration),
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
      let model_data = self.model_data.read().unwrap();

      let Some(strata_keys) = model_data.get_strata_keys(&Strata(strata_number)) else { continue };

      for model in strata_keys
        .iter()
        .map(|key| self.model_data.read().unwrap().get_model(key))
      {
        let Some(model) = model else {
          // This is left here just incase something comes up that allows it to happen.
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
  ///
  /// # Errors
  ///
  /// - Returns an error if the printer hasn't been started with [`start_printer()`](crate::screen::screen_data::ScreenData::start_printer).
  pub fn clear_screen(&mut self) -> Result<(), ScreenError> {
    self.printer.clear_grid().unwrap();

    Ok(())
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

  /// Removes the ModelData of the given key and returns it.
  ///
  /// Returns None if there's no model with the given key.
  pub fn remove_model(&mut self, key: &u64) -> Option<ModelData> {
    self.model_data.write().unwrap().remove(key)
  }

  /// Starts the animation thread for the screen.
  ///
  /// This allows for the use of animation methods on Models.
  ///
  /// # Errors
  ///
  /// - An error is returned if the animation thread was already started.
  pub fn start_animation_thread(&mut self) -> Result<(), ScreenError> {
    match ModelAnimationData::start_animation_thread(self) {
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

  /// Returns a copy of the AnimationRequest sender for animation threads.
  ///
  /// None is returned if [`screen_data.start_animation_thread()`](ScreenData::start_animation_thread) hasn't been called yet.
  pub(crate) fn get_animation_connection(&self) -> Option<mpsc::UnboundedSender<AnimationRequest>> {
    Some(self.animation_thread_connection.as_ref()?.clone_sender())
  }

  pub fn get_event_sync(&self) -> EventSync {
    self.event_sync.clone()
  }

  /// Places the appearance of the model in the given frame.
  fn apply_model_in_frame(model: ModelData, current_frame: &mut String) {
    let model_frame_position = model.top_left();
    let (model_width, _model_height) = model.get_sprite_dimensions();
    let air_character = model.get_air_char();

    let model_shape = model.get_sprite().replace('\n', "");
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
  use super::*;
  use engine_math::coordinates::*;

  const WORLD_POSITION: (usize, usize) = (10, 10);
  const SHAPE: &str = "xxxxx\nxxaxx\nxxxxx";

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
    // Places the model on the screen.
    //
    // Checks if the first character in the model is equal to the first character
    // of where the model was expected to be in the frame.
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
