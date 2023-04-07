use proc_macro::TokenStream;
use quote::quote;

pub fn generate_traits(ast: &syn::DeriveInput) -> TokenStream {
  let name = &ast.ident;

  quote! {
    impl DisplayModel for #name {
      fn get_position(&self) -> usize {
        self.model_data.get_model_position()
      }

      fn get_top_left_position(&self) -> usize {
        self.model_data.top_left()
      }

      fn get_sprite_dimensions(&self) -> (usize, usize) {
        self.model_data.get_sprite_dimensions()
      }

      fn move_to(&mut self, new_position: (usize, usize)) -> Vec<ModelData> {
        self.model_data.move_to(new_position)
      }

      fn move_by(&mut self, added_position: (isize, isize)) -> Vec<ModelData> {
        self.model_data.move_by(added_position)
      }

      fn move_to_collision_check(&self, new_position: (usize, usize)) -> Vec<ModelData> {
        self.model_data.move_to_collision_check(new_position)
      }

      fn move_by_collision_check(&self, added_position: (isize, isize)) -> Vec<ModelData> {
        self.model_data.move_by_collision_check(added_position)
      }

      fn get_air_char(&self) -> char {
        self.model_data.get_air_char()
      }

      fn get_sprite(&self) -> String {
        self.model_data.get_sprite()
      }

      fn get_unique_hash(&self) -> u64 {
        self.model_data.get_unique_hash()
      }

      fn get_strata(&self) -> Strata {
        self.model_data.get_strata()
      }

      fn change_strata(&mut self, new_strata: Strata) -> Result<(), ModelError> {
        self.model_data.change_strata(new_strata)
      }

      fn get_name(&self) -> String {
        self.model_data.get_name()
      }

      fn change_name(&mut self, new_name: String) {
        self.model_data.change_name(new_name);
      }

      fn get_model_data(&self) -> ModelData {
        self.model_data.clone()
      }
    }
  }
  .into()
}
