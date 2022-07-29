pub struct Square {
  // define what a square is
  name: String,
  position: Coordinates,
}

impl Square {
  pub fn new() -> Self {
    Square {
      name: "Square".to_string(),
    }
  }
}

// probably implement some sort of formatter that'll construct it into what is needed
