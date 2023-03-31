use log::warn;
use oneshot::Sender;
use std::io;
use std::io::{Read, Write};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;
use std::time::Instant;
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

#[allow(unused)]
use log::{debug, info};

fn get_user_input() -> String {
  let stdin = 0;

  let mut termios = Termios::from_fd(stdin).unwrap();

  termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
  tcsetattr(stdin, TCSANOW, &termios).unwrap();

  let stdout = io::stdout();
  let mut reader = io::stdin();
  let mut buffer = vec![0; 1]; // read exactly one byte

  stdout.lock().flush().unwrap();
  if reader.read_exact(&mut buffer).is_err() {
    warn!("Failed to read from stdin");

    tcsetattr(stdin, TCSANOW, &termios).unwrap(); // reset the stdin to original termios data

    return String::from("|");
  }

  tcsetattr(stdin, TCSANOW, &termios).unwrap(); // reset the stdin to original termios data

  // Checks if the buffer contained "^["
  // This would likely mean the printer just tried getting it's origin position.
  // We need to stop locking stdin for ~200ms to let the printer set it's origin position.
  //
  // This is, of course, assuming the user doesn't just panic the moment the printer returns any error.
  // if buffer == [27] {
  //   warn!("User Input thread Spawned before printer could set origin. Please print once before attempting to spawn user input thread");
  //
  //   let mut cleared_buffer = vec![];
  //
  //   let now = Instant::now();
  //   let timeout = Duration::from_millis(200);
  //
  //   while now.elapsed() < timeout {
  //     reader.read_exact(&mut cleared_buffer).unwrap();
  //   }
  //
  //   // while buf[0] != delimiter && now.elapsed().unwrap() < timeout {
  //   //   if stdin.read(&mut buf)? > 0 {
  //   //     read_chars.push(buf[0]);
  //   //   }
  //   // }
  //
  //   // std::thread::sleep(std::time::Duration::from_millis(200));
  //
  //   return String::from("|");
  // }

  String::from_utf8(buffer.to_vec()).unwrap()
}

/// Spawns a thread that will request an input from the user at every moment.
///
/// The Receiver for the user's input, and a sender to kill the input thread is returned.
///
/// DO NOT use "|", as that character is a character SPECIFICALLY USED for when something unexpected happened.
/// Sometimes "|" is used for dropping the lock the thread holds on stdio, so don't panic when "|" is returned.
pub fn spawn_input_thread() -> (Receiver<String>, Sender<()>) {
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

  (input_receiver, kill_sender)
}
