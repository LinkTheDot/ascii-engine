use config::{Config, ConfigError, File};
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
}

impl Default for ConfigData {
  fn default() -> Self {
    Self {
      log_level: "debug".to_string(),
      empty_pixel: " ".to_string(),
      tick_duration: 24,
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
    .set_default("log_level", default_config_data.log_level)?
    .set_default("empty_pixel", default_config_data.empty_pixel)?
    .set_default("tick_duration", default_config_data.tick_duration)?
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

  write!(config_file, "{}", serialized_config_data)?;

  Ok(())
}
