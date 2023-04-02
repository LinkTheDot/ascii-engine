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
        let new_index = new_position.0 + (CONFIG.grid_width as usize * new_position.1);

        self.model_data.change_position(new_index);

        self.model_data.check_collisions_against_all_models()
      }

      fn move_by(&mut self, added_position: (isize, isize)) -> Vec<ModelData> {
        let true_width = CONFIG.grid_width as isize + 1;

        let new_index =
          added_position.0 + (true_width * added_position.1) + self.get_top_left_position() as isize;

        self.model_data.change_position(new_index as usize);

        self.model_data.check_collisions_against_all_models()
      }

      fn get_air_char(&self) -> char {
        self.model_data.get_air_char()
      }

      fn get_sprite(&self) -> String {
        self.model_data.get_sprite()
      }

      fn change_sprite(&mut self, new_model: String) {
        self.model_data.change_sprite(new_model)
      }

      fn get_unique_hash(&self) -> u64 {
        self.model_data.get_unique_hash()
      }

      fn get_strata(&self) -> Strata {
        self.model_data.get_strata()
      }

      fn change_strata(&mut self, new_strata: Strata) -> Result<(), ModelError> {
        if new_strata.correct_range() {
          self.change_strata(new_strata)?;
          self.model_data.fix_model_strata()?;
        } else {
          return Err(ModelError::IncorrectStrataRange(new_strata));
        }

        Ok(())
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
