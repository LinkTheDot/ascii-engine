use crate::general_data::coordinates::*;
use crate::objects::errors::*;
use guard::guard;

/// The sprite is data about the display and hitbox side of an object.
///
/// The sprite will contain how an object will look, where it's hitbox will be, and
/// what character in the skin of the object should be classified as "air".
#[allow(unused)]
#[derive(Debug)]
pub struct Sprite {
  skin: Skin,
  hitbox: Vec<(isize, isize)>,
}

#[derive(Debug)]
pub struct Hitbox {
  pub shape: String,
  pub center_character: char,
  pub air_character: char,
  pub center_is_hitbox: bool,
}

#[derive(Debug)]
pub struct Skin {
  pub shape: String,
  pub center_character: char,
  pub center_replacement_character: char,
  pub air_character: char,
}

impl Sprite {
  pub fn new(mut skin: Skin, hitbox: Hitbox) -> Result<Self, ObjectError> {
    let hitbox = hitbox.get_hitbox_data()?;
    skin.fix_skin();

    Ok(Self { skin, hitbox })
  }

  pub fn get_shape(&self) -> &str {
    &self.skin.shape
  }

  pub fn get_mut_shape(&mut self) -> &mut String {
    &mut self.skin.shape
  }

  pub fn get_hitbox(&self) -> &Vec<(isize, isize)> {
    &self.hitbox
  }

  pub fn change_hitbox(&mut self, new_hitbox: Hitbox) {
    self.hitbox = new_hitbox.get_hitbox_data().unwrap();
  }
}

impl Hitbox {
  pub fn new(
    shape: String,
    center_character: char,
    air_character: char,
    center_is_hitbox: bool,
  ) -> Self {
    Self {
      shape,
      center_character,
      air_character,
      center_is_hitbox,
    }
  }

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
        let current_index = (
          current_iteration % hitbox_width,
          current_iteration / hitbox_width,
        );

        if current_hitbox_char != self.air_character
          || self.center_is_hitbox && current_hitbox_char == self.center_character
        {
          let coordinates = current_index.subtract(hitbox_center_coordinates);

          hitbox_bounds.push(coordinates);
        }

        hitbox_bounds
      },
    ))
  }
}

impl Skin {
  pub fn new(
    shape: String,
    center_character: char,
    center_replacement_character: char,
    air_character: char,
  ) -> Self {
    Self {
      shape,
      center_character,
      center_replacement_character,
      air_character,
    }
  }

  fn fix_skin(&mut self) {
    self.shape = self.shape.replace(
      &self.center_character.to_string(),
      &self.center_replacement_character.to_string(),
    );
  }
}

/// Returns
/// (width, height).
///
/// An error is returned when the hitbox isn't a rectangle.
fn valid_rectangle_check(object: &str) -> Result<(usize, usize), ObjectError> {
  let rows: Vec<&str> = object.split('\n').collect();
  let object_width = if !rows.is_empty() {
    rows[0].chars().count()
  } else {
    return Err(ObjectError::EmptyHitboxString);
  };

  let rows_have_same_lengths = rows.iter().all(|row| row.chars().count() == object_width);

  if rows_have_same_lengths {
    Ok((object_width, rows.len()))
  } else {
    Err(ObjectError::NonRectangularShape)
  }
}
