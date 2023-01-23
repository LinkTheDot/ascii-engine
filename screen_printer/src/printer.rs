pub use crate::dynamic_printer::*;
use std::cmp::Ordering;
use std::fmt;
use std::{io, io::Write};

/// These are the possible ways the program can fail.
///
/// Each error will contain 'ErrorData' which holds the
/// expected and outcome results in the event of the error.
#[derive(Debug)]
pub enum PrintingError {
  RowTooLong(LengthErrorData),
  RowTooShort(LengthErrorData),

  /// In the context of creating a grid from a full list of characters
  TooManyCharacters(LengthErrorData),
  /// In the context of creating a grid from a full list of characters
  TooLittleCharacters(LengthErrorData),

  RowsDontMatchLengths,

  InvalidGridInput(LengthErrorData),
  CursorError(String),
}

impl fmt::Display for PrintingError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

/// The namespace for all methods used to create and print a grid
///
/// Also used to store data for the dynamic printing method
#[derive(Debug)]
pub struct Printer {
  pub previous_grid: String,

  /// Creation will be on first print
  pub origin_position: (usize, usize),

  pub grid_height: usize,
  pub grid_width: usize,
}

/// Error data for when incorrect sizes are detected in a method
#[derive(Debug)]
pub struct LengthErrorData {
  pub expected_length: usize,
  pub got_length: usize,
}

impl LengthErrorData {
  /// Creates a new LengthErrorData for the expected length and actual length
  #[allow(clippy::new_without_default)]
  pub fn new(expected_length: usize, got_length: usize) -> Self {
    Self {
      expected_length,
      got_length,
    }
  }
}

impl Printer {
  /// Creates a new Printer, this is not needed for most methods since Printer
  /// is only there for the namespace
  ///
  /// However you will need to create a Printer for using the [`dynamic_print()`](crate::printer::Printer::dynamic_print()) method.
  #[allow(clippy::new_without_default)]
  pub fn new(grid_width: usize, grid_height: usize) -> Self {
    Self {
      previous_grid: String::new(),
      origin_position: (0, 0),
      grid_width,
      grid_height,
    }
  }

  /// Creates a grid of the given size with the given character.
  ///
  /// It's recommended that the passed in item is only 1 character long.
  ///
  /// # Example
  /// ```
  /// use screen_printer::printer::*;
  ///
  /// let character = "a";
  /// let expected_grid = "aaa\naaa\naaa";
  ///
  /// let grid = Printer::create_grid_from_single_character(&character, 3, 3);
  ///
  /// assert_eq!(expected_grid, grid);
  /// ```
  pub fn create_grid_from_single_character<T>(character: &T, width: usize, height: usize) -> String
  where
    T: fmt::Display,
  {
    let row = Self::get_row_of_character(character, width);

    Self::create_grid_from_single_row(&row, height)
  }

  /// Creates a grid of the given height with the given row
  ///
  /// # Example
  /// ```
  /// use screen_printer::printer::*;
  ///
  /// let row = "abcd";
  /// let expected_grid = "abcd\nabcd\nabcd";
  ///
  /// let grid = Printer::create_grid_from_single_row(&row, 3);
  ///
  /// assert_eq!(expected_grid, grid);
  /// ```
  pub fn create_grid_from_single_row<T>(row: &T, height: usize) -> String
  where
    T: fmt::Display,
  {
    let rows = (0..height).fold(Vec::new(), |mut rows, _| {
      rows.push(&row);

      rows
    });

    Printer::create_grid_from_multiple_rows(&rows).unwrap()
  }

  /// Creates a grid of the given size with the given list of characters
  ///
  /// An error is returned when the grid size doesn't match the amount of characters given
  ///
  /// # Example
  /// ```
  /// use screen_printer::printer::*;
  ///
  /// let characters = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i"];
  /// let expected_grid = "abc\ndef\nghi";
  ///
  /// let grid = Printer::create_grid_from_full_character_list(&characters, 3, 3).unwrap();
  ///
  /// assert_eq!(expected_grid, grid);
  /// ```
  pub fn create_grid_from_full_character_list<T>(
    characters: &Vec<T>,
    width: usize,
    height: usize,
  ) -> Result<String, PrintingError>
  where
    T: fmt::Display,
  {
    let grid_size = width * height;

    match characters.len().cmp(&grid_size) {
      Ordering::Less => Err(PrintingError::TooLittleCharacters(LengthErrorData::new(
        characters.len(),
        grid_size,
      ))),
      Ordering::Greater => Err(PrintingError::TooManyCharacters(LengthErrorData::new(
        characters.len(),
        width * height,
      ))),
      Ordering::Equal => Ok(create_grid_from_characters(characters, width)),
    }
  }

  /// Creates a grid of the given rows
  ///
  /// An error is returned if any of the rows isn't the same length as the first
  ///
  /// # Example
  /// ```
  /// use screen_printer::printer::*;
  ///
  /// let rows = vec![
  ///   "abc",
  ///   "def",
  ///   "ghi",
  /// ];
  ///
  /// let expected_grid = "abc\ndef\nghi";
  ///
  /// let grid = Printer::create_grid_from_multiple_rows(&rows).unwrap();
  ///
  /// assert_eq!(expected_grid, grid);
  /// ```
  pub fn create_grid_from_multiple_rows<T>(rows: &[T]) -> Result<String, PrintingError>
  where
    T: fmt::Display,
  {
    let rows: Vec<String> = rows.iter().map(|row| format!("{}", row)).collect();
    let width = rows[0].chars().count();

    if Self::rows_have_same_lengths(&rows, width) {
      Ok(rows.join("\n"))
    } else {
      Err(PrintingError::RowsDontMatchLengths)
    }
  }

  /// Moves the cursor up by the given height and prints the given grid.
  ///
  /// This is for printing over the previously printed grid.
  /// It's recommended to add some whitespace before your first print so the grid
  /// doesn't print into anything that was printed before this method was called.
  ///
  /// # Example
  /// ```
  /// use screen_printer::printer::*;
  ///
  /// let height = 10;
  /// let width = 10;
  /// let grid = Printer::create_grid_from_single_character(&"a", width, height);
  ///
  /// print!("{}", "\n".repeat(height + 5)); // add some space for the grid
  /// Printer::print_over_previous_grid(grid, height);
  /// ```
  pub fn print_over_previous_grid(grid: String, height: usize) {
    print!("\x1b[{};A", height - 1);
    print!("\r{}", grid);
    let _ = io::stdout().flush();
  }

  /// Creates a row of the given width with the given character.
  ///
  /// # Example
  /// ```
  /// use screen_printer::printer::*;
  ///
  /// let width = 3;
  /// let row = Printer::get_row_of_character(&"a", width);
  ///
  /// assert_eq!(row, "aaa".to_string());
  /// ```
  pub fn get_row_of_character<T>(character: &T, width: usize) -> String
  where
    T: fmt::Display,
  {
    (0..width).fold(String::new(), |row, _| format!("{}{}", row, character))
  }

  /// Returns true of all given rows have the same amount of characters as the expected input.
  fn rows_have_same_lengths(rows: &[String], expected_width: usize) -> bool {
    rows.iter().all(|row| row.chars().count() == expected_width)
  }
}

/// Creates a grid of the given width out of the given 1D array of characters.
fn create_grid_from_characters<T>(characters: &[T], width: usize) -> String
where
  T: fmt::Display,
{
  characters
    .chunks(width)
    .map(|row| {
      row.iter().fold(String::new(), |mut row, character| {
        row.push_str(format!("{}", character).as_str());

        row
      })
    })
    .collect::<Vec<String>>()
    .join("\n")
}
