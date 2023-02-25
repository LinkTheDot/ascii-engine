#![cfg(test)]

use super::*;

#[cfg(test)]
mod grid_size_matches_previous_grid_logic {
  use super::*;

  // Can't use dynamic_print in tests because getting origin isn't possible.
  // Printing escape codes in a test also makes the terminal explode.
  #[test]
  fn does_match() {
    let mut printer = Printer::new(3, 3);
    let grid1 =
      Printer::create_grid_from_single_character(&"a", printer.grid_width, printer.grid_height);
    let grid2 =
      Printer::create_grid_from_single_character(&"b", printer.grid_width, printer.grid_height);

    printer.previous_grid = grid1;

    assert!(printer.grid_size_matches_previous_grid(&grid2));
  }

  #[test]
  fn does_not_match() {
    let mut printer = Printer::new(3, 3);
    let grid1 =
      Printer::create_grid_from_single_character(&"a", printer.grid_width, printer.grid_height);
    let grid2 =
      Printer::create_grid_from_single_character(&"b", printer.grid_width + 2, printer.grid_height);

    printer.previous_grid = grid1;

    assert!(!printer.grid_size_matches_previous_grid(&grid2));
  }
}

#[cfg(test)]
mod get_grid_diff_logic {
  use super::*;
  // Base grid will be
  // abcde
  // 12345
  // vwxyz
  static BASE_GRID: &str = "abcde\n12345\nvwxyz";
  static GRID_SIZES: (usize, usize) = (5, 3);

  fn get_preassigned_printer() -> Printer {
    let mut printer = Printer::new(GRID_SIZES.0, GRID_SIZES.1);
    printer.previous_grid = BASE_GRID.to_string();

    printer
  }

  /// The indices will apply to any newlines, so make
  /// sure to account for those.
  fn get_modified_base_grid(indices: Vec<usize>) -> String {
    indices
      .into_iter()
      .fold(BASE_GRID.to_string(), |mut base_grid, index| {
        base_grid.remove(index);
        base_grid.insert(index, 'l');

        base_grid
      })
  }

  #[test]
  fn one_pixel_difference() {
    // lbcde
    // 12345
    // vwxyz
    let printer = get_preassigned_printer();
    let different_grid = get_modified_base_grid(vec![0]);

    let expected_different_pixels = vec![PixelDifference {
      pixels: String::from("l"),
      index: 0,
    }];

    let different_pixels = printer.get_grid_diff(&different_grid);

    assert_eq!(expected_different_pixels, different_pixels);
  }

  #[cfg(test)]
  mod two_pixel_difference {
    use super::*;

    #[test]
    fn different_pixels_are_together() {
      // llcde
      // 12345
      // vwxyz
      let printer = get_preassigned_printer();
      let different_grid = get_modified_base_grid(vec![0, 1]);

      let expected_different_pixels = vec![PixelDifference {
        pixels: String::from("ll"),
        index: 0,
      }];

      let different_pixels = printer.get_grid_diff(&different_grid);

      assert_eq!(expected_different_pixels, different_pixels);
    }

    #[test]
    fn different_pixels_are_together_split_by_newline() {
      // abcdl
      // l2345
      // vwxyz
      let printer = get_preassigned_printer();
      let different_grid = get_modified_base_grid(vec![4, 6]);

      let expected_different_pixels = vec![PixelDifference {
        pixels: String::from("l\nl"),
        index: 4,
      }];

      let different_pixels = printer.get_grid_diff(&different_grid);

      assert_eq!(expected_different_pixels, different_pixels);
    }

    #[test]
    fn different_pixels_are_together_right_before_newline() {
      // abcll
      // 12345
      // vwxyz
      let printer = get_preassigned_printer();
      let different_grid = get_modified_base_grid(vec![3, 4]);

      let expected_different_pixels = vec![PixelDifference {
        pixels: String::from("ll"),
        index: 3,
      }];

      let different_pixels = printer.get_grid_diff(&different_grid);

      assert_eq!(expected_different_pixels, different_pixels);
    }

    #[test]
    fn different_pixels_are_together_right_after_newline() {
      // abcde
      // ll345
      // vwxyz
      let printer = get_preassigned_printer();
      let different_grid = get_modified_base_grid(vec![7, 8]);

      let expected_different_pixels = vec![PixelDifference {
        pixels: String::from("ll"),
        index: 6,
      }];

      let different_pixels = printer.get_grid_diff(&different_grid);

      assert_eq!(expected_different_pixels, different_pixels);
    }

    #[test]
    fn different_pixels_are_apart() {
      // alcde
      // 1l345
      // vwxyz
      let printer = get_preassigned_printer();
      let different_grid = get_modified_base_grid(vec![1, 8]);

      let expected_different_pixels = vec![
        PixelDifference {
          pixels: String::from("l"),
          index: 1,
        },
        PixelDifference {
          pixels: String::from("l"),
          index: 7,
        },
      ];

      let different_pixels = printer.get_grid_diff(&different_grid);

      assert_eq!(expected_different_pixels, different_pixels);
    }

