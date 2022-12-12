use crate::CONFIG;
use chrono::Utc;
use log::{LevelFilter, SetLoggerError};
use log4rs::{
  append::file::FileAppender,
  config::{Appender, Config, Root},
  encode::pattern::PatternEncoder,
};

/// Setups up the file logger for the program
pub fn setup_file_logger() -> Result<log4rs::Handle, SetLoggerError> {
  let date = Utc::now();
  let log_file_path = format!("logs/{}", date).replace(' ', "-");
  let logging_format = "{d(%H:%M:%S %Z)(utc)} | {f}: {L} | {l} - {m}\n";

  let log_level = get_log_level();

  let logfile = FileAppender::builder()
    .encoder(Box::new(PatternEncoder::new(logging_format)))
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
