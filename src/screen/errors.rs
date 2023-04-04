use crate::models::errors::*;

/// The list of errors that could happen when dealing with the screen.
///
/// Includes wrappers for both
/// [`ModelError`](crate::models::errors::ModelError) and
/// [`PrintingError`](screen_printer::printer::PrintingError).
#[derive(Debug, PartialEq, Eq)]
pub enum ScreenError {
  /// A wrapper for [`ModelError`](crate::models::errors::ModelError).
  ModelError(ModelError),

  /// A wrapper for [`PrintingError`](screen_printer::printer::PrintingError).
  PrintingError(screen_printer::printer::PrintingError),

  /// This error is returned when an action was made that requires the printer to be running first.
  PrinterNotStarted,

  /// This error is returned when attempting to start the printer when it has already been started.
  PrinterAlreadyStarted,
}
