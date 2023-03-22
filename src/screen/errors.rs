use crate::models::errors::*;

#[derive(Debug, PartialEq, Eq)]
pub enum ScreenError {
  ModelError(ModelError),

  PrintingError(screen_printer::printer::PrintingError),

  NoExistingModels,
}
