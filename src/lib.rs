//! # Ascii Engine
//!
//! Ascii Engine is a terminal based graphics engine.
//!
//! ## Imports
//!
//! Most things that you'll need can be imported through the prelude.
//!
//! ```rust
//! use ascii_engine::prelude::*;
//! ```
//!
//! ## Usage
//!
//! First your project will need to start by creating the Screen. This can be done
//! like so:
//!
//! ```rust
//! use ascii_engine::prelude::*;
//!
//! let screen_data = ScreenData::new();
//! ```
//!
//! Next you'll want to start the printer for the screen.
//!
//! ```rust,no_run,ignore
//! screen_data.start_printer().unwrap();
//! ```
//!
//! Once you've started the printer,, all you need to do is call the
//! `print_screen()` method.
//!
//! ```rust,no_run,ignore
//! screen_data.print_screen().unwrap();
//! ```
//!
//! This will build and print a frame for the screen.
//!
//! # Models
//!
//! Models also referred to as DisplayModels, contain data for the screen to display
//! and also how models interact with each other.
//!
//! Creating a model requires the creating of a model file.
//!
//! Once you have a model file you can use the `from_file(path, world_position)`
//! method to create an instance of ModelData for your model.
//!
//! #### Example
//!
//! ```rust,no_run
//! use ascii_engine::prelude::*;
//! use std::path::Path;
//!
//! let model_path = Path::new("models/my_model.model");
//! let my_model_world_position = (10, 10);
//! let model_data = ModelData::from_file(model_path, my_model_world_position).unwrap();
//! ```
//!
//! ## Model File Creation
//!
//! To create a model file you'll start by making a file named as such:
//!
//! ```no_run,bash,ignore
//! model_name.model
//! ```
//!
//! Once you have your model file, you'll need to feed it the data for your model.
//!
//! A model file is formatted as such:
//!
//! ```no_run,bash,ignore
//! - Header
//! - Data
//! - Spacer
//! ```
//!
//! The required headers are
//!
//! ```no_run,bash,ignore
//! - Skin
//! - Appearance
//! - Hitbox_Dimensions
//! ```
//!
//! The available spacer is
//!
//! ```no_run,bash,ignore
//! -=--=-
//! ```
//!
//! The data can differ from header to header.
//!
//! ## Skin Data
//!
//! NONE of these fields can be `=`. The `name` field can NOT contain `'`. The
//! required fields under the "Skin" header are as such:
//!
//! - anchor (This is the assigned character for a model's hitbox and world
//!   placement anchor)
//!
//! ```no_run,bash,ignore
//! anchor="a"
//! ```
//!
//! - anchor_replacement (This is the character that will replace the anchor
//!   character. This can be thought of as the "fix" for when you're building out
//!   the appearance of a model)
//!
//! ```no_run,bash,ignore
//! anchor_replacement="-"
//! ```
//!
//! - air (This is the character that will be designated as the model's air. Air
//!   will be transparent on the screen)
//!
//! ```no_run,bash,ignore
//! air="-"
//! ```
//!
//! - name (This is the assigned name for the model. The name can be used to
//!   identify collisions and what you want to do depending on the collided model)
//!
//! ```no_run,bash,ignore
//! name="Square"
//! ```
//!
//! - strata (The strata is what layer the model is on the screen. Refer to the
//!   Strata struct for more information)
//!
//! ```no_run,bash,ignore
//! strata="95"
//! ```
//!
//! ## Appearance
//!
//! This will be how your model looks on the screen. The appearance must be
//! rectangular in shape.
//!
//! To build a non-rectangular look to your model, you can use the air character
//! defined under the "Skin" header to have a transparent pixel.
//!
//! Your model's appearance requires you to have an anchor character. The anchor
//! character will be used to dictate where the model is placed on the screen. The
//! anchor also dictates where the anchor for Hitbox_Dimensions will be placed
//! relative to your model's appearance.
//!
//! The appearance field will look something like this:
//!
//! ```no_run,bash,ignore
//! =====
//! |-a-|
//! =====
//! ```
//!
//! ## Hitbox_Dimensions
//!
//! The Hitbox_Dimensions field is very similar to the Appearance.
//!
//! Just like the appearance, the Hitbox_Dimensions requires one anchor character to
//! be assigned within it. The anchor character will dictate where the hitbox is
//! placed relative to the anchor in the appearance.
//!
//! This will dictate the size of your model's hitbox, and it must be a rectangular
//! shape. Any character that isn't the anchor will be accepted for dictating the
//! dimensions of the hitbox.
//!
//! The Hitbox_Dimensions field will look something like this:
//!
//! ```no_run,bash,ignore
//! =====
//! ==a==
//! =====
//! ```
//!
//! # Creating a file
//!
//! Now that all of the information required to make a model has been defined, we
//! can get to actually making one.
//!
//! Here's a mock model file of a simple square model.
//!
//! ```no_run,bash,ignore
//! Skin
//! anchor="a"
//! anchor_replacement="-"
//! air="-"
//! name="Square"
//! strata="95"
//! -=--=-
//! Appearance
//! =====
//! |-a-|
//! =====
//! -=--=-
//! Hitbox_Dimensions
//! =====
//! ==a==
//! =====
//! -=--=-
//! ```
//!
//! First we define the `Skin` header.
//!
//! Next we assign our character for the anchor.
//!
//! With our anchor assigned, we now assign a character to replace it so we don't
//! have a random `a` on our model. Here we go with the air character, because I
//! want the square to have a hole in the center.
//!
//! From there we assign air to `-`,
//!
//! We give our model a name, in this case `Square`.
//!
//! Lastly we give it a strata of `95`, meaning anything that overlaps with our
//! square, and that has a strata < 95, will be under the square.
//!
//! ## Comments
//!
//! It should be known that the character sequence `+-` anywhere in a line is
//! reserved for comments. This means if you put `+-` anywhere on a line in your
//! model file, that line will be ignored by the parser.
//!
//! ## DisplayModel Trait
//!
//! Now that you know how to create ModelData. You need to learn how to use it.
//!
//! First, you must create your own struct that will contain the ModelData. This
//! struct will derive the `DisplayModel` trait.
//!
//! ```rust
//! use ascii_engine::prelude::*;
//!
//! #[derive(DisplayModel)]
//! struct Square {
//!   model_data: ModelData,
//! }
//! ```
//!
//! With DisplayModel, this will allow you to add this struct to the screen.
//!
//! #### Adding your model to the screen.
//!
//! ```rust,no_run,ignore
//! use ascii_engine::prelude::*;
//! use std::path::Path;
//!
//! #[derive(DisplayModel)]
//! struct Square {
//!   model_data: ModelData,
//! }
//!
//! let mut screen = ScreenData::new();
//! screen.start_printer().unwrap();
//!
//! let square_model_path = Path::new("models/square.model");
//! let square_model_data = ModelData::from_file(square_model_path, (10, 10)).unwrap();
//! let square = Square { model_data: square_model_data };
//!
//! // Now that everything is setup, we can add our square to the screen.
//! screen.add_model(&square).unwrap();
//! ```
//!
//! From here, we can do just about anything we want.
//!
//! ## More Depth
//!
//! For a more in depth example of how to really get into using models, look at the
//! `model_tests` example in the examples directory.

use crate::general_data::config_builder;
use lazy_static::lazy_static;

lazy_static! {
  // Only way this can cause an error is if the code for the config builder was done wrong.
  pub static ref CONFIG: config_builder::ConfigData = config_builder::get_config().unwrap();
}

pub mod defaults;
pub mod errors;
pub mod prelude;

pub mod general_data {
  pub mod config_builder;
  pub mod coordinates;
  pub mod file_logger;
  pub mod hasher;
  pub mod user_input;
}

pub mod models {
  pub mod animation;
  pub mod animation_file_parser;
  pub mod errors;
  pub mod hitboxes;
  pub mod model_data;
  pub mod model_file_parser;
  pub mod sprites;
  pub mod traits;
}

pub mod screen {
  pub mod errors;
  pub mod models;
  pub mod screen_data;
}
