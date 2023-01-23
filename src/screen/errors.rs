use crate::objects::errors::*;

#[derive(Debug)]
pub enum ScreenError {
  ObjectError(ObjectError),

  PrintingError(screen_printer::printer::PrintingError),
}
