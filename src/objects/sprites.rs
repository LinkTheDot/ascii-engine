use crate::general_data::coordinates::*;
use crate::objects::errors::*;
use guard::guard;

#[allow(unused)]
use log::debug;

/// The Sprite is data about the display and hitbox side of an object.
///
/// The Sprite will contain how an object will look, where it's Hitbox will be, and
/// what character in the skin of the object should be classified as "air".
#[derive(Debug)]
pub struct Sprite {
  skin: Skin,
  hitbox: Vec<(isize, isize)>,
}

/// The Hitbox is how the screen will determine object interactions.
///
/// Creating a hitbox involves getting the shape of the hitbox, and
/// designating a character to the center and air characters in the shape.
///
/// If you want the center character to also be apart of the hitbox, a bool
/// is stored for such a thing.
///
/// Any character that isn't air or the center will be classified apart of the hitbox.
///
/// The hitbox will be the physical bounds in relation to that of the Skin.
/// When comparing both the skin and hitbox, the designated center positions in the
/// hitbox and skin shapes will determine the placement of the hitbox in relation to the
/// skin.
#[derive(Debug)]
pub struct Hitbox {
  pub shape: String,
  pub center_character: char,
  pub air_character: char,
  pub center_is_hitbox: bool,
}

/// The Skin is how an object will appear on the screen.
///
/// When creating a skin's shape, center and air characters will need to be designated.
/// The center character will be replaced with the 'center_replacement_character' field when
/// building the shape of the Skin.
#[derive(Debug)]
pub struct Skin {
  pub shape: String,
  pub center_character: char,
  pub center_replacement_character: char,
  pub air_character: char,
  /// Doesn't count new lines
  center_character_index: usize,
}

impl Sprite {
  // OBJECT CREATION IS SUBJECT TO CHANGE
  /// Creates a new Sprite with the given Skin and Hitbox.
  pub fn new(mut skin: Skin, hitbox: Hitbox) -> Result<Self, ObjectError> {
    let hitbox = hitbox.get_hitbox_data()?;
    skin.fix_skin();

    Ok(Self { skin, hitbox })
  }

  pub fn get_center_character_index(&self) -> usize {
    self.skin.center_character_index
  }

  /// Returns a reference to the skin's shape
  pub fn get_shape(&self) -> &str {
    &self.skin.shape
  }

  /// Returns a mutable reference to the skin's shape
  pub fn get_mut_shape(&mut self) -> &mut String {
    &mut self.skin.shape
  }

  /// Returns a reference to the relative points of the hitbox to
  /// the designated center point of the object's skin.
  pub fn get_hitbox(&self) -> &Vec<(isize, isize)> {
    &self.hitbox
  }

  /// Replaces the object's hitbox with a new one
  pub fn change_hitbox(&mut self, new_hitbox: Hitbox) -> Result<(), ObjectError> {
    match new_hitbox.get_hitbox_data() {
      Ok(hitbox_data) => self.hitbox = hitbox_data,
      Err(error) => return Err(error),
    }

    Ok(())
  }

  pub fn air_character(&self) -> char {
    self.skin.air_character
  }
}

impl Hitbox {
  // OBJECT CREATION IS SUBJECT TO CHANGE
  /// Creation of a Hitbox
  ///
  /// # Hitbox
  /// ```bash,no_run
  /// before   after
  ///  xxx  |   xxx
  ///  -c-  |   -x-
  /// ```
  pub fn new(
    shape: &str,
    center_character: char,
    air_character: char,
    center_is_hitbox: bool,
  ) -> Self {
    Self {
      shape: shape.to_string(),
      center_character,
      air_character,
      center_is_hitbox,
    }
  }

  /// Converts the given data into a list of relative points from the center.
  ///
  /// Returns an error when an invalid hitbox is passed in, or when there's no
  /// valid center character in the shape of the hitbox.
  fn get_hitbox_data(self) -> Result<Vec<(isize, isize)>, ObjectError> {
    let hitbox_width = valid_rectangle_check(&self.shape)?.0;
    let hitbox = &self.shape.split('\n').collect::<String>();
    let hitbox_center_index = hitbox
      .chars()
      .position(|pixel| pixel == self.center_character);

    guard!( let Some(hitbox_center_index) = hitbox_center_index else { return Err(ObjectError::NoCenter) });

    let hitbox_center_coordinates = (
      hitbox_center_index % hitbox_width,
      hitbox_center_index / hitbox_width,
    );

    Ok(hitbox.chars().enumerate().fold(
      Vec::new(),
      |mut hitbox_bounds, (current_iteration, current_hitbox_char)| {
        let current_character_coordinates = (
          current_iteration % hitbox_width,
          current_iteration / hitbox_width,
        );

        if current_hitbox_char != self.air_character && current_hitbox_char != self.center_character
          || self.center_is_hitbox && current_hitbox_char == self.center_character
        {
          let coordinates = current_character_coordinates.subtract(hitbox_center_coordinates);

          hitbox_bounds.push(coordinates);
        }

        hitbox_bounds
      },
    ))
  }
}

