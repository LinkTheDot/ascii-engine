// use crate::models::animation::ModelAnimationData;
use crate::models::errors::*;
use crate::models::{hitboxes::*, model_data::*, sprites::*, strata::*};
use engine_math::rectangle::Rectangle;
use log::error;
use std::io::Read;
use std::path::{Path, PathBuf};

/// The namespace for the ModelParser methods.
pub struct ModelParser;

/// The data required for building an instance of ModelData.
#[derive(Default, Debug)]
struct ModelDataBuilder {
  anchor: Option<char>,
  anchor_replacement: Option<char>,
  air: Option<char>,
  name: Option<String>,
  strata: Option<Strata>,
  appearance: Option<String>,
  hitbox_dimensions: Option<String>,

  animation_file_path: Option<Box<PathBuf>>,
}

/// Used for which section the parser is currently checking the data for.
#[derive(Debug, Eq, PartialEq)]
enum Section {
  Skin,
  Appearance,
  HitboxDimensions,
  Unknown,
}

pub(crate) struct LineComponents<'a> {
  pub(crate) data_type: &'a str,
  pub(crate) line_contents: &'a str,
}

impl ModelDataBuilder {
  /// Used to build out the ModelData from the given data from the ModelDataBulder.
  ///
  /// # Errors
  ///
  /// - Returns an error when the ModelDataBuilder is missing one or more fields of data.
  /// - Returns an error when the Appearance data had no anchor.
  fn build(self, frame_position: (usize, usize)) -> Result<ModelData, ModelError> {
    if let Err(error) = self.check_if_all_data_exists() {
      return Err(ModelError::ModelCreationError(error));
    }

    let hitbox_data = self.build_hitbox_data()?;
    let sprite = self.build_sprite()?;

    ModelData::new(
      frame_position,
      sprite,
      hitbox_data,
      self.strata.unwrap(),
      self.name.unwrap(),
    )
  }

  /// Creates a [`Sprite`](crate::models::sprites::Sprite) with the data inside of self.
  ///
  /// # Errors
  ///
  /// - Returns an error when no anchor was found on the appearance of the model.
  fn build_sprite(&self) -> Result<Sprite, ModelError> {
    let mut base_sprite = Sprite::new();
    let appearance = self.appearance.clone().unwrap();
    let anchor = self.anchor.unwrap();
    let anchor_replacement = self.anchor_replacement.unwrap();
    let air = self.air.unwrap();

    base_sprite.change_shape(appearance, None, Some(anchor))?;
    base_sprite.change_air_character(air)?;
    base_sprite.change_anchor_replacement_character(anchor_replacement);

    Ok(base_sprite)
  }

  /// Creates [`HitboxCreationData`](crate::models::hitboxes::HitboxCreationData) with the data inside of self.
  fn build_hitbox_data(&self) -> Result<HitboxCreationData, ModelError> {
    let hitbox_shape = self.hitbox_dimensions.as_ref().unwrap();
    let anchor_character = self.anchor.unwrap();

    let hitbox_dimensions = Rectangle::get_string_dimensions(hitbox_shape).unwrap();
    let anchor_index = Sprite::calculate_anchor_index(hitbox_shape, anchor_character)?;

    Ok(HitboxCreationData::new(hitbox_dimensions, anchor_index))
  }

  /// Checks if every field in the given ModelDataBuilder exists.
  /// Does not check if any of the data that does exist is valid or not.
  ///
  /// # Errors
  ///
  /// Returns the ModelCreationError::MissingData() error.
  ///
  /// - This will contain a list of every missing field as strings.
  /// - These strings will describe everything that was missing.
  fn check_if_all_data_exists(&self) -> Result<(), ModelCreationError> {
    let mut error_list = vec![];

    if self.anchor.is_none() {
      error_list.push("Anchor Character".to_string());
    }

    if self.anchor_replacement.is_none() {
      error_list.push("Anchor Replacement Character".to_string());
    }

    if self.air.is_none() {
      error_list.push("Air Character".to_string());
    }

    if self.name.is_none() {
      error_list.push("Assigned Name".to_string());
    }

    if self.strata.is_none() {
      error_list.push("Strata".to_string());
    }

    if self.appearance.is_none() {
      error_list.push("Appearance".to_string());
    }

    if error_list.is_empty() {
      Ok(())
    } else {
      error!("A model was attempted to be made with missing data: {error_list:?}");

      Err(ModelCreationError::MissingData(error_list))
    }
  }
}

