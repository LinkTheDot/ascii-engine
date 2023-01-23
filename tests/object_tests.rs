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
    let skin = Skin::new(SHAPE, CENTER_CHAR, CENTER_REPLACEMENT_CHAR, AIR_CHAR).unwrap();

    let expected_center_character_index = 5;

    assert_eq!(
      expected_center_character_index,
      skin.get_center_character_index()
    );
  }
}
