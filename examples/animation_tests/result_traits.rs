pub trait ResultTraits<T> {
  /// Logs the result if it's an error.
  /// The message will be under 'Error' when logged.
  fn log_if_err(self) -> Option<T>;
}

impl<T, E> ResultTraits<T> for Result<T, E>
where
  E: std::fmt::Debug,
{
  fn log_if_err(self) -> Option<T> {
    if let Err(error) = self {
      log::error!("{error:?}");

      return None;
    }

    self.ok()
  }
}
