#![allow(unused)]

use crate::errors::*;
use crate::models::animation::ModelAnimationData;
use crate::models::animation::{AnimationFrames, AnimationLoopCount};
use crate::models::hitboxes::valid_rectangle_check;
use crate::models::model_file_parser::{line_to_parts, LineComponents};
use std::cell::RefMut;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

/// If any line contains this character sequence, skip it.
const COMMENT_STRING: &str = "+- ";

pub struct AnimationParser;

struct AnimationData {
  name: String,
  frames: AnimationFrames,
}

#[derive(Default)]
struct AnimationBuilder {
  default_frame_duration: Option<u32>,
  animation_frames: Vec<(u32, String)>,
  animation_loop_count: Option<AnimationLoopCount>,
}

#[derive(Default)]
struct FrameBuilder {
  frame_duration: Option<u32>,
  appearance: Option<String>,
}

enum AnimationParsingSection {
  AnimationData,
  AnimationFrames,
}

impl AnimationParser {
  pub(crate) fn parse(
    animation_directories: Vec<PathBuf>,
  ) -> Result<ModelAnimationData, AnimationError> {
    let mut model_animation_data = ModelAnimationData::new();

    animation_directories
      .into_iter()
      .for_each(|animation_file_path| {
        if animation_file_path.extension() == Some(OsStr::new("animate")) {
          log::error!("Non-animation file was passed into the animation parser");

          return;
        }

        let animation_file_path_string = format!("{:?}", animation_file_path);

        match AnimationParser::get_animation_from_path(animation_file_path) {
          Ok(animation_data) => {
            if model_animation_data.contains_animation(&animation_data.name) {
              log::warn!(
                "Found animation with duplicate name: {:?}",
                animation_data.name
              );

              return;
            }

            model_animation_data
              .add_new_animation_to_list(animation_data.name.clone(), animation_data.frames)
              .unwrap();
          }
          // Errors are logged so as to not stop the entire process for
          // one incorrectly configured animation file.
          Err(error) => log::error!(
            "Animation File: {:?} Had an error and could not be parsed: {:?}",
            animation_file_path_string,
            error
          ),
        }
      });

    Ok(model_animation_data)
  }

  fn get_animation_from_path(
    animation_file_path: PathBuf,
  ) -> Result<AnimationData, AnimationParserError> {
    let animation_name = format!(
      "{:?}",
      animation_file_path.with_extension("").file_name().unwrap()
    );
    let animation_file_name = animation_file_path.as_os_str().to_owned();
    let Ok(mut animation_file) = File::open(animation_file_path) else { 
      return Err(AnimationParserError::CouldntGetAnimationPath(animation_file_name)); 
    };

    let mut file_contents_buffer = String::new();
    if let Err(error) = animation_file.read_to_string(&mut file_contents_buffer) {
      log::error!(
        "An error has occurred when reading an animation file: {:?}",
        error
      );

      return Err(AnimationParserError::CouldntReadAnimationFile);
    }

    if file_contents_buffer.is_empty() {
      return Err(AnimationParserError::AnimationFileIsEmpty);
    }

    let file_rows: Vec<&str> = file_contents_buffer.split('\n').collect();
    let animation_frames = AnimationParser::create_animation_frames(file_rows)?;

    Ok(AnimationData {
      name: animation_name,
      frames: animation_frames,
    })
  }

  // First section will contain data about the animation.
  // Data meaning the default duration for any unlabeled frames,
  // and a required loop_count field defining how many times the animation will run.
  fn create_animation_frames(
    file_lines: Vec<&str>,
  ) -> Result<AnimationFrames, AnimationParserError> {
    let mut current_section = AnimationParsingSection::AnimationData;
    let mut animation_builder = AnimationBuilder::default();
    let mut frame_builder = FrameBuilder::default();

    file_lines
      .into_iter()
      .enumerate()
      .try_for_each(|(iteration, animation_file_line)| {
        if animation_file_line.contains(COMMENT_STRING) {
          return Ok(());
        }

        match current_section {
          AnimationParsingSection::AnimationData => {
            if animation_file_line.trim() == "-" {
              current_section = AnimationParsingSection::AnimationFrames;

              return Ok(());
            }

            let Ok(line_components) = line_to_parts(animation_file_line, iteration) else {
              return Err(AnimationParserError::InvalidSyntax(iteration));
            };

            Self::animation_data_checks(&mut animation_builder, line_components, iteration)?;
          }

          AnimationParsingSection::AnimationFrames => {
            Self::animation_frame_checks(
              animation_file_line,
              iteration,
              &mut animation_builder,
              &mut frame_builder,
            )?;
          }
        };

        Ok(())
      })?;

    let Some(loop_count) = animation_builder.animation_loop_count.take() else {
      return Err(AnimationParserError::MissingLoopCount);
    };

    Ok(AnimationFrames::from((
      loop_count,
      animation_builder.animation_frames,
    )))
  }

