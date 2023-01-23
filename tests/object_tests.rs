use ascii_engine::prelude::*;

const SHAPE: &str = "x-x\nxcx\nx-x";
const CENTER_CHAR: char = 'c';
const CENTER_REPLACEMENT_CHAR: char = '-';
const AIR_CHAR: char = '-';

#[cfg(test)]
mod skin_logic {
  use super::*;

  #[test]
  fn center_character_index_check() {
    let skin = get_skin();

    let expected_center_character_index = 4;

    assert_eq!(
      expected_center_character_index,
      skin.get_center_character_index()
    );
  }

  #[test]
  fn no_center_on_shape() {
    let skin_result = Skin::new("xxx\nxxx", CENTER_CHAR, CENTER_REPLACEMENT_CHAR, AIR_CHAR);

    assert!(skin_result.is_err());
  }
}

#[cfg(test)]
mod sprite_logic {
  use super::*;

  #[test]
  fn center_character_index_check() {
    let sprite = get_sprite(true);

    let expected_index = 4;

    let center_skin_index = sprite.get_center_character_index();

    assert_eq!(center_skin_index, expected_index);
  }

  #[test]
  fn get_shape_logic() {
    let sprite = get_sprite(true);

    let expected_skin = SHAPE.replace(
      &CENTER_CHAR.to_string(),
      &CENTER_REPLACEMENT_CHAR.to_string(),
    );

    let sprite_skin = sprite.get_shape();

    assert_eq!(sprite_skin, expected_skin);
  }

  #[test]
  fn get_mut_shape_logic() {
    let mut sprite = get_sprite(true);

    let mut expected_skin = SHAPE.replace(
      &CENTER_CHAR.to_string(),
      &CENTER_REPLACEMENT_CHAR.to_string(),
    );

    let sprite_skin = sprite.get_mut_shape();

    assert_eq!(sprite_skin, &mut expected_skin);
  }

  #[test]
  fn get_hitbox_logic() {
    let sprite = get_sprite(true);

    // xxx
    //  x  < this x is the center character
    let expected_hitbox_data = vec![(-1, -1), (0, -1), (1, -1), (0, 0)];

    let hitbox_data = sprite.get_hitbox();

    assert_eq!(hitbox_data, &expected_hitbox_data);
  }

  #[test]
  fn change_hitbox_valid_new_hitbox() {
    let mut sprite = get_sprite(true);
    let new_hitbox = Hitbox::new("xxxxx\n--c--", 'c', '-', false);

    let expected_hitbox_data = vec![(-2, -1), (-1, -1), (0, -1), (1, -1), (2, -1)];

    sprite.change_hitbox(new_hitbox).unwrap();

    let new_hitbox_data = sprite.get_hitbox();

    assert_eq!(new_hitbox_data, &expected_hitbox_data);
  }

  #[test]
  /// Has no center character.
  fn change_hitbox_invalid_new_hitbox() {
    let mut sprite = get_sprite(true);
    let new_hitbox = Hitbox::new("xxxxx\n-----", 'c', '-', false);

    let changed_hitbox_result = sprite.change_hitbox(new_hitbox);

    assert!(changed_hitbox_result.is_err());
  }

  #[test]
  fn get_air_character_logic() {
    let sprite = get_sprite(true);

    let expected_air_character = AIR_CHAR;

    let air_character = sprite.air_character();

    assert_eq!(air_character, expected_air_character);
  }

  fn get_sprite(center_is_hitbox: bool) -> Sprite {
    let skin = get_skin();
    let hitbox = get_hitbox(center_is_hitbox);

    match Sprite::new(skin, hitbox) {
      Ok(sprite) => sprite,
      Err(error) => panic!("An error has occurred while getting the sprite: '{error:?}'"),
    }
  }
}

fn get_skin() -> Skin {
  match Skin::new(SHAPE, CENTER_CHAR, CENTER_REPLACEMENT_CHAR, AIR_CHAR) {
    Ok(skin) => skin,
    Err(error) => panic!("An error has occurred while getting the skin: '{error:?}'"),
  }
}

fn get_hitbox(center_is_hitbox: bool) -> Hitbox {
  let shape = "xyz\n-c-";
  let center_character = 'c';
  let air_character = '-';

  Hitbox::new(shape, center_character, air_character, center_is_hitbox)
}
