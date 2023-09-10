use crate::errors::*;
use crate::general_data::file_logger;
use crate::models::animation_thread;
use crate::screen::model_manager::*;
use crate::screen::model_storage::*;
use crate::screen::printer::*;
use crate::screen::stored_worlds::*;
use crate::CONFIG;
use event_sync::EventSync;
use model_data_structures::models::{animation::*, model_data::*};
use screen_printer::printer::*;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;
// use std::collections::VecDeque;
// use std::time::Instant;

/// ScreenData is where all the internal information required to create frames is held.
///
/// # Creation
///
/// ```
///  use ascii_engine::prelude::*;
///
///  let screen_data = ScreenData::new();
/// ```
///
/// # Usage
///
/// ```ignore
///  use ascii_engine::prelude::*;
///
///  let mut screen_data = ScreenData::new();
///  screen_data.start_printer().unwrap();
///
///  // Add models to be printed to the screen.
///
///  if let Err(error) = screen_data.print_screen() {
///    log::error!("An error has occurred while printing the screen: {error:?}");
///  }
/// ```
///
/// To create your own models refer to [`ModelData`](model_data_structures::models::model_data::ModelData).
/// For adding them to the screen look to [add_model()](crate::screen::screen_data::ScreenData::add_model()).
pub struct ScreenData {
  printer: ScreenPrinter,
  event_sync: EventSync,
  model_storage: Arc<RwLock<ModelStorage>>,

  /// Hides the terminal cursor as long as this lives
  _cursor_hider: termion::cursor::HideCursor<std::io::Stdout>,

  animation_thread_connection: Option<AnimationThreadConnection>,
}

impl ScreenData {
  /// Creates the screen.
  ///
  /// # Creation
  ///
  /// ```
  ///  use ascii_engine::prelude::*;
  ///
  ///  let screen_data = ScreenData::new();
  /// ```
  ///
  /// # Usage
  ///
  /// ```ignore
  ///  use ascii_engine::prelude::*;
  ///
  ///  let mut screen_data = ScreenData::new();
  ///  screen_data.start_printer().unwrap();
  ///
  ///  // Add models to be printed to the screen.
  ///
  ///  if let Err(error) = screen_data.print_screen() {
  ///    log::error!("An error has occurred while printing the screen: {error:?}");
  ///  }
  /// ```
  ///
  /// To create your own models refer to [`ModelData`](model_data_structures::models::model_data::ModelData).
  /// For adding them to the screen look to [add_model()](crate::screen::screen_data::ScreenData::add_model()).
  pub fn new() -> ScreenData {
    Self::new_screen(Default::default())
  }

  pub fn from_world(world: StoredWorld) -> Self {
    let stored_models = ModelStorage::from(world);

    Self::new_screen(stored_models)
  }

  fn new_screen(stored_models: ModelStorage) -> Self {
    print!("{}", termion::clear::All);

    // The handle for the file logger, isn't needed right now
    let _ = file_logger::setup_file_logger();
    let cursor_hider = termion::cursor::HideCursor::from(std::io::stdout());
    let printing_position =
      PrintingPosition::new(XPrintingPosition::Middle, YPrintingPosition::Middle);
    let printer = Printer::new_with_printing_position(printing_position);
    let model_storage: Arc<RwLock<ModelStorage>> = Arc::new(RwLock::new(stored_models));
    let printer = ScreenPrinter::new(
      Arc::new(Mutex::new(printer)),
      ModelStorage::create_read_only(model_storage.clone()),
    );

    ScreenData {
      printer,
      event_sync: EventSync::new(CONFIG.tick_duration),
      model_storage,
      _cursor_hider: cursor_hider,
      animation_thread_connection: None,
    }
  }

