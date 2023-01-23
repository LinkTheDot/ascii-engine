use crate::printer::*;
use guard::guard;
use log::info;
use std::{io, io::Write};
use termion::cursor::DetectCursorPos;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;

mod tests;

/// A PixelDifference will contain a single instance of
/// a contigious string of different pixels within the grid.
///
/// The index in which the first different pixel occupies
/// is also stored.
#[derive(Debug, PartialEq)]
struct PixelDifference {
  pixels: String,
  index: usize,
}

impl PixelDifference {
  /// Creates a new pixel difference with the given pixel and index
  fn new(pixel: String, index: usize) -> Self {
    Self {
      pixels: pixel,
      index,
    }
  }

  /// Checks if the current index is bordering the grid, and adds a newline
  /// to the PixelDifference if it is.
  fn check_newlines(&mut self, current_index: usize, grid_width: usize, grid_size: usize) {
    if current_index % grid_width == 0 && current_index != grid_size - 1 {
      self.pixels.push('\n');
    }
  }
}

pub trait DynamicPrinter {
  /// The dynamic_print() method will efficiently print differences of passed
  /// in grids.
  ///
  /// # Example
  /// ```rust,no_run
  /// use screen_printer::printer::*;
  ///
  /// let width = 3;
  /// let height = 3;
  /// let mut printer = Printer::new(width, height);
  ///
  /// let grid1 = "abc\n123\nxyz".to_string();
  /// let grid2 = "abc\n123\nasd".to_string();
  ///
  /// // The first print will remember where to print any future grids.
  /// printer.dynamic_print(grid1).unwrap();
  /// // Should look like
  /// // abc
  /// // 123
  /// // xyz
  ///
  /// // The second print will compare the new grid, and
  /// // print only the differences from the previously printed grid.
  /// printer.dynamic_print(grid2).unwrap();
  /// // Should look like
  /// // abc
  /// // 123
  /// // asd < only part that got printed
  /// ```
  ///
  /// The way the printer remembers where to print a grid is based on where the cursor was
  /// upon first print.
  /// The cursor's location marks the bottom right of the grid.
  /// If the x or y of origin are to go out of bounds, said axis will be set to 0.
  ///
  /// An error is returned when the cursor can't be read or the passed in grid does not match the expected size.
  fn dynamic_print(&mut self, grid: String) -> Result<(), PrintingError>;

  /// Resets all data for the printer and assigns it a new size.
  fn reset(&mut self, new_width: Option<usize>, new_height: Option<usize>);

  /// Replaces every character in the grid with whitespace
  fn clear_grid(&mut self) -> Result<(), PrintingError>;
}

impl DynamicPrinter for Printer {
  fn dynamic_print(&mut self, grid: String) -> Result<(), PrintingError> {
    if self.grid_size_matches_previous_grid(&grid) {
      let different_pixels = self.get_grid_diff(&grid);

      let printable_difference = self.get_printable_diff(different_pixels);

      print!("{}", printable_difference);
    } else if self.previous_grid.is_empty() {
      self.set_origin()?;
      self.move_to_origin();

      print!("{}", grid);
    } else {
      let previous_grid_size = self.previous_grid.chars().count();
      let new_grid_size = grid.chars().count();

      return Err(PrintingError::InvalidGridInput(LengthErrorData::new(
        previous_grid_size,
        new_grid_size,
      )));
    }

    let _ = io::stdout().flush();
    self.previous_grid = grid;

    Ok(())
  }

  fn reset(&mut self, new_width: Option<usize>, new_height: Option<usize>) {
    self.previous_grid = String::new();

    if let Some(width) = new_width {
      self.grid_width = width;
    }

    if let Some(height) = new_height {
      self.grid_height = height;
    }
  }

  fn clear_grid(&mut self) -> Result<(), PrintingError> {
    let empty_grid =
      Self::create_grid_from_single_character(&" ", self.grid_width, self.grid_height);

    self.dynamic_print(empty_grid)?;

    Ok(())
  }
}

trait DynamicPrinterMethods {
  /// Returns true if both the previous and new grid have the same
  /// length per row and amount of rows
  fn grid_size_matches_previous_grid(&self, grid: &str) -> bool;

  /// Gets a list of the pixel indexes that were different from the previous grid
  fn get_grid_diff(&self, grid: &str) -> Vec<PixelDifference>;

  /// Moves the cursor to the assigned origin.
  fn move_to_origin(&self);

