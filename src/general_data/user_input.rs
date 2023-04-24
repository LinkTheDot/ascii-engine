use crate::screen::{errors::ScreenError, screen_data::ScreenData};
use log::error;
use log::info;
use oneshot::Sender;
use std::io;
use std::io::{Read, Write};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

const ERROR_CHARACTER: &str = "|";

/// Gets a user's input without canonical mode.
///
/// If anything unexpected happens the ERROR_CHARACTER is returned.
fn get_user_input() -> String {
  let stdin = 0;

  let mut termios = Termios::from_fd(stdin).unwrap();

  termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
  tcsetattr(stdin, TCSANOW, &termios).unwrap();

  let mut buffer = vec![0; 1]; // read exactly one byte

  io::stdout().lock().flush().unwrap();

  if io::stdin().read_exact(&mut buffer).is_err() {
    error!("The input thread was interrupted when attempting to read from stdin");

    tcsetattr(stdin, TCSANOW, &termios).unwrap(); // reset the stdin to original termios data

    return String::from(ERROR_CHARACTER);
  }

  tcsetattr(stdin, TCSANOW, &termios).unwrap(); // reset the stdin to original termios data

  String::from_utf8(buffer.to_vec()).unwrap()
}

/// Spawns a thread that will request an input from the user at every moment.
///
/// The Receiver for the user's input, and a sender to kill the input thread is returned.
///
/// DO NOT use "|", as that character is a character SPECIFICALLY USED for when something unexpected happened.
/// Sometimes "|" is used for dropping the lock the thread holds on stdio, so don't panic when "|" is returned.
pub fn spawn_input_thread(
  screen: &ScreenData,
) -> Result<(Receiver<String>, Sender<()>), ScreenError> {
  if !screen.printer_started() {
    return Err(ScreenError::PrinterNotStarted);
  }

  let (input_sender, input_receiver) = channel();
  let (kill_sender, kill_receiver) = oneshot::channel();

  info!("Spawning input thread.");
  let _ = thread::spawn(move || {
    info!("Input thread successfully spawned.");

    while kill_receiver.try_recv().is_err() {
      let input = get_user_input();

      let _ = input_sender.send(input);
    }

    info!("Input thread killed.");
  });

  Ok((input_receiver, kill_sender))
}
