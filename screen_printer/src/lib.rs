//! Screen Printer is a rust crate that will allow you to build and print arrays of
//! data into a grid format.
//!
//! The purpose of this crate is to make it easier to print a grid that's stored in
//! an array.
//!
//! ## Examples
//!
//! #### Creating and Printing a Grid
//!
//! ```rust
//! use screen_printer::printer::*;
//!
//! const WIDTH: usize = 5;
//! const HEIGHT: usize = 4;
//!
//! fn main() {
//!   let character_list = vec![
//!     "a", "b", "c", "d", "e", //
//!     "f", "g", "h", "i", "j", //
//!     "k", "l", "m", "n", "o", //
//!     "p", "q", "r", "s", "t",
//!   ];
//!
//!   let grid =
//!     Printer::create_grid_from_full_character_list(&character_list, WIDTH, HEIGHT).unwrap();
//!
//!   print!("{}", "\n".repeat(HEIGHT * 2)); // This gives the grid space for the first print
//!   Printer::print_over_previous_grid(grid, HEIGHT);
//! }
//! ```
//!
//! #### Using the dynamic print method
//! ```rust,no_run
//! use screen_printer::printer::*;
//!
//! const WIDTH: usize = 3;
//! const HEIGHT: usize = 3;
//!
//! fn main() {
//!   let mut printer = Printer::new(WIDTH, HEIGHT);
//!
//!   // get the grid data
//!   let grid_1_rows = vec!["abc", "123", "xyz",];
//!   // create the grid
//!   let grid_1 = Printer::create_grid_from_multiple_rows(&grid_1_rows).unwrap();
//!   // print the first grid, using said grid as a
//!   // basis for all future grids
//!   printer.dynamic_print(grid_1).unwrap();
//!
//!   // get the grid data
//!   let grid_2_rows = vec!["abc", "789", "xyz",];
//!   // create the grid
//!   let grid_2 = Printer::create_grid_from_multiple_rows(&grid_1_rows).unwrap();
//!   // print only the differences in the grid from the previous one
//!   printer.dynamic_print(grid_2).unwrap();
//! }
//! ```
//! This will result in
//! ```bash,no_run
//! abc
//! 123
//! xyz
//! ```
//! Into
//! ```bash,no_run
//! abc
//! 789 < only line that was actually printed
//! xyz
//! ```

/// Methods for efficient grid printing
pub mod dynamic_printer;
/// Creation and basic printing for grids
pub mod printer;
