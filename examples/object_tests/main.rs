use ascii_engine::prelude::*;

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
  let sprite = get_square_sprite();
  let square_object_data = ObjectData::new((10, 10), sprite, Strata::Top(0));
  let square = Square {
    object_data: square_object_data,
  };

  println!("{:#?}", square);
}

fn get_square_sprite() -> Sprite {
  let hitbox = Hitbox {
    shape: "xxx\n-c-".to_string(),
    center_character: 'c',
    air_character: '-',
    center_is_hitbox: true,
  };

  let skin = Skin {
    shape: "xxx\nxcx".to_string(),
    center_character: 'c',
    center_replacement_character: 'x',
    air_character: '-',
  };

  Sprite::new(skin, hitbox).unwrap()
}
