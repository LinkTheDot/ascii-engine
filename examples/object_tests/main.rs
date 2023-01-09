use ascii_engine::prelude::*;
use log::info;
use std::{thread, time::Duration};

//skin
//xxx
//xcx
//
//hitbox
//xxx
//-x-

#[derive(Debug, Object)]
struct Square {
  object_data: ObjectData,
}

fn main() {
  let mut screen = ScreenData::new().unwrap();

  let sprite = get_square_sprite();
  let square_object_data = ObjectData::new((10, 10), sprite, Strata(0)).unwrap();
  let mut square = Square {
    object_data: square_object_data,
  };

  info!("{:#?}", square);

  screen.print_screen();
  thread::sleep(Duration::from_secs(3));

  screen.add_object(&mut square).unwrap();

  screen.print_screen();
  thread::sleep(Duration::from_secs(3));
}

fn get_square_sprite() -> Sprite {
  let hitbox = Hitbox::new("xxx\n-c-", 'c', '-', true);
  let skin = Skin::new("--x--\n-xxx-\nxxxxx\nx-c-x", 'c', 'x', '-').unwrap();

  Sprite::new(skin, hitbox).unwrap()
}
