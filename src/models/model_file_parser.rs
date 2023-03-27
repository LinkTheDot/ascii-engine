use crate::models::errors::*;
use crate::models::{hitboxes::*, model_data::*};
use guard::guard;
use log::error;
use std::io::Read;

pub struct ModelParser;

#[derive(Default, Debug)]
struct ModelDataBuilder {
  center: Option<char>,
  center_replacement: Option<char>,
  air: Option<char>,
  name: Option<String>,
  strata: Option<Strata>,
  appearance: Option<String>,
  hitbox_dimensions: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
enum Section {
  Skin,
  Appearance,
  HitboxDimensions,
  Unknown,
}

impl ModelDataBuilder {
  fn build(self, frame_position: (usize, usize)) -> Result<ModelData, ModelError> {
    if let Err(error) = self.check_if_all_data_exists() {
      return Err(ModelError::ModelCreationError(error));
    }

    let hitbox_data = self.build_hitbox_data();
    let sprite = self.build_sprite()?;

    ModelData::new(
      frame_position,
      sprite,
      hitbox_data,
      self.strata.unwrap(),
      self.name.unwrap(),
    )
  }

  fn build_sprite(&self) -> Result<Sprite, ModelError> {
    let skin = Skin::new(
      self.appearance.as_ref().unwrap(),
      self.center.unwrap(),
      self.center_replacement.unwrap(),
      self.air.unwrap(),
    )?;

    Sprite::new(skin)
  }

  fn build_hitbox_data(&self) -> HitboxCreationData {
    let hitbox_shape = &self.hitbox_dimensions.as_ref().unwrap();
    let center_character = self.center.unwrap();

    HitboxCreationData::new(hitbox_shape, center_character)
  }

  /// Checks if every field in the given ModelDataBuilder exists.
  /// Does not check if any of the data that does exist is valid or not.
  ///
  /// # Errors
  ///
  /// Returns the ModelCreationError::MissingData() error.
  ///
  /// This will contain a list of every missing field as strings.
  /// These strings will describe everything that was missing.
  fn check_if_all_data_exists(&self) -> Result<(), ModelCreationError> {
    let mut error_list = vec![];

    if self.center.is_none() {
      error_list.push("Center Character".to_string());
    }

    if self.center_replacement.is_none() {
      error_list.push("Center Replacement Character".to_string());
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

    if self.hitbox_dimensions.is_none() {
      error_list.push("Hitbox Dimensions".to_string());
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
  /// Comments: if "+- " is anywhere on a line, it'll be ignored
  /// Spliters: Are -=--=-
  pub fn parse(
    mut model_file: std::fs::File,
    frame_position: (usize, usize),
  ) -> Result<ModelData, ModelError> {
    let mut file_string_buffer = String::new();
    model_file.read_to_string(&mut file_string_buffer).unwrap();

    if file_string_buffer.is_empty() {
      return Err(ModelError::ModelCreationError(
        ModelCreationError::ModelFileIsEmpty,
      ));
    }

    let file_string_rows: Vec<&str> = file_string_buffer.split('\n').collect();
    let model_data_builder = ModelParser::parse_rows(file_string_rows)?;

    model_data_builder.build(frame_position)
  }

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
          // Spacers
          // - --
          // - =
          "-=--=-" => section = Section::Unknown,
          "" => section = Section::Unknown,

          // Headers
          // - Skin
          // - HitboxDimensions
          // - Appearance
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
            // Contents
            // - center
            // - center_replacement
            // - air
            // - name
            // - strata

            // Containers
            // - ''
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

    model_data_builder.appearance = Some(appearance);
    model_data_builder.hitbox_dimensions = Some(hitbox_dimensions);

    Ok(model_data_builder)
  }

  fn skin_checks(
    model_data_builder: &mut ModelDataBuilder,
    model_file_row: &str,
    line_number: usize,
  ) -> Result<(), ModelCreationError> {
    let split_row: Vec<&str> = model_file_row.split('=').collect();

    if split_row.len() != 2 {
      return Err(ModelCreationError::InvalidSyntax(line_number));
    }

    let data_type = split_row[0];
    let contained_row_contents = split_row[1];

    guard!( let Some(row_contents) = contained_row_contents.split('\'').nth(1) else {
      return Err(ModelCreationError::InvalidSyntax(line_number));
    });

    match data_type.to_lowercase().trim() {
      "center" => {
        let center_character = Self::contents_to_char(row_contents, line_number)?;

        model_data_builder.center = Some(center_character);
      }

      "center_replacement" => {
        let center_replacement = Self::contents_to_char(row_contents, line_number)?;

        model_data_builder.center_replacement = Some(center_replacement);
      }

      "air" => {
        let air_character = Self::contents_to_char(row_contents, line_number)?;

        model_data_builder.air = Some(air_character);
      }

      "name" => {
        if row_contents.is_empty() {
          error!("Attempted to build an object with an empty name");

          return Err(ModelCreationError::InvalidStringSizeAtLine(line_number));
        }

        model_data_builder.name = Some(row_contents.to_string());
      }

      "strata" => {
        if row_contents.is_empty() {
          error!("Attempted to build an object with an empty strata value");

          return Err(ModelCreationError::InvalidStringSizeAtLine(line_number));
        }

        let strata = match row_contents.trim().parse() {
          Ok(strata_number) => Strata(strata_number),
          Err(_) => return Err(ModelCreationError::InvalidSyntax(line_number)),
        };

        if !strata.correct_range() {
          error!("Attempted to build object with an incorrect strata range: {strata:?}");

          return Err(ModelCreationError::InvalidStrataRange(strata.0));
        }

        model_data_builder.strata = Some(strata);
      }

      _ => return Err(ModelCreationError::InvalidSyntax(line_number)),
    }

    Ok(())
  }

  fn contents_to_char(contents: &str, line_number: usize) -> Result<char, ModelCreationError> {
    if contents.len() > 1 {
      return Err(ModelCreationError::InvalidStringSizeAtLine(line_number));
    }

    contents
      .chars()
      .next()
      .ok_or(ModelCreationError::InvalidStringSizeAtLine(line_number))
  }
}