impl Skin {
  // OBJECT CREATION IS SUBJECT TO CHANGE
  /// Creation of a skin.
  ///
  /// # Skin
  /// ```bash,no_run
  /// before   after
  ///  xxx  |   xxx
  ///  xcx  |   xxx
  /// ```
  pub fn new(
    shape: &str,
    center_character: char,
    center_replacement_character: char,
    air_character: char,
  ) -> Result<Self, ObjectError> {
    let cleaned_shape = shape.replace('\n', "");
    let center_character_index = cleaned_shape
      .chars()
      .position(|pixel| pixel == center_character);

    match center_character_index {
      None => Err(ObjectError::NoCenter),
      Some(center_character_index) => Ok(Self {
        shape: shape.to_string(),
        center_character,
        center_replacement_character,
        air_character,
        center_character_index,
      }),
    }
  }

  /// Replaces the center character in the skin's shape with the given
  /// replacement character.
  fn fix_skin(&mut self) {
    self.shape = self.shape.replace(
      &self.center_character.to_string(),
      &self.center_replacement_character.to_string(),
    );
  }

  pub fn get_center_character_index(&self) -> usize {
    self.center_character_index
  }
}

/// Returns
/// (width, height).
///
/// An error is returned when the hitbox isn't a rectangle.
fn valid_rectangle_check(object: &str) -> Result<(usize, usize), ObjectError> {
  if object.chars().count() == 0 {
    return Err(ObjectError::EmptyHitboxString);
  }

  let rows: Vec<&str> = object.split('\n').collect();
  let object_width = rows[0].chars().count();

  let rows_have_same_lengths = rows.iter().all(|row| row.chars().count() == object_width);

  if rows_have_same_lengths {
    Ok((object_width, rows.len()))
  } else {
    Err(ObjectError::NonRectangularShape)
  }
}

#[cfg(test)]
mod skin_logic {
  use super::*;

  #[test]
  fn fix_skin_logic() {
    let mut skin = Skin::new("x-x\nxcx\nx-x", 'c', '-', '-').unwrap();

    let expected_shape = "x-x\nx-x\nx-x";

    skin.fix_skin();

    assert_eq!(skin.shape, expected_shape)
  }
}

#[cfg(test)]
mod valid_rectangle_check_logic {
  use super::*;

  #[test]
  fn valid_rectangle() {
    let rectangle = "xxx\nxxx\nxxx";

    let expected_dimensions = Ok((3, 3));

    let rectangle_dimensions = valid_rectangle_check(rectangle);

    assert_eq!(rectangle_dimensions, expected_dimensions);
  }

  #[test]
  fn invalid_rectangle() {
    let shape = "xx\nxxx\nx\nxxxxxx";

    let expected_error = Err(ObjectError::NonRectangularShape);

    let returned_data = valid_rectangle_check(shape);

    assert_eq!(returned_data, expected_error);
  }

  #[test]
  fn empty_string_passed_in() {
    let empty_string = "";

    let expected_error = Err(ObjectError::EmptyHitboxString);

    let returned_data = valid_rectangle_check(empty_string);

    assert_eq!(returned_data, expected_error);
  }
}

#[cfg(test)]
mod get_hitbox_data_logic {
  use super::*;

  #[test]
  fn valid_data_center_is_hitbox() {
    let hitbox = get_hitbox_data(true);

    // xxx
    //  x  < this x is the center character
    let expected_hitbox_data = Ok(vec![(-1, -1), (0, -1), (1, -1), (0, 0)]);

    let hitbox_data = hitbox.get_hitbox_data();

    assert_eq!(hitbox_data, expected_hitbox_data);
  }

  #[test]
  fn valid_data_center_is_not_hitbox() {
    let hitbox = get_hitbox_data(false);

    // xxx
    //  c  < this center character is not apart of the hitbox
    let expected_hitbox_data = Ok(vec![(-1, -1), (0, -1), (1, -1)]);

    let hitbox_data = hitbox.get_hitbox_data();

    assert_eq!(hitbox_data, expected_hitbox_data);
  }

  #[test]
  fn invalid_shape() {
    let mut hitbox = get_hitbox_data(true);
    hitbox.shape = "a-s-d-qwf-e-ff\n\n\nwe-gwe-w-vwea\nasd\n".to_string();

    let expected_error = Err(ObjectError::NonRectangularShape);

    let hitbox_data = hitbox.get_hitbox_data();

    assert_eq!(hitbox_data, expected_error);
  }

  #[test]
  fn no_center_character() {
    let mut hitbox = get_hitbox_data(true);
    hitbox.shape = "".to_string();

    let expected_error = Err(ObjectError::EmptyHitboxString);

    let hitbox_data = hitbox.get_hitbox_data();

    assert_eq!(hitbox_data, expected_error);
  }

  fn get_hitbox_data(center_is_hitbox: bool) -> Hitbox {
    let shape = "xyz\n-c-";
    let center_character = 'c';
    let air_character = '-';

    Hitbox::new(shape, center_character, air_character, center_is_hitbox)
  }
}
