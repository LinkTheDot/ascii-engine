use crate::objects::errors::*;

#[derive(Debug, PartialEq, Eq)]
pub enum ScreenError {
  ObjectError(ObjectError),

  PrintingError(screen_printer::printer::PrintingError),

  NoExistingObjects,
}
