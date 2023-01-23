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
  ///
  /// The example will be creating
  /// # Skin
  /// ```bash,no_run
  /// before   after
  ///  xxx  |   xxx
  ///  xcx  |   xxx
  /// ```
  /// # Hitbox
  /// ```bash,no_run
  /// before   after
  ///  xxx  |   xxx
  ///  -c-  |   -x-
  /// ```
  ///
  /// # Hitbox Creation
  /// ```
  /// use ascii_engine::prelude::*;
  ///
  /// let hitbox = Hitbox {
  ///   shape: "xxx\n-c-".to_string(),
  ///   center_character: 'c',
  ///   air_character: '-',
  ///   center_is_hitbox: true,
  /// };
  /// ```
  ///
  /// # Skin Creation
  /// ```
  /// use ascii_engine::prelude::*;
  ///
  /// let skin = Skin {
  ///   shape: "xxx\nxcx".to_string(),
  ///   center_character: 'c',
  ///   center_replacement_character: 'x',
  ///   air_character: '-',
  /// };
  /// ```
  pub fn new(mut skin: Skin, hitbox: Hitbox) -> Result<Self, ObjectError> {
    let hitbox = hitbox.get_hitbox_data()?;
    skin.fix_skin();

    Ok(Self { skin, hitbox })
  }

  pub fn get_center_character_index(&self) -> &usize {
    &self.skin.center_character_index
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
  ///
  /// # Hitbox Creation
  /// ```
  /// use ascii_engine::prelude::*;
  ///
  /// let hitbox = Hitbox {
  ///   shape: "xxx\n-c-".to_string(),
  ///   center_character: 'c',
  ///   air_character: '-',
  ///   center_is_hitbox: true,
  /// };
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

        if current_hitbox_char != self.air_character
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
  ///
  /// # Skin Creation
  /// ```
  /// use ascii_engine::prelude::*;
  ///
  /// let skin = Skin {
  ///   shape: "xxx\nxcx".to_string(),
  ///   center_character: 'c',
  ///   center_replacement_character: 'x',
  ///   air_character: '-',
  /// };
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