  /// The checks for data in a line of any individual animation file.
  /// Applies the data to the passed in AnimationParsingData.
  ///
  /// # Errors
  ///
  /// - An error is returned when the passed in components are empty.
  /// - An error is returned when a variable has been defined twice.
  /// - An error is returned when the global frame duration is 0.
  /// - An error is returned when invalid contents were passed in, such as letters where an integer should be.
  /// - Any invalid syntax was found in the line's contents.
  fn animation_data_checks(
    animation_parsing_data: &mut AnimationBuilder,
    line_components: LineComponents,
    iteration: usize,
  ) -> Result<(), AnimationParserError> {
    let LineComponents {
      data_type,
      line_contents,
    } = line_components;

    match data_type.to_lowercase().trim() {
      "default_duration" => {
        if line_contents.is_empty() {
          return Err(AnimationParserError::InvalidLineContents(iteration));
        } else if animation_parsing_data.default_frame_duration.is_some() {
          return Err(AnimationParserError::DuplicateVariable(iteration));
        }

        let Ok(frame_duration) = line_contents.trim().parse() else {
          return Err(AnimationParserError::InvalidLineContents(iteration));
        };

        if frame_duration == 0 {
          return Err(AnimationParserError::FrameDurationOfZero(iteration));
        }

        animation_parsing_data.default_frame_duration = Some(frame_duration);
      }
      "loop_count" => {
        if line_contents.is_empty() {
          return Err(AnimationParserError::InvalidLineContents(iteration));
        } else if animation_parsing_data.animation_loop_count.is_some() {
          return Err(AnimationParserError::DuplicateVariable(iteration));
        }

        if line_contents.to_lowercase().trim() == "forever" {
          animation_parsing_data.animation_loop_count = Some(AnimationLoopCount::Forever);

          return Ok(());
        }

        let Ok(count) = line_contents.trim().parse() else {
                  return Err(AnimationParserError::InvalidLineContents(iteration));
                };

        animation_parsing_data.animation_loop_count = Some(AnimationLoopCount::Limited(count));
      }
      _ => return Err(AnimationParserError::InvalidSyntax(iteration)),
    }

    Ok(())
  }

  ///
  fn animation_frame_checks(
    animation_file_line: &str,
    current_line: usize,
    animation_builder: &mut AnimationBuilder,
    frame_builder: &mut FrameBuilder,
  ) -> Result<(), AnimationParserError> {
    if let Some(frame_duration) =
      Self::check_if_frame_duration_assignment(animation_file_line, current_line)?
    {
      // Checking if this line is trying to define a duration.
      // Here's what you're going to do next:
      // - Check if this is a duplicate through the FrameBuilder, if so return an error.
      // - Check if this is the first frame with the `animation_builder.has_no_frames()` method.
      //   If there are no frames, that means this is the first frame in the sequence.
      //   Which means the frame builder is not completed and that step should be skipped.
      // - Check if this is the end of this current frame, if so transfer the data with
      //   the code below, and continue on with building this frame.
      // - If none of these are the case, just push the line's contents onto the frame builder.

      if !animation_builder.has_no_frames() {
        let Some(frame_appearance) = &frame_builder.appearance else { 
          return Err(AnimationParserError::FrameHasNoAppearance);
        };

        if valid_rectangle_check(frame_appearance).is_err() {
          return Err(AnimationParserError::InvalidFrameDimensions(current_line));
        }

        frame_builder.move_frame_to_animation_builder_and_clear_self(animation_builder);
        frame_builder.reassign_frame_duration(frame_duration);
      }
    }

    Ok(())
  }

  /// If the current line is trying to assign the frame's duration, reuturns that value.
  ///
  /// # Errors
  ///
  /// - An error is returned when the line isn't formatted exactly as such:
  /// ```bash,no_run
  /// - 5 ticks
  /// ```
  /// The indicator \-, the duration of the frame, and the word `ticks` to define the measurement of time.
  fn check_if_frame_duration_assignment(
    animation_file_line: &str,
    current_line: usize,
  ) -> Result<Option<u32>, AnimationParserError> {
    if animation_file_line.starts_with("- ") {
      let line_contents: Vec<&str> = animation_file_line.split_whitespace().skip(0).collect();

      if line_contents.len() != 2 || line_contents[1].to_lowercase().trim() != "ticks" {
        return Err(AnimationParserError::InvalidSyntax(current_line));
      }

      if let Ok(frame_duration) = line_contents[0].trim().parse() {
        return Ok(Some(frame_duration));
      } else {
        return Err(AnimationParserError::InvalidLineContents(current_line));
      }
    }

    Ok(None)
  }
}

impl FrameBuilder {
  /// Clears all data from self and transfers it to the passed in AnimationParsingData.
  fn move_frame_to_animation_builder_and_clear_self(
    &mut self,
    animation_parsing_data: &mut AnimationBuilder,
  ) {
    animation_parsing_data.animation_frames.push((
      self.frame_duration.unwrap(),
      std::mem::take(&mut self.appearance.take().unwrap()),
    ));

    self.frame_duration = None;
  }

  fn reassign_frame_duration(&mut self, new_duration: u32) {
    self.frame_duration = Some(new_duration);
  }
}

impl AnimationBuilder {
  /// Returns true if the current amount of animation frames is 0.
  fn has_no_frames(&self) -> bool {
    self.animation_frames.is_empty()
  }
}