  /// Creates a new frame of the world as it currently stands.
  ///
  /// This method will build out a frame for the world and return it.
  /// This could be used for when you don't want to use the built in printer and maybe want to
  /// send the data somewhere else other than a terminal.
  ///
  /// If you want to print to a terminal it's best to use the
  /// [`print_screen()`](crate::screen::screen_data::ScreenData::print_screen) method for that.
  pub fn display(&self) -> String {
    self.printer.display()
  }

  /// Prints the screen as it currently is.
  ///
  /// This will use a built in printer to efficiently print to the screen.
  /// This prevents any flickers that normally appear in the terminal when printing a lot in a given time frame.
  ///
  /// # Usage
  ///
  /// ```ignore
  ///  use ascii_engine::prelude::*;
  ///
  ///  let mut screen_data = ScreenData::new();
  ///  screen_data.start_printer().unwrap();
  ///
  ///  // Add models to the screen.
  ///
  ///  if let Err(error) = screen_data.print_screen() {
  ///    log::error!("An error has occurred while printing the screen: {error:?}");
  ///  }
  /// ```
  ///
  /// # Errors
  ///
  /// - Returns an error if a model is overlapping on the edge of the grid.
  pub fn print_screen(&mut self) -> Result<(), ScreenError> {
    self.printer.print_screen()
  }

  /// Prints whitespace over the screen.
  ///
  /// This can be used to reset the grid if things get desynced from possible bugs.
  pub fn clear_screen(&mut self) {
    self.printer.clear_screen();
  }

  /// Returns a copy of the ScreenPrinter.
  ///
  /// The ScreenPrinter can be used for printing the screen, and can be passed around to different threads.
  pub fn get_screen_printer(&self) -> ScreenPrinter {
    self.printer.clone()
  }

  /// Adds the passed in model to the list of all models in the world.
  ///
  /// Returns the hash of that model
  ///
  /// Refer to [`ModelData`](model_data_structures::models::model_data::ModelData) on how to create your own model.
  ///
  /// # Errors
  ///
  /// - An error is returned when attempting to add a model that already exists.
  pub fn add_model(&mut self, model: ModelData) -> Result<(), ModelError> {
    self.model_storage.write().unwrap().insert(model)
  }

  /// Removes the ModelData of the given key and returns it.
  ///
  /// Returns None if there's no model with the given key.
  pub fn remove_model(&mut self, key: &u64) -> Option<ModelData> {
    self.model_storage.write().unwrap().remove(key)
  }

  /// Replaces the currently existing list of all models that exist in the world with a new, empty list.
  ///
  /// Returns a wrapper for the stored list of models that existed in the world.
  pub fn reset_world(&mut self) -> StoredWorld {
    let mut existing_models = self.model_storage.write().unwrap();

    let old_world_models = std::mem::take(&mut *existing_models);

    old_world_models.extract_model_list()
  }

  /// Replaces the currently stored list of existing models with the given stored list.
  ///
  /// Returns the list of models that was stored.
  pub fn load_world(&mut self, new_world: StoredWorld) -> StoredWorld {
    let mut existing_models = self.model_storage.write().unwrap();

    let old_model_list = std::mem::take(&mut *existing_models);
    *existing_models = ModelStorage::from(new_world);

    old_model_list.extract_model_list()
  }

  /// Returns a new ModelManager.
  ///
  /// The ModelManager will handle all actions requested to models in the world.
  pub fn get_model_manager(&self) -> ModelManager {
    ModelManager::new(self.model_storage.clone())
  }

  pub fn connect_model_manager_to_animation_thread(&self, model_manager: &mut ModelManager) {
    let Some(animation_thread_connection) = &self.animation_thread_connection else {
      return;
    };

    model_manager.add_animation_connection(animation_thread_connection.clone_sender());
  }

  /// Starts the animation thread for the screen.
  ///
  /// This allows for the use of animation methods on Models.
  ///
  /// # Errors
  ///
  /// - An error is returned if the animation thread was already started.
  pub fn start_animation_thread(&mut self) -> Result<(), ScreenError> {
    match animation_thread::start_animation_thread(self) {
      Ok(animation_connection) => self.animation_thread_connection = Some(animation_connection),
      Err(animation_error) => return Err(ScreenError::AnimationError(animation_error)),
    }

    Ok(())
  }