    #[test]
    fn different_pixels_touch_last_index() {
      // abcde
      // 12345
      // vwxll
      let printer = get_preassigned_printer();
      let different_grid = get_modified_base_grid(vec![15, 16]);

      let expected_different_pixels = vec![PixelDifference {
        pixels: String::from("ll"),
        index: 13,
      }];

      let different_pixels = printer.get_grid_diff(&different_grid);

      assert_eq!(expected_different_pixels, different_pixels);
    }
  }

  #[cfg(test)]
  mod three_pixel_difference {
    use super::*;

    #[test]
    fn different_pixels_are_together() {
      // lllde
      // 1l345
      // vwxyz
      let printer = get_preassigned_printer();
      let different_grid = get_modified_base_grid(vec![0, 1, 2]);

      let expected_different_pixels = vec![PixelDifference {
        pixels: String::from("lll"),
        index: 0,
      }];

      let different_pixels = printer.get_grid_diff(&different_grid);

      assert_eq!(expected_different_pixels, different_pixels);
    }

    #[test]
    fn first_two_pixels_together() {
      // llcde
      // 1l345
      // vwxyz
      let printer = get_preassigned_printer();
      let different_grid = get_modified_base_grid(vec![0, 1, 8]);

      let expected_different_pixels = vec![
        PixelDifference {
          pixels: String::from("ll"),
          index: 0,
        },
        PixelDifference {
          pixels: String::from("l"),
          index: 7,
        },
      ];

      let different_pixels = printer.get_grid_diff(&different_grid);

      assert_eq!(expected_different_pixels, different_pixels);
    }

    #[test]
    fn last_two_pixels_together() {
      // alcde
      // ll345
      // vwxyz
      let printer = get_preassigned_printer();
      let different_grid = get_modified_base_grid(vec![1, 7, 8]);

      let expected_different_pixels = vec![
        PixelDifference {
          pixels: String::from("l"),
          index: 1,
        },
        PixelDifference {
          pixels: String::from("ll"),
          index: 6,
        },
      ];

      let different_pixels = printer.get_grid_diff(&different_grid);

      assert_eq!(expected_different_pixels, different_pixels);
    }

    #[test]
    fn last_two_pixels_together_split_by_newline() {
      // alcdl
      // l2345
      // vwxyz
      let printer = get_preassigned_printer();
      let different_grid = get_modified_base_grid(vec![1, 4, 6]);

      let expected_different_pixels = vec![
        PixelDifference {
          pixels: String::from("l"),
          index: 1,
        },
        PixelDifference {
          pixels: String::from("l\nl"),
          index: 4,
        },
      ];

      let different_pixels = printer.get_grid_diff(&different_grid);

      assert_eq!(expected_different_pixels, different_pixels);
    }
  }

  #[cfg(test)]
  mod four_pixel_difference {
    use super::*;

    #[test]
    fn different_pixels_are_together() {
      // lllle
      // 12345
      // vwxyz
      let printer = get_preassigned_printer();
      let different_grid = get_modified_base_grid(vec![0, 1, 2, 3]);

      let expected_different_pixels = vec![PixelDifference {
        pixels: String::from("llll"),
        index: 0,
      }];

      let different_pixels = printer.get_grid_diff(&different_grid);

      assert_eq!(expected_different_pixels, different_pixels);
    }

    #[test]
    fn split_pixel_pairs() {
      // llcde
      // ll345
      // vwxyz
      let printer = get_preassigned_printer();
      let different_grid = get_modified_base_grid(vec![0, 1, 6, 7]);

      let expected_different_pixels = vec![
        PixelDifference {
          pixels: String::from("ll"),
          index: 0,
        },
        PixelDifference {
          pixels: String::from("ll"),
          index: 5,
        },
      ];

      let different_pixels = printer.get_grid_diff(&different_grid);

      assert_eq!(expected_different_pixels, different_pixels);
    }

    #[test]
    fn all_pixels_split() {
      // lbcdl
      // 12345
      // lwxyl
      let printer = get_preassigned_printer();
      let different_grid = get_modified_base_grid(vec![0, 4, 12, 16]);

      let expected_different_pixels = vec![
        PixelDifference {
          pixels: String::from("l"),
          index: 0,
        },
        PixelDifference {
          pixels: String::from("l"),
          index: 4,
        },
        PixelDifference {
          pixels: String::from("l"),
          index: 10,
        },
        PixelDifference {
          pixels: String::from("l"),
          index: 14,
        },
      ];

      let different_pixels = printer.get_grid_diff(&different_grid);

      assert_eq!(expected_different_pixels, different_pixels);
    }
  }

  #[test]
  fn entire_grid_difference() {
    // lllll
    // lllll
    // lllll
    let printer = get_preassigned_printer();
    let different_grid =
      get_modified_base_grid(vec![0, 1, 2, 3, 4, 6, 7, 8, 9, 10, 12, 13, 14, 15, 16]);

    let expected_different_pixels = vec![PixelDifference {
      pixels: String::from("lllll\nlllll\nlllll"),
      index: 0,
    }];

    let different_pixels = printer.get_grid_diff(&different_grid);

    assert_eq!(expected_different_pixels, different_pixels);
  }
}
