#![allow(unused)]

use interactable_screen::clock::clock_struct::*;
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn wait_for_x_ticks_logic() {
  let clock = ScreenClock::default();
  let x_ticks = 10;

  let run_test_x_times = 50;
  let mut failed_runs = 0;

  let expected_duration = TICK_DURATION * x_ticks as u64;
  let expected_failed_runs = 0;

  for _ in 0..run_test_x_times {
    clock.spawn_clock_thread();

    // gives it a split second to let the clock do it's thing
    thread::sleep(Duration::from_millis(1));

    let mut now = Instant::now();

    clock.wait_for_x_ticks(x_ticks);

    let mut elapsed_time = now.elapsed().as_millis();

    if !elapsed_time == expected_duration as u128 + 1
      || !elapsed_time == expected_duration as u128 - 1
      || !elapsed_time == expected_duration as u128
    {
      println!("elapsed_time is - {elapsed_time}\nexpected - {expected_duration} +- 1..2");

      failed_runs += 1;
    }
  }

  assert_eq!(failed_runs, expected_failed_runs);
}
