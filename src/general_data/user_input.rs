use oneshot::Sender;
use std::io;
use std::io::{Read, Write};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

pub fn get_user_input() -> String {
  let stdin = 0;

  let mut termios = Termios::from_fd(stdin).unwrap();

  termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
  tcsetattr(stdin, TCSANOW, &termios).unwrap();

  let stdout = io::stdout();
  let mut reader = io::stdin();
  let mut buffer = vec![0; 1]; // read exactly one byte

  stdout.lock().flush().unwrap();
  reader.read_exact(&mut buffer).unwrap();

  tcsetattr(stdin, TCSANOW, &termios).unwrap(); // reset the stdin to original termios data

  String::from_utf8(buffer.to_vec()).unwrap()
}

pub fn spawn_input_thread() -> (Receiver<String>, Sender<()>) {
  let (input_sender, input_receiver) = channel();
  let (end_sender, end_receiver) = oneshot::channel();

  let _ = thread::spawn(move || {
    while end_receiver.try_recv().is_err() {
      let input = get_user_input();

      let _ = input_sender.send(input);
    }

    println!("thread killed");
  });

  (input_receiver, end_sender)
}
