use crate::CONFIG;
use chrono::Utc;
use log::{LevelFilter, SetLoggerError};
use log4rs::{
  append::file::FileAppender,
  config::{Appender, Config, Root},
  encode::pattern::PatternEncoder,
};

/// Creates a new log file in "crate/logs/".
/// The new log file will be named after the current time and date based on UTC.
/// The name format is as such Y-M-D-H:M:S-UTC or Year-Month-Day-Hour:Minute:Second-TimeZone.
///
/// The format in the log file can be multiple options.
/// The default is "Long".
///
/// long: "(Hour:Minute:Second)(TimeZone) | FilePath: Line | Level - Message".
/// short: "FilePath: Line | Level - Message".
/// shortest: Level - "Message"
// TODO: List the errors.
#[cfg(not(tarpaulin_include))]
pub fn setup_file_logger() -> Result<log4rs::Handle, SetLoggerError> {
  let date = Utc::now();
  let log_file_path = format!("logs/{date}").replace(' ', "-");

  let logging_format = get_logging_format();
  let log_level = get_log_level();

  let logfile = FileAppender::builder()
    .encoder(Box::new(PatternEncoder::new(&logging_format)))
    .build(log_file_path)
    .unwrap();

  let config = Config::builder()
    .appender(Appender::builder().build("logfile", Box::new(logfile)))
    .build(Root::builder().appender("logfile").build(log_level))
    .unwrap();

  log4rs::init_config(config)
}

fn get_log_level() -> LevelFilter {
  match CONFIG.log_level.to_lowercase().trim() {
    "trace" => LevelFilter::Trace,
    "info" => LevelFilter::Info,
    "error" => LevelFilter::Error,
    "warn" => LevelFilter::Warn,
    "debug" => LevelFilter::Debug,
    _ => LevelFilter::Off,
  }
}

/// To get the list of possible fields refer to the docs listed below:
/// https://docs.rs/log4rs/1.2.0/log4rs/encode/pattern/index.html#formatters
fn get_logging_format() -> String {
  let logging_format = match CONFIG.log_file_message_size.to_lowercase().trim() {
    "long" => "{d(%H:%M:%S %Z)(utc)} | {f}: {L} | {l} - {m}\n",
    "short" => "{f}: {L} - {l} - {m}\n",
    "shortest" => "{m}\n",
    _ => "{d(%H:%M:%S %Z)(utc)} | {f}: {L} | {l} - {m}\n",
  };

  logging_format.to_string()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn get_log_level_expected_default() {
    let log_level = get_log_level();

    let expected_default_log_level = LevelFilter::Debug;

    assert_eq!(log_level, expected_default_log_level);
  }

  #[test]
  fn get_logging_format_expected_default() {
    let logging_format = get_logging_format();

    let expected = "{d(%H:%M:%S %Z)(utc)} | {f}: {L} | {l} - {m}\n";

    assert_eq!(logging_format, expected);
  }
}
