use ascii_engine::prelude::*;

const SHAPE: &str = "x-x\nxcx\nx-x";
const CENTER_CHAR: char = 'c';
const CENTER_REPLACEMENT_CHAR: char = '-';
const AIR_CHAR: char = '-';
const OBJECT_NAME: &str = "rectangle";

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
    let sprite = get_sprite();

    let expected_index = 4;

    let center_skin_index = sprite.get_center_character_index();

    assert_eq!(center_skin_index, expected_index);
  }

  #[test]
  fn get_shape_logic() {
    let sprite = get_sprite();

    let expected_skin = SHAPE.replace(
      &CENTER_CHAR.to_string(),
      &CENTER_REPLACEMENT_CHAR.to_string(),
    );

    let sprite_skin = sprite.get_shape();

    assert_eq!(sprite_skin, expected_skin);
  }

  #[test]
  fn get_mut_shape_logic() {
    let mut sprite = get_sprite();

    let mut expected_skin = SHAPE.replace(
      &CENTER_CHAR.to_string(),
      &CENTER_REPLACEMENT_CHAR.to_string(),
    );

    let sprite_skin = sprite.get_mut_shape();

    assert_eq!(sprite_skin, &mut expected_skin);
  }

  #[test]
  #[ignore]
  fn get_hitbox_logic() {
    // let sprite = get_sprite(true);
    //
    // // xxx
    // //  x  < this x is the center character
    // let expected_hitbox_data = vec![(-1, -1), (0, -1), (1, -1), (0, 0)];
    //
    // let hitbox_data = sprite.get_hitbox();
    //
    // assert_eq!(hitbox_data, &expected_hitbox_data);
  }

  #[test]
  #[ignore]
  fn change_hitbox_valid_new_hitbox() {
    // let mut sprite = get_sprite(true);
    // let new_hitbox = HitboxCreationData::new("xxxxx\n--c--", 'c', '-', false);
    //
    // let expected_hitbox_data = vec![(-2, -1), (-1, -1), (0, -1), (1, -1), (2, -1)];
    //
    // sprite.change_hitbox(new_hitbox).unwrap();
    //
    // let new_hitbox_data = sprite.get_hitbox();
    //
    // assert_eq!(new_hitbox_data, &expected_hitbox_data);
  }

  #[test]
  #[ignore]
  /// Has no center character.
  fn change_hitbox_invalid_new_hitbox() {
    // let mut sprite = get_sprite(true);
    // let new_hitbox = HitboxCreationData::new("xxxxx\n-----", 'c', '-', false);
    //
    // let changed_hitbox_result = sprite.change_hitbox(new_hitbox);
    //
    // assert!(changed_hitbox_result.is_err());
  }

  #[test]
  fn get_air_character_logic() {
    let sprite = get_sprite();

    let expected_air_character = AIR_CHAR;

    let air_character = sprite.air_character();

    assert_eq!(air_character, expected_air_character);
  }
}

#[cfg(test)]
mod object_data_logic {
  use super::*;

  #[cfg(test)]
  mod strata_correct_range_logic {
    use super::*;

    #[test]
    fn valid_range() {
      let strata = Strata(5);

      assert!(strata.correct_range());
    }

    #[test]
    /// Strata is higher than 100
    fn invalid_range() {
      let strata = Strata(101);

      assert!(!strata.correct_range());
    }
  }

  #[test]
  #[ignore]
  fn creation_logic_valid_strata() {
    // let position = (10, 10);
    // let sprite = get_sprite(true);
    // let strata = Strata(5);
    //
    // let object_data = ObjectData::new(position, sprite, strata);
    //
    // assert!(object_data.is_ok());
  }

  #[test]
  #[ignore]
  /// Strata is higher than 100
  fn creation_logic_invalid_strata() {
    // let position = (10, 10);
    // let sprite = get_sprite(true);
    // let strata = Strata(101);
    //
    // let object_data = ObjectData::new(position, sprite, strata);
    //
    // assert!(object_data.is_err());
  }

  #[cfg(test)]
  mod get_logic {
    use super::*;

