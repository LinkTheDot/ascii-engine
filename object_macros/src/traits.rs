use proc_macro::TokenStream;
use quote::quote;

pub fn generate_traits(ast: &syn::DeriveInput) -> TokenStream {
  let name = &ast.ident;

  quote! {
  impl Object for #name {
    fn get_position(&self) -> &usize {
      self.object_data.get_object_position()
    }

    fn get_top_left_position(&self) -> &usize {
      self.object_data.top_left()
    }

    fn get_sprite_dimensions(&self) -> (usize, usize) {
      self.object_data.get_sprite_dimensions()
    }

    fn move_to(&mut self, new_position: (usize, usize)) -> Result<(), ObjectError> {
      let new_index = new_position.0 + (CONFIG.grid_width as usize * new_position.1);

      self.object_data.change_position(new_index)
    }

    fn move_by(&mut self, added_position: (isize, isize)) {
      //
      //
    }

    fn get_air_char(&self) -> char {
      self.object_data.get_air_char()
    }

    fn get_sprite(&self) -> &str {
      self.object_data.get_sprite()
    }

    fn change_sprite(&mut self, new_model: String) {
      self.object_data.change_sprite(new_model)
    }

    fn get_hitbox(&self) -> &Vec<(isize, isize)> {
      self.object_data.get_hitbox()
    }

    fn change_hitbox(&mut self, new_hitbox: Hitbox) -> Result<(), ascii_engine::objects::errors::ObjectError> {
      self.object_data.change_hitbox(new_hitbox)
    }

    fn get_unique_hash(&self) -> &u64 {
      self.object_data.get_unique_hash()
    }

    fn get_strata(&self) -> &Strata {
      self.object_data.get_strata()
    }

    fn change_strata(&mut self, new_strata: Strata) -> Result<(), ObjectError> {
      if !new_strata.correct_range() {
        return Err(ObjectError::IncorrectStrataRange(new_strata));
      } else {
        self.object_data.change_strata(new_strata)
      }

      Ok(())
    }
  }
    }
  .into()
}
