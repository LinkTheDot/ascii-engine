#![allow(unused)]

use crate::errors::*;
use crate::models::animation::ModelAnimationData;

pub struct AnimationParser;

impl AnimationParser {
  pub(crate) fn parse(
    _animation_file: std::fs::File,
  ) -> Result<ModelAnimationData, AnimationError> {
    todo!()
  }
}
