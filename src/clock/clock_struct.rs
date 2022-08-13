use crossbeam_channel::*;
use std::{error::Error, thread, time::Duration};

///in milliseconds
pub const TICK_DURATION: u64 = 24;

pub struct ScreenClock {
  tick_update_receiver: Receiver<()>,
  tick_update_sender: Sender<()>,
}

impl ScreenClock {
  pub fn new() -> Result<Self, Box<dyn Error>> {
    let (sender, receiver) = bounded(1);

    Ok(ScreenClock {
      tick_update_receiver: receiver,
      tick_update_sender: sender,
    })
  }

  pub fn spawn_clock_thread(&self) -> Result<(), Box<dyn Error>> {
    let drain_receiver = self.tick_update_receiver.clone();
    let sender = self.tick_update_sender.clone();

    thread::spawn(move || loop {
      sender.try_send(()).unwrap_or(());

      if !sender.is_empty() {
        drain_receiver.drain_channel();
      }

      thread::sleep(Duration::from_millis(TICK_DURATION));
    });

    Ok(())
  }

  pub fn wait_for_x_ticks(&self, x: u16) {
    for _ in 0..x {
      self.tick_update_receiver.wait_for_tick();

      // goes fast enough to skip over ticks so it needs to stop
      // for a millisecond
      thread::sleep(Duration::from_millis(1));
    }
  }
}

trait ReceiverMethods {
  fn drain_channel(&self);
  fn wait_for_tick(&self);
}

impl<T> ReceiverMethods for Receiver<T> {
  fn drain_channel(&self) {
    while self.try_recv().is_ok() {}
  }

  fn wait_for_tick(&self) {
    while self.is_empty() {}
  }
}