impl ModelParser {
  /// Parses the passed in file of ``name.model``.
  ///
  /// # Errors
  ///
  /// - Returns an error when any data within the model file is invalid. (Yes that includes when it's empty.)
  pub fn parse(
    mut model_file: std::fs::File,
    frame_position: (usize, usize),
  ) -> Result<ModelData, ModelError> {
    let mut file_contents_buffer = String::new();
    model_file
      .read_to_string(&mut file_contents_buffer)
      .unwrap();

    if file_contents_buffer.is_empty() {
      return Err(ModelError::ModelCreationError(
        ModelCreationError::ModelFileIsEmpty,
      ));
    }

    let file_rows: Vec<&str> = file_contents_buffer.split('\n').collect();
    // let mut model_data_builder = ModelParser::parse_rows(file_rows)?;
    let model_data_builder = ModelParser::parse_rows(file_rows)?;

    // let model_animation_file_path = model_data_builder.animation_file_path.take();
    // let mut model_data = model_data_builder.build(frame_position)?;
    let model_data = model_data_builder.build(frame_position)?;

    // if let Some(model_animation_file_path) = model_animation_file_path {
    //   match ModelAnimationData::from_file(*model_animation_file_path) {
    //     Ok(animation_data) => model_data.assign_model_animation(animation_data)?,
    //     Err(animation_error) => return Err(ModelError::AnimationError(animation_error)),
    //   }
    // }

    Ok(model_data)
  }

  /// Takes the rows from the model file and adds the data to a ModelDataBuilder.
  ///
  /// Returns the ModelDataBuilder with all the data contained in the model file for creating a ModelData.
  ///
  /// # Errors
  ///
  /// - Returns an error when the syntax on any line was invalid.
  fn parse_rows(model_file_lines: Vec<&str>) -> Result<ModelDataBuilder, ModelError> {
    let mut model_data_builder = ModelDataBuilder::default();
    let mut section = Section::Unknown;
    let mut appearance_rows: Vec<&str> = vec![];
    let mut hitbox_dimension_rows: Vec<&str> = vec![];

    model_file_lines
      .iter()
      .enumerate()
      .try_for_each(|(iteration, model_file_line)| {
        // Accounts for the fact that lines start at 1 not 0.
        let line_number = iteration + 1;

        match model_file_line.to_lowercase().trim() {
          "-=--=-" => section = Section::Unknown,
          "" => section = Section::Unknown,

          "skin" => {
            section = Section::Skin;

            return Ok(());
          }
          "appearance" => {
            section = Section::Appearance;

            return Ok(());
          }
          "hitbox_dimensions" => {
            section = Section::HitboxDimensions;

            return Ok(());
          }
          _ => {
            if model_file_line.contains("+- ") {
              return Ok(());
            }
          }
        }

        match section {
          Section::Skin => {
            if let Err(error) =
              ModelParser::skin_checks(&mut model_data_builder, model_file_line, line_number)
            {
              return Err(ModelError::ModelCreationError(error));
            }
          }

          Section::Appearance => appearance_rows.push(model_file_line),
          Section::HitboxDimensions => hitbox_dimension_rows.push(model_file_line),
          Section::Unknown => return Ok(()),
        }

        Ok(())
      })?;

    let appearance = appearance_rows.join("\n");
    let hitbox_dimensions = hitbox_dimension_rows.join("\n");

    if !appearance.is_empty() {
      model_data_builder.appearance = Some(appearance);
    }
    model_data_builder.hitbox_dimensions = Some(hitbox_dimensions);

    Ok(model_data_builder)
  }

