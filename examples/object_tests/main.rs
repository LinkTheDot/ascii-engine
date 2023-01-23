use ascii_engine::general_data::user_input::spawn_input_thread;
use ascii_engine::prelude::*;
use log::info;
use std::sync::{Arc, Mutex};
// use std::{thread, time::Duration};

//skin
//xxx
//xcx
//
//hitbox
//xxx
//-x-

#[derive(Debug, Object)]
struct Square {
  object_data: Arc<Mutex<ObjectData>>,
}

fn main() {
  let mut screen = ScreenData::new().unwrap();

  let (mut current_x, mut current_y) = (5, 5);

  let sprite = get_square_sprite();
  let square_object_data = ObjectData::new((current_x, current_y), sprite, Strata(0)).unwrap();
  let mut square = Square {
    object_data: Arc::new(Mutex::new(square_object_data)),
  };

  let sprite = get_square_sprite();
  let square_object_data = ObjectData::new((10, 10), sprite, Strata(1)).unwrap();
  let square2 = Square {
    object_data: Arc::new(Mutex::new(square_object_data)),
  };

  info!("{:#?}", square);

  screen.add_object(&square).unwrap();
  screen.add_object(&square2).unwrap();

  screen.print_screen().unwrap();
  let (user_input, kill_sender) = spawn_input_thread();

  for input in user_input {
    match input.trim() {
      "w" => current_y -= 1,
      "a" => current_x -= 1,
      "s" => current_y += 1,
      "d" => current_x += 1,
      "q" => {
        kill_sender.send(()).unwrap();

        break;
      }
      _ => continue,
    }

    info!("Position: ({}, {})", current_x, current_y);
    square.move_to((current_x, current_y)).unwrap();
    screen.print_screen().unwrap();

    info!("current_object_data: \n{:#?}", square);
    screen.wait_for_x_ticks(1);
  }
}

fn get_square_sprite() -> Sprite {
  let hitbox = Hitbox::new("xxx\n-c-", 'c', '-', true);
  let skin = Skin::new("xxxxx\nx-c-x\nxxxxx", 'c', '-', '-').unwrap();
  // "--x--\n-xxx-\nxxxxx\nx-c-x"

  Sprite::new(skin, hitbox).unwrap()
}
