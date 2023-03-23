use crate::models::errors::*;
use crate::models::{hitboxes::*, model_data::*};
use log::{debug, error};
use std::io::Read;

pub struct ModelParser;

#[derive(Default)]
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
  fn build(self, position: (usize, usize)) -> Result<ModelData, ModelError> {
    if let Err(error) = self.check_if_all_data_is_exists() {
      return Err(ModelError::ModelCreationError(error));
    }

    let hitbox_data = self.build_hitbox_data();
    let sprite = self.build_sprite()?;

    ModelData::new(
      position,
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
  /// These strings will desribe everything that was missing.
  fn check_if_all_data_is_exists(&self) -> Result<(), ModelCreationError> {
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
  #[allow(clippy::wildcard_in_or_patterns)]
  pub fn parse(
    mut model_file: std::fs::File,
    position: (usize, usize),
  ) -> Result<ModelData, ModelError> {
    let mut file_string = String::new();
    model_file.read_to_string(&mut file_string).unwrap();

    if file_string.is_empty() {
      let error = ModelCreationError::ModelFileIsEmpty;

      return Err(ModelError::ModelCreationError(error));
    }

    let file_string_rows = file_string.split('\n');
    let mut model_data_builder = ModelDataBuilder::default();
    let mut section = Section::Unknown;
    let mut appearance_rows: Vec<&str> = vec![];
    let mut hitbox_dimension_rows: Vec<&str> = vec![];

    for (line_number, file_row) in file_string_rows.enumerate() {
      // Accounts for the fact that lines start at 1 not 0.
      let line_number = line_number + 1;

      // Headers
      // - SKIN
      // - HitboxDimensions
      // - Appearance
      //
      // Contents
      // - center
      // - center_replacement
      // - air
      // - name
      // - strata
      //
      // Spacers
      // - --
      // - =
      //
      // Containers
      // - ''

      match file_row.to_lowercase().trim() {
        "--" => section = Section::Unknown,
        "" => section = Section::Unknown,
        "skin" => {
          section = Section::Skin;

          continue;
        }
        "appearance" => {
          section = Section::Appearance;

          continue;
        }
        "hitbox_dimensions" => {
          section = Section::HitboxDimensions;

          continue;
        }
        _ => (),
      }

      match section {
        Section::Skin => {
          debug!("In Skin");
          section = Section::Skin;

          if let Err(error) =
            ModelParser::skin_checks(&mut model_data_builder, file_row, line_number)
          {
            return Err(ModelError::ModelCreationError(error));
          }
        }

        Section::Appearance => appearance_rows.push(file_row),
        Section::HitboxDimensions => hitbox_dimension_rows.push(file_row),
        _ => continue,
      }
    }

    let appearance = appearance_rows.join("\n");
    let hitbox_dimensions = hitbox_dimension_rows.join("\n");

    model_data_builder.appearance = Some(appearance);
    model_data_builder.hitbox_dimensions = Some(hitbox_dimensions);

    model_data_builder.build(position)
  }

  fn skin_checks(
    model_data_builder: &mut ModelDataBuilder,
    file_row: &str,
    line_number: usize,
  ) -> Result<(), ModelCreationError> {
    let mut data_type = String::new();
    let mut row_contents = String::new();

    let mut left_hand_side = true;
    let mut in_string = false;

    // file_row.chars().for_each(|character| {
    for character in file_row.chars() {
      match character {
        '=' => {
          left_hand_side = false;
        }
        '\'' => {
          in_string = !in_string;

          continue;
        }
        _ => (),
      };

      if left_hand_side {
        data_type.push(character);
      } else if in_string {
        row_contents.push(character);
      }
    }
    // );

    match data_type.to_lowercase().trim() {
      "center" => {
        debug!("Got to center: {}", row_contents);
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

        model_data_builder.name = Some(row_contents);
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

  fn contents_to_char(contents: String, line_number: usize) -> Result<char, ModelCreationError> {
    if contents.len() != 1 {
      return Err(ModelCreationError::InvalidStringSizeAtLine(line_number));
    }

    Ok(contents.chars().next().unwrap())
  }
}