  /// Assigns origin with the current cursor position and the size of the grid.
  ///
  /// If the cursor is at (10, 10) and the grid is 5x5 then origin will be set to (5, 5).
  /// If the grid is larger than the cursor's current position, origin will be assigned 0 in it's place.
  /// This means that if the cursor is at (10, 10), and the grid is 20x5, origin will be assigned to (0, 5).
  fn set_origin(&mut self) -> Result<(), PrintingError>;

  /// Returns the current position of the cursor
  fn get_current_cursor_position() -> Result<(usize, usize), PrintingError>;

  fn get_printable_diff(&mut self, pixel_differences: Vec<PixelDifference>) -> String;
}

impl DynamicPrinterMethods for Printer {
  fn grid_size_matches_previous_grid(&self, grid: &str) -> bool {
    let new_grid_height = grid.rsplit('\n').count();
    let old_grid_height = self.previous_grid.rsplit('\n').count();

    let mut new_grid_split = grid.rsplit('\n');

    guard!(let Some(old_grid_row) = self.previous_grid.rsplit('\n').next() else { return false; });
    let old_grid_row_width = old_grid_row.chars().count();

    new_grid_height == old_grid_height
      && new_grid_split.all(|new_row| new_row.chars().count() == old_grid_row_width)
  }

  fn get_grid_diff(&self, grid: &str) -> Vec<PixelDifference> {
    let old_grid = self.previous_grid.replace('\n', "");
    let new_grid = grid.replace('\n', "");

    let grid_size = new_grid.chars().count();
    let grid_iter = old_grid.chars().zip(new_grid.chars());

    let mut last_edited_pixel_index = 0;

    grid_iter.enumerate().fold(
      Vec::new(),
      |mut different_pixels, (pixel_index, (old_pixel, new_pixel))| {
        if new_pixel != old_pixel {
          if let Some(latest_pixel) = different_pixels.get_mut_top() {
            if last_edited_pixel_index == pixel_index - 1 || latest_pixel.index == pixel_index - 1 {
              latest_pixel.check_newlines(pixel_index, self.grid_width, grid_size);

              latest_pixel.pixels.push_str(&new_pixel.to_string());

              last_edited_pixel_index = pixel_index;
              return different_pixels;
            }
          }

          let pixel_difference = PixelDifference::new(new_pixel.to_string(), pixel_index);

          different_pixels.push(pixel_difference);
        }

        different_pixels
      },
    )
  }

  fn move_to_origin(&self) {
    print!(
      "\x1B[{};{}H",
      self.origin_position.1, self.origin_position.0
    );
  }

  fn set_origin(&mut self) -> Result<(), PrintingError> {
    let (mut x, mut y) = Self::get_current_cursor_position()?;

    if y > self.grid_height {
      y -= self.grid_height;
    } else {
      y = 0;
    }

    if x > self.grid_width {
      x -= self.grid_width;
    } else {
      x = 0;
    }

    self.origin_position.0 = x;
    self.origin_position.1 = y;

    Ok(())
  }

  fn get_current_cursor_position() -> Result<(usize, usize), PrintingError> {
    let mut stdout = MouseTerminal::from(io::stdout().into_raw_mode().unwrap());
    let cursor_position = stdout.cursor_pos();

    info!("Origin Position: {cursor_position:?}");

    match cursor_position {
      Ok(position) => Ok((position.0 as usize, position.1 as usize)),
      Err(error) => Err(PrintingError::CursorError(error.to_string())),
    }
  }

  fn get_printable_diff(&mut self, pixel_differences: Vec<PixelDifference>) -> String {
    pixel_differences
      .iter()
      .fold(String::new(), |mut printable_diff, pixel_difference| {
        let (mut x, mut y) = pixel_difference
          .index
          .index_as_coordinates(&self.grid_width);

        x += self.origin_position.0;
        y += self.origin_position.1;

        let cursor_movement = format!("\x1B[{};{}H", y, x);
        let movement_with_pixels = format!("{}{}", cursor_movement, pixel_difference.pixels);

        printable_diff.push_str(&movement_with_pixels);

        printable_diff
      })
  }
}

trait VecMethods<T> {
  /// Gets a mutable reference to the top most item in a vector
  fn get_mut_top(&mut self) -> Option<&mut T>;
}

impl<T> VecMethods<T> for std::vec::Vec<T> {
  fn get_mut_top(&mut self) -> Option<&mut T> {
    let size = self.len();

    if size != 0 {
      self.get_mut(size - 1)
    } else {
      None
    }
  }
}

trait UsizeMethods {
  /// Converts an index into coordinates
  fn index_as_coordinates(&self, grid_width: &Self) -> (usize, usize);
}

impl UsizeMethods for usize {
  fn index_as_coordinates(&self, grid_width: &Self) -> (usize, usize) {
    (self % grid_width, self / grid_width)
  }
}
