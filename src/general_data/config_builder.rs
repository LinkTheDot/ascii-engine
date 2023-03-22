use config::{builder::DefaultState, Config, ConfigBuilder, ConfigError, File};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::{fs::OpenOptions, path::Path};

/// The list of options for the config.
#[derive(Deserialize, Serialize)]
pub struct ConfigData {
  pub log_level: String,
  pub empty_pixel: String,

  /// In milliseconds
  pub tick_duration: u32,
  pub grid_width: u32,
  pub grid_height: u32,
}

impl Default for ConfigData {
  fn default() -> Self {
    Self {
      log_level: "debug".to_string(),
      empty_pixel: " ".to_string(),
      tick_duration: 24,
      grid_width: 175,
      grid_height: 40,
    }
  }
}

/// Returns a new ConfigData.
///
/// If a config.toml didn't already exist
/// a new one is created and set with default values.
pub fn get_config() -> Result<ConfigData, ConfigError> {
  let default_config_data = ConfigData::default();
  let config_path_name = "config.toml";
  let config_path = Path::new(config_path_name);

  if !config_path.exists() {
    let _ = create_missing_config_file(config_path, &default_config_data);
  }

  Config::builder()
    .set_defaults(default_config_data)?
    .add_source(File::with_name(config_path_name))
    .build()?
    .try_deserialize()
}

/// Creates a new config file with their default values.
fn create_missing_config_file(
  config_path: &Path,
  default_config_data: &ConfigData,
) -> anyhow::Result<()> {
  let serialized_config_data = toml::to_string(&default_config_data)?;

  let mut config_file = OpenOptions::new()
    .write(true)
    .create_new(true)
    .open(config_path)?;

  write!(config_file, "{serialized_config_data}")?;

  Ok(())
}

trait ConfigTraits {
  /// Assigns the default values given by "ConfigData::default()" to the config.
  ///
  /// Assigned defaults will correct any missing fields in the config file.
  /// For when the entire file is missing, look to "create_missing_config_file()".
  ///
  /// # Errors
  ///
  /// An error is returned when one of the input names contains non-ascii characters.
  fn set_defaults(self, default_data: ConfigData) -> Result<Self, ConfigError>
  where
    Self: Sized;
}

impl ConfigTraits for ConfigBuilder<DefaultState> {
  fn set_defaults(self, default_data: ConfigData) -> Result<Self, ConfigError>
  where
    Self: Sized,
  {
    self
      .set_default("log_level", default_data.log_level)?
      .set_default("empty_pixel", default_data.empty_pixel)?
      .set_default("tick_duration", default_data.tick_duration)?
      .set_default("grid_width", default_data.grid_width)?
      .set_default("grid_height", default_data.grid_height)
  }
}