    #[test]
    #[ignore]
    fn get_top_left_index() {
      // let (x, y) = (10, 10);
      // let object_data = get_object_data((x, y), true);
      //
      // let expected_top_left_index = ((CONFIG.grid_width + 1) as usize * (y - 1)) + (x - 1);
      //
      // assert_eq!(object_data.top_left(), &expected_top_left_index);
    }

    #[test]
    #[ignore]
    fn get_new_top_left() {
      // let (x, y) = (10, 10);
      // let object_data = get_object_data((x, y), true);
      //
      // let expected_new_top_left_index = ((CONFIG.grid_width + 1) as usize * (y - 1)) + (x - 1);
      //
      // assert_eq!(
      //   object_data.get_top_left_index_of_skin(),
      //   expected_new_top_left_index
      // );
    }

    #[test]
    #[ignore]
    fn get_sprite_skin_dimensions() {
      let (x, y) = (10, 10);
      let object_data = get_object_data((x, y));

      let expected_sprite_skin_dimensions = (3, 3);

      assert_eq!(
        object_data.get_sprite_dimensions(),
        expected_sprite_skin_dimensions
      );
    }

    #[test]
    fn get_air_character() {
      let (x, y) = (10, 10);
      let object_data = get_object_data((x, y));

      let expected_air_character = '-';

      assert_eq!(object_data.get_air_char(), expected_air_character);
    }

    #[test]
    fn get_unique_hash() {
      let (x, y) = (10, 10);
      let object_data = get_object_data((x, y));

      assert!(*object_data.get_unique_hash() != 0);
    }

    #[test]
    fn get_center_frame_index() {
      let (x, y) = (10, 10);
      let object_data = get_object_data((x, y));

      let expected_current_center_frame_index = ((CONFIG.grid_width + 1) as usize * y) + x;

      assert_eq!(
        object_data.get_object_position(),
        expected_current_center_frame_index
      );
    }

    #[test]
    fn get_sprite_skin() {
      let (x, y) = (10, 10);
      let object_data = get_object_data((x, y));

      let expected_sprite_skin = "x-x\nx-x\nx-x";

      assert_eq!(object_data.get_sprite(), expected_sprite_skin);
    }

    #[test]
    #[ignore]
    fn get_hitbox() {
      // let (x, y) = (10, 10);
      // let object_data = get_object_data((x, y));
      //
      // let expected_hitbox = vec![(-1, -1), (0, -1), (1, -1), (0, 0)];
      //
      // assert_eq!(object_data.get_hitbox(), &expected_hitbox);
    }

    #[test]
    fn get_strata() {
      let (x, y) = (10, 10);
      let object_data = get_object_data((x, y));

      let expected_strata = Strata(0);

      assert_eq!(object_data.get_strata(), &expected_strata);
    }
  }

  #[cfg(test)]
  mod change_logic {
    use super::*;

    #[test]
    #[ignore]
    fn change_position() {
      // let (x, y) = (10, 10);
      // let mut object_data = get_object_data((x, y));
      // let (new_x, new_y) = (x + 5, y + 5);
      //
      // let expected_new_position = ((CONFIG.grid_width + 1) as usize * new_y) + new_x;
      // let expected_new_top_left = calculate_skin_top_left_index((new_x, new_y));
      //
      // object_data.change_position(((CONFIG.grid_width + 1) as usize * new_y) + new_x);
      //
      // let object_position = object_data.get_object_position();
      // let top_left_position = *object_data.top_left();
      //
      // assert_eq!(object_position, expected_new_position);
      // assert_eq!(top_left_position, expected_new_top_left);
    }

    #[test]
    #[ignore]
    fn change_position_out_of_bounds_right() {
      // let (x, y) = (CONFIG.grid_width as usize, 15);
      // let mut object_data = get_object_data((x, y));
      // let (new_x, new_y) = (x - 1, y + 1);
      //
      // let expected_result = Err(ObjectError::OutOfBounds(Direction::Right));
      //
      // let change_position_result =
      //   object_data.change_position(((CONFIG.grid_width) as usize * new_y) + new_x);
      //
      // assert_eq!(change_position_result, expected_result);
    }

