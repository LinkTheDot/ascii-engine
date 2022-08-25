use interactable_screen::objects::object_data::ObjectInformation;
use interactable_screen::screen::screen_data::*;

#[cfg(test)]
mod update_existing_objects_logic {
  use super::*;

  #[test]
  fn adding_a_new_object() {
    let mut screen = ScreenData::default();
    let new_object = get_basic_object_information();
    let expected_total_count = 1;
    let expected_existing_count = 0;

    screen.update_existing_objects(&new_object);

    let screen_object_data = screen
      .get_existing_object(&new_object.get_name().to_string())
      .unwrap();

    let total_count = screen_object_data.get_total_count();
    let existing_count = screen_object_data.get_currently_existing();

    assert_eq!(total_count, expected_total_count);
    assert_eq!(existing_count, expected_existing_count);
  }

  #[test]
  fn updating_an_existing_object() {
    let mut screen = ScreenData::default();
    let new_object = get_basic_object_information();
    let expected_total_count = 2;
    let expected_existing_count = 0;

    screen.update_existing_objects(&new_object);
    screen.update_existing_objects(&new_object);

    let screen_object_data = screen
      .get_existing_object(&new_object.get_name().to_string())
      .unwrap();

    let total_count = screen_object_data.get_total_count();
    let existing_count = screen_object_data.get_currently_existing();

    assert_eq!(total_count, expected_total_count);
    assert_eq!(existing_count, expected_existing_count);
  }
}

fn get_basic_object_information() -> ObjectInformation<'static> {
  let name = "square";
  let object_shape = "xx\nxx";
  let position = None;
  let keep_data = None;

  ObjectInformation::from(name, object_shape, position, keep_data)
}