  pub fn stop_animation_thread(&mut self) -> Result<(), ScreenError> {
    if !self.animation_thread_started() {
      return Err(ScreenError::AnimationError(
        AnimationError::AnimationThreadNotStarted,
      ));
    }

    let animation_thread_connection = self.animation_thread_connection.take().unwrap();

    animation_thread_connection.kill_thread();

    Ok(())
  }

  pub fn animation_thread_started(&self) -> bool {
    self.animation_thread_connection.is_some()
  }

  pub fn get_event_sync(&self) -> EventSync {
    self.event_sync.clone()
  }

  /// Spawns a thread that will print the screen a given amount of times per second.
  ///
  /// The max printing rate is 60, and high rates of printing to the terminal can cause
  /// artifacting.
  ///
  /// Takes an optional amount of times the thread can error consecutively until it closes.
  /// If none is passed in, the thread will never close when the printer errors.
  ///
  /// Returns the JoinHandle and kill_sender to the thread.
  pub fn spawn_printing_thread(
    &self,
    printing_rate: u32,
    consecutive_errors_to_exit: Option<u32>,
  ) -> (JoinHandle<()>, oneshot::Sender<()>) {
    let mut printer = self.printer.clone();
    let printing_rate = printing_rate.max(1).min(60);
    let (kill_sender, kill_receiver) = oneshot::channel();

    let thread_handle = std::thread::spawn(move || {
      let printing_event_sync = EventSync::new(1000 / printing_rate);

      let mut consecutive_errors = 0;
      let errors_until_exit = consecutive_errors_to_exit.unwrap_or(0); // 0 will never let this check pass.

      // let mut previous_frame_durations: VecDeque<u128> = VecDeque::new();
      // let previous_frame_duration_count = 10;

      while kill_receiver.try_recv().is_err() && consecutive_errors < errors_until_exit
        || errors_until_exit == 0
      {
        // let frame_duration_average = (previous_frame_durations.iter().sum::<u128>()
        //   / (previous_frame_durations.len() as u128)
        //     .min(previous_frame_duration_count)
        //     .max(1))
        // .max(1);
        // log::debug!("Duration: {:?}", 1000000 / frame_duration_average);
        //
        // let now = Instant::now();
        //
        printing_event_sync.wait_for_tick();

        if let Err(error) = printer.print_screen() {
          log::error!(
            "An error prevented the screen from getting printed: {:?}",
            error
          );

          consecutive_errors += 1;
        } else {
          consecutive_errors = 0;
        }
        //
        // previous_frame_durations.push_front(now.elapsed().as_micros());
        // if previous_frame_durations.len() > previous_frame_duration_count as usize {
        //   previous_frame_durations.pop_back();
        // }
      }

      log::warn!(
        "CLOSING PRINTING THREAD DUE TO {} CONSECUTIVE ERRORS",
        errors_until_exit
      );
    });

    (thread_handle, kill_sender)
  }
}

#[cfg(test)]
mod tests {
  // use super::*;
  //
  // const WORLD_POSITION: (usize, usize) = (10, 10);
  // const SHAPE: &str = "xxxxx\nxxaxx\nxxxxx";
  //
  // #[cfg(test)]
  // mod get_animation_connection_logic {
  //   use super::*;
  //
  //   #[test]
  //   fn animation_not_started() {
  //     let screen = ScreenData::new();
  //
  //     let result = screen.get_animation_connection();
  //
  //     assert!(result.is_none());
  //   }
  //
  //   #[test]
  //   fn animation_is_started() {
  //     let mut screen = ScreenData::new();
  //     screen.start_animation_thread().unwrap();
  //
  //     let result = screen.get_animation_connection();
  //
  //     assert!(result.is_some());
  //   }
  // }
}
