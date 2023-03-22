use proc_macro::TokenStream;
use quote::quote;

pub fn generate_traits(ast: &syn::DeriveInput) -> TokenStream {
  let name = &ast.ident;

  quote! {
    impl Model for #name {
      fn get_position(&self) -> usize {
        let model_data_guard = self.model_data.lock().unwrap();

        model_data_guard.get_model_position()
      }

      fn get_top_left_position(&self) -> usize {
        let model_data_guard = self.model_data.lock().unwrap();

        *model_data_guard.top_left()
      }

      fn get_sprite_dimensions(&self) -> (usize, usize) {
        let model_data_guard = self.model_data.lock().unwrap();

        model_data_guard.get_sprite_dimensions()
      }

      fn move_to(&mut self, new_position: (usize, usize)) -> Vec<std::sync::Arc<std::sync::Mutex<ModelData>>> {
        let mut model_data_guard = self.model_data.lock().unwrap();

        let new_index = new_position.0 + (CONFIG.grid_width as usize * new_position.1);

        model_data_guard.change_position(new_index);

        model_data_guard.check_collisions_against_all_models()
      }

      fn move_by(&mut self, added_position: (isize, isize)) -> Vec<std::sync::Arc<std::sync::Mutex<ModelData>>> {
        let true_width = CONFIG.grid_width as isize + 1;

        let new_index =
          added_position.0 + (true_width * added_position.1) + self.get_top_left_position() as isize;

        let mut model_data_guard = self.model_data.lock().unwrap();

        model_data_guard.change_position(new_index as usize);

        model_data_guard.check_collisions_against_all_models()
      }

      fn get_air_char(&self) -> char {
        let model_data_guard = self.model_data.lock().unwrap();

        model_data_guard.get_air_char()
      }

      fn get_sprite(&self) -> String {
        let model_data_guard = self.model_data.lock().unwrap();

        model_data_guard.get_sprite().to_string()
      }

      fn change_sprite(&mut self, new_model: String) {
        let mut model_data_guard = self.model_data.lock().unwrap();

        model_data_guard.change_sprite(new_model)
      }

      // fn get_hitbox(&self) -> Vec<(isize, isize)> {
      //   let model_data_guard = self.model_data.lock().unwrap();
      //
      //   model_data_guard.get_hitbox().clone()
      // }
      //
      // fn change_hitbox(
      //   &mut self,
      //   new_hitbox: Hitbox,
      // ) -> Result<(), ascii_engine::models::errors::ModelError> {
      //   let mut model_data_guard = self.model_data.lock().unwrap();
      //
      //   model_data_guard.change_hitbox(new_hitbox)
      // }

      fn get_unique_hash(&self) -> u64 {
        let model_data_guard = self.model_data.lock().unwrap();

        *model_data_guard.get_unique_hash()
      }

      fn get_strata(&self) -> Strata {
        let model_data_guard = self.model_data.lock().unwrap();

        *model_data_guard.get_strata()
      }

      fn change_strata(&mut self, new_strata: Strata) -> Result<(), ModelError> {
        let mut model_data_guard = self.model_data.lock().unwrap();

        if new_strata.correct_range() {
          model_data_guard.change_strata(new_strata);
          model_data_guard.fix_model_strata()?;

        } else {
          return Err(ModelError::IncorrectStrataRange(new_strata));
        }

        Ok(())
      }

      fn get_name(&self) -> String {
        let model_data_guard = self.model_data.lock().unwrap();

        model_data_guard.get_name().to_owned()
      }

      fn change_name(&mut self, new_name: String) {
        let mut model_data_guard = self.model_data.lock().unwrap();

        model_data_guard.change_name(new_name);
      }

      fn get_model_data(&self) -> std::sync::Arc<std::sync::Mutex<ModelData>> {
        std::sync::Arc::clone(&self.model_data)
      }

      fn assign_model_list(&mut self, model_list: std::sync::Arc<std::sync::RwLock<Models>>) {
        let mut model_guard = self.model_data.lock().unwrap();

        model_guard.assign_model_list(model_list);
      }
    }
  }
  .into()
}
