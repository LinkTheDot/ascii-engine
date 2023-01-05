use proc_macro::TokenStream;
use quote::quote;

pub fn generate_traits(ast: &syn::DeriveInput) -> TokenStream {
  let name = &ast.ident;

  quote! {
  trait Object {
    fn get_position(&self) -> &usize;

    fn move_to(&mut self, new_position: (usize, usize));
    fn move_by(&mut self, added_position: (isize, isize));

    fn get_sprite(&self) -> &str;
    fn change_sprite(&mut self, new_model: String);

    fn get_hitbox(&self) -> &Vec<(isize, isize)>;
    fn change_hitbox(&mut self, new_hitbox_model: Hitbox);

    fn get_unique_hash(&self) -> &u64;

    fn get_strata(&self) -> &Strata;
    fn change_strata(&mut self, new_strata: Strata);
  }

  impl Object for #name {
    fn get_position(&self) -> &usize {
      self.object_data.get_object_position()
    }

    fn move_to(&mut self, new_position: (usize, usize)) {
      // check if the new position is out of bounds.
      // this will probably require building a config in this crate
    }

    fn move_by(&mut self, added_position: (isize, isize)) {
      // check if the new position is out of bounds.
      // this will probably require building a config in this crate
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

    fn change_hitbox(&mut self, new_hitbox: Hitbox) {
      self.object_data.change_hitbox(new_hitbox)
    }

    fn get_unique_hash(&self) -> &u64 {
      self.object_data.get_unique_hash()
    }

    fn get_strata(&self) -> &Strata {
      self.object_data.get_strata()
    }

    fn change_strata(&mut self, new_strata: Strata) {
      self.object_data.change_strata(new_strata);
    }
  }
    }
  .into()
}
