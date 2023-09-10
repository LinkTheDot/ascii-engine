use config::{builder::DefaultState, Config, ConfigBuilder, ConfigError, File};
use serde::{Deserialize, Serialize};
use std::env;
use std::ffi::OsStr;
use std::io::Write;
use std::{fs::OpenOptions, path::Path};

/// The list of options for the config.
#[derive(Deserialize, Serialize)]
pub struct ConfigData {
  pub log_level: String,
  pub log_file_message_size: String,
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
      log_file_message_size: "long".to_string(),
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
  let config_path_name = get_config_path_name();
  let config_path = Path::new(&config_path_name);

  if !config_path.exists() {
    let _ = create_missing_config_file(config_path, &default_config_data);
  }

  Config::builder()
    .set_data(default_config_data, &config_path_name)?
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

  write!(config_file, "{}", serialized_config_data)?;

  Ok(())
}

fn get_config_path_name() -> String {
  if let Some(potential_path) = env::args().nth(1) {
    let path = Path::new(&potential_path);

    if path.exists() && path.extension() == Some(OsStr::new(".toml")) {
      return potential_path;
    }
  }

  "config.toml".to_string()
}

trait ConfigTraits {
  /// Assigns the default and config_file data.
  ///
  /// Any missing fields from "config.toml" will be replaced with their defaults.
  ///
  /// If the program is running off of a "test" build, then the config will
  /// only be built with the default data.
  ///
  /// # Errors
  ///
  /// - An error is returned when one of the default config names contains non-ascii characters.
  fn set_data(self, default_data: ConfigData, path: &str) -> Result<Self, ConfigError>
  where
    Self: Sized;

  /// Assigns the default values given by "ConfigData::default()" to the config.
  ///
  /// Assigned defaults will correct any missing fields in the config file.
  /// For when the entire file is missing, look to "create_missing_config_file()".
  ///
  /// # Errors
  ///
  /// - An error is returned when one of the default config names contains non-ascii characters.
  fn set_defaults(self, default_data: ConfigData) -> Result<Self, ConfigError>
  where
    Self: Sized;
}

impl ConfigTraits for ConfigBuilder<DefaultState> {
  fn set_data(self, default_data: ConfigData, path: &str) -> Result<Self, ConfigError>
  where
    Self: Sized,
  {
    let config_with_defaults = self.set_defaults(default_data);

    if !running_on_test_build() {
      Ok(config_with_defaults?.add_source(File::with_name(path)))
    } else {
      config_with_defaults
    }
  }

  fn set_defaults(self, default_data: ConfigData) -> Result<Self, ConfigError>
  where
    Self: Sized,
  {
    self
      .set_default("log_level", default_data.log_level)?
      .set_default("log_file_message_size", default_data.log_file_message_size)?
      .set_default("empty_pixel", default_data.empty_pixel)?
      .set_default("tick_duration", default_data.tick_duration)?
      .set_default("grid_width", default_data.grid_width)?
      .set_default("grid_height", default_data.grid_height)
  }
}

fn running_on_test_build() -> bool {
  cfg!(test)
}