  /// Parses the given row under the ``Skin`` header and adds the data to the given ModelDataBuilder.
  ///
  /// # Errors
  ///
  /// - Returns an error when the row had invalid syntax.
  fn skin_checks(
    model_data_builder: &mut ModelDataBuilder,
    model_file_row: &str,
    line_number: usize,
  ) -> Result<(), ModelCreationError> {
    // let (data_type, row_contents) = Self::line_to_parts(model_file_row, line_number)?;
    let LineComponents {
      data_type,
      line_contents,
    } = line_to_parts(model_file_row, line_number)?;

    match data_type.to_lowercase().trim() {
      "anchor" => {
        let anchor_character = Self::contents_to_char(line_contents, line_number)?;

        model_data_builder.anchor = Some(anchor_character);
      }

      "anchor_replacement" => {
        let anchor_replacement = Self::contents_to_char(line_contents, line_number)?;

        model_data_builder.anchor_replacement = Some(anchor_replacement);
      }

      "air" => {
        let air_character = Self::contents_to_char(line_contents, line_number)?;

        model_data_builder.air = Some(air_character);
      }

      "name" => {
        if line_contents.is_empty() {
          error!("Attempted to build an object with an empty name");

          return Err(ModelCreationError::InvalidStringSizeAtLine(line_number));
        }

        model_data_builder.name = Some(line_contents.to_string());
      }

      "strata" => {
        if line_contents.is_empty() {
          error!("Attempted to build an object with an empty strata value");

          return Err(ModelCreationError::InvalidStringSizeAtLine(line_number));
        }

        let strata = match line_contents.trim().parse() {
          Ok(strata_number) => Strata(strata_number),
          Err(_) => return Err(ModelCreationError::InvalidSyntax(line_number)),
        };

        if !strata.correct_range() {
          error!("Attempted to build object with an incorrect strata range: {strata:?}");

          return Err(ModelCreationError::InvalidStrataRange(strata.0));
        }

        model_data_builder.strata = Some(strata);
      }

      // TODO Add a custom error for animation files that hold a path.
      "animation_path" => {
        if line_contents.is_empty() {
          error!("Attempted to build an object with an empty animation path");

          return Err(ModelCreationError::InvalidSyntax(line_number));
        }

        let animation_path = Path::new(line_contents);

        Self::animation_path_checks(animation_path, line_number)?;

        model_data_builder.animation_file_path = Some(Box::new(animation_path.to_owned()));
      }

      _ => return Err(ModelCreationError::InvalidSyntax(line_number)),
    }

    Ok(())
  }

  /// Checks if the contents are 1 character, and converts it into a ``char``.
  ///
  /// # Errors
  ///
  /// - Returns an error when contents > 1 character.
  fn contents_to_char(contents: &str, line_number: usize) -> Result<char, ModelCreationError> {
    if contents.len() > 1 {
      return Err(ModelCreationError::InvalidStringSizeAtLine(line_number));
    }

    contents
      .chars()
      .next()
      .ok_or(ModelCreationError::InvalidStringSizeAtLine(line_number))
  }

  fn animation_path_checks(
    animation_path: &Path,
    line_number: usize,
  ) -> Result<(), ModelCreationError> {
    if !animation_path.is_dir() {
      error!(
        "Attempted to build an object with an animation file instead of an animation directory"
      );

      Err(ModelCreationError::InvalidSyntax(line_number))
    } else if !animation_path.exists() {
      error!("Attempted to build an object with an invalid defined animation path");

      Err(ModelCreationError::InvalidSyntax(line_number))
    } else {
      Ok(())
    }
  }
}

/// Returns the components of this line.
///
/// # Errors
///
/// - Returns an error when the syntax on this line was incorrect.
pub(crate) fn line_to_parts(
  model_file_row: &str,
  line_number: usize,
) -> Result<LineComponents, ModelCreationError> {
  let (data_type, contained_row_contents) = match model_file_row.split_once('=') {
    Some(split_row) => split_row,
    None => return Err(ModelCreationError::InvalidSyntax(line_number)),
  };

  let mut row_contents = contained_row_contents.split('\'').nth(1);

  if row_contents.is_none() {
    if let Some(real_contents) = contained_row_contents.split('\"').nth(1) {
      row_contents = Some(real_contents)
    } else {
      return Err(ModelCreationError::InvalidSyntax(line_number));
    }
  }

  let row_contents = row_contents.unwrap();

  Ok(LineComponents::new(data_type, row_contents))
}

