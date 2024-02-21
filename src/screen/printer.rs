use super::model_storage::ReadOnlyModelStorage;
use crate::CONFIG;
use model_data_structures::{
  errors::*,
  models::{model_data::*, strata::*},
};
use screen_printer::printer::*;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct ScreenPrinter {
  printer: Arc<Mutex<Printer>>,
  model_storage: ReadOnlyModelStorage,
}

impl ScreenPrinter {
  pub(crate) fn new(printer: Arc<Mutex<Printer>>, model_storage: ReadOnlyModelStorage) -> Self {
    Self {
      printer,
      model_storage,
    }
  }

  // TODO: list the errors
  #[cfg(not(tarpaulin_include))]
  pub fn print_screen(&mut self) -> Result<(), ScreenError> {
    let frame = self.display();

    if let Err(error) = self.printer.lock().unwrap().dynamic_print(frame) {
      return Err(ScreenError::PrintingError(error));
    }

    Ok(())
  }

  #[cfg(not(tarpaulin_include))]
  pub fn clear_screen(&mut self) {
    self.printer.lock().unwrap().clear_grid().unwrap();
  }

  /// Creates a new frame of the world as it currently stands.
  ///
  /// This method will build out a frame for the world and return it.
  /// This could be used for when you don't want to use the built in printer and maybe want to
  /// send the data somewhere else other than a terminal.
  pub fn display(&self) -> String {
    let mut frame = Self::create_blank_frame();

    for strata_number in 0..=100 {
      let existing_models = self.model_storage.read_model_storage();

      let Some(strata_keys) = existing_models.get_strata_keys(&Strata(strata_number)) else {
        continue;
      };

      for model in strata_keys.iter().map(|key| existing_models.get_model(key)) {
        let Some(model) = model else {
          log::error!(
            "A model in strata {strata_number} that doesn't exist was attempted to be run."
          );

          continue;
        };

        Self::apply_model_in_frame(model, &mut frame);
      }
    }

    frame
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

  /// Places the appearance of the model in the given frame.
  fn apply_model_in_frame(mut model: ModelData, current_frame: &mut String) {
    let model_frame_position = model.get_frame_position();
    let model_appearance = model.get_appearance_data();
    let model_appearance = model_appearance.lock().unwrap();
    let model_sprite = model_appearance.get_appearance();
    let sprite_width = model_sprite.get_dimensions().x;
    let air_character = model_sprite.air_character();

    let model_shape = model_sprite.get_appearance().replace('\n', "");
    let model_characters = model_shape.chars();

    drop(model_appearance);

    for (index, character) in model_characters.enumerate() {
      if character == air_character || !character.is_ascii() {
        continue;
      }

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

#[cfg(test)]
mod tests {
  use super::*;
  use model_data_structures::models::testing_data::*;

  const WORLD_POSITION: (usize, usize) = (10, 10);
  const SHAPE: &str = "xxxxx\nxxaxx\nxxxxx";

  #[test]
  fn create_blank_frame() {
    let expected_pixel_count =
      ((CONFIG.grid_width * CONFIG.grid_height) + CONFIG.grid_height - 1) as usize;

    let blank_frame = ScreenPrinter::create_blank_frame();

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
      let model_data = TestingData::new_test_model(WORLD_POSITION);
      let find_character = SHAPE.chars().next().unwrap();
      let top_left_index = model_data.get_frame_position();
      let mut current_frame = ScreenPrinter::create_blank_frame();

      let expected_top_left_character = find_character;
      let expected_left_of_expected_character = CONFIG.empty_pixel.chars().next().unwrap();

      ScreenPrinter::apply_model_in_frame(model_data, &mut current_frame);

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
}
