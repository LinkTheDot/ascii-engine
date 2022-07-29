#![allow(unused)]

use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::*;
use std::thread;
use std::time::Duration;

pub const GRID_WIDTH: usize = 5; // temporary number testing is needed

pub struct ScreenData {
  screen_update_receiver: Receiver<String>,
  screen_update_sender: Sender<String>,
  screen: Vec<Pixel>,
}

pub struct Pixel {
  objects_within: Vec<String>,
}

impl Pixel {
  pub fn new() -> Self {
    Pixel {
      objects_within: vec![],
    }
  }
}

impl ScreenData {
  pub fn new() -> Self {
    let (screen_update_sender, screen_update_receiver) = channel();
    let screen = generate_pixel_grid();

    ScreenData {
      screen_update_receiver,
      screen_update_sender,
      screen,
    }
  }
}

pub fn run_screen() {
  todo!()
}

pub fn generate_pixel_grid() -> Vec<Pixel> {
  todo!()
}