impl<'a> LineComponents<'a> {
  fn new(data_type: &'a str, line_contents: &'a str) -> Self {
    Self {
      data_type,
      line_contents,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs::File;
  use std::path::Path;

  #[test]
  fn missing_data() {
    let missing_file_path = Path::new("../tests/models/missing_data.model");
    let missing_data_file = File::open(missing_file_path).unwrap();

    let error = ModelCreationError::MissingData(get_error_list());
    let expected_result = Err(ModelError::ModelCreationError(error));

    let result = ModelParser::parse(missing_data_file, (0, 0));

    assert_eq!(result, expected_result);
  }

  fn get_error_list() -> Vec<String> {
    vec![
      "Anchor Character".to_string(),
      "Anchor Replacement Character".to_string(),
      "Air Character".to_string(),
      "Assigned Name".to_string(),
      "Strata".to_string(),
      "Appearance".to_string(),
    ]
  }

  #[test]
  fn equals_in_skin_field() {
    let file_path = Path::new("../tests/models/equals_in_skin_field.model");
    let model_file = File::open(file_path).unwrap();

    ModelParser::parse(model_file, (0, 0)).unwrap();
  }

  #[test]
  fn empty_model_file() {
    let file_path = Path::new("../tests/models/empty_file.model");
    let empty_file = File::open(file_path).unwrap();

    let error = ModelCreationError::ModelFileIsEmpty;
    let expected_result = Err(ModelError::ModelCreationError(error));

    let result = ModelParser::parse(empty_file, (0, 0));

    assert_eq!(result, expected_result);
  }

  #[test]
  fn model_with_impossible_strata() {
    let file_path = Path::new("../tests/models/wrong_strata_range.model");
    let model_file = File::open(file_path).unwrap();

    let error = ModelCreationError::InvalidStrataRange(1000);
    let expected_result = Err(ModelError::ModelCreationError(error));

    let result = ModelParser::parse(model_file, (0, 0));

    assert_eq!(result, expected_result);
  }

  #[test]
  fn model_with_no_data_in_strata() {
    let file_path = Path::new("../tests/models/empty_strata.model");
    let model_file = File::open(file_path).unwrap();

    let error = ModelCreationError::InvalidStringSizeAtLine(6);
    let expected_result = Err(ModelError::ModelCreationError(error));

    let result = ModelParser::parse(model_file, (0, 0));

    assert_eq!(result, expected_result);
  }

  #[test]
  fn model_with_no_name() {
    let file_path = Path::new("../tests/models/empty_name.model");
    let model_file = File::open(file_path).unwrap();

    let error = ModelCreationError::InvalidStringSizeAtLine(5);
    let expected_result = Err(ModelError::ModelCreationError(error));

    let result = ModelParser::parse(model_file, (0, 0));

    assert_eq!(result, expected_result);
  }

  #[test]
  fn incorrect_right_hand_side_under_skin_header() {
    let file_path = Path::new("../tests/models/incorrect_right_hand_side.model");
    let model_file = File::open(file_path).unwrap();

    let error = ModelCreationError::InvalidStringSizeAtLine(6);
    let expected_result = Err(ModelError::ModelCreationError(error));

    let result = ModelParser::parse(model_file, (0, 0));

    assert_eq!(result, expected_result);
  }

  #[test]
  fn characters_in_strata_field() {
    let file_path = Path::new("../tests/models/characters_in_strata.model");
    let model_file = File::open(file_path).unwrap();

    let error = ModelCreationError::InvalidSyntax(6);
    let expected_result = Err(ModelError::ModelCreationError(error));

    let result = ModelParser::parse(model_file, (0, 0));

    assert_eq!(result, expected_result);
  }
}