    #[test]
    #[ignore]
    fn change_position_out_of_bounds_down() {
      // let (x, y) = (15, CONFIG.grid_width as usize);
      // let mut object_data = get_object_data((x, y), true);
      // let (new_x, new_y) = (x + 1, y + 1);
      //
      // let expected_result = Err(ObjectError::OutOfBounds(Direction::Down));
      //
      // let change_position_result =
      //   object_data.change_position(((CONFIG.grid_width + 1) as usize * new_y) + new_x);
      //
      // assert_eq!(change_position_result, expected_result);
    }

    #[test]
    fn change_strata_logic() {
      let (x, y) = (10, 10);
      let mut object_data = get_object_data((x, y));

      let expected_new_strata = Strata(5);

      object_data.change_strata(Strata(5));

      assert_eq!(object_data.get_strata(), &expected_new_strata);
    }

    #[test]
    #[ignore]
    fn change_hitbox_valid_new_hitbox() {
      // let mut object_data = get_object_data((5, 5), true);
      // let new_hitbox = HitboxCreationData::new("xxxxx\n--c--", 'c', '-', false);
      //
      // let expected_hitbox_data = vec![(-2, -1), (-1, -1), (0, -1), (1, -1), (2, -1)];
      //
      // object_data.change_hitbox(new_hitbox).unwrap();
      //
      // let new_hitbox_data = object_data.get_hitbox();
      //
      // assert_eq!(new_hitbox_data, &expected_hitbox_data);
    }

    #[test]
    #[ignore]
    /// Has no center character.
    fn change_hitbox_invalid_new_hitbox() {
      // let mut object_data = get_object_data((5, 5), true);
      // let new_hitbox = HitboxCreationData::new("xxxxx\n-----", 'c', '-', false);
      //
      // let changed_hitbox_result = object_data.change_hitbox(new_hitbox);
      //
      // assert!(changed_hitbox_result.is_err());
    }

    #[test]
    fn change_sprite() {
      let mut object_data = get_object_data((5, 5));
      let new_sprite = "xxx\nx-x";

      let expected_sprite = new_sprite;

      object_data.change_sprite(new_sprite.to_owned());

      let changed_sprite = object_data.get_sprite();

      assert_eq!(changed_sprite, expected_sprite);
    }

    // fn calculate_skin_top_left_index(position: (usize, usize)) -> usize {
    //   let object_data = get_object_data(position);
    //   let sprite = get_sprite();
    //   let object_position = object_data.get_object_position();
    //
    //   let relative_coordinates = get_0_0_relative_to_center(&sprite);
    //
    //   let true_width = CONFIG.grid_width as isize + 1;
    //
    //   (relative_coordinates.0 + object_position as isize + (true_width * relative_coordinates.1))
    //     as usize
    // }
    //
    // fn get_0_0_relative_to_center(sprite: &Sprite) -> (isize, isize) {
    //   let sprite_rows: Vec<&str> = sprite.get_shape().split('\n').collect();
    //   let sprite_width = sprite_rows[0].chars().count() as isize;
    //
    //   let skin_center_index = sprite.get_center_character_index() as isize;
    //   let skin_center_coordinates = (
    //     skin_center_index % sprite_width,
    //     skin_center_index / sprite_width,
    //   );
    //
    //   (-skin_center_coordinates.0, -skin_center_coordinates.1)
    // }
  }
}

fn get_object_data(object_position: (usize, usize)) -> ObjectData {
  let sprite = get_sprite();
  let hitbox = get_hitbox();
  let strata = Strata(0);

  ObjectData::new(
    object_position,
    sprite,
    hitbox,
    strata,
    OBJECT_NAME.to_string(),
  )
  .unwrap()
}

fn get_sprite() -> Sprite {
  let skin = get_skin();

  Sprite::new(skin).unwrap()
}

fn get_skin() -> Skin {
  Skin::new(SHAPE, CENTER_CHAR, CENTER_REPLACEMENT_CHAR, AIR_CHAR).unwrap()
}

fn get_hitbox() -> HitboxCreationData {
  let shape = "xxx\n-c-";

  HitboxCreationData::new(shape, 'c')
}
