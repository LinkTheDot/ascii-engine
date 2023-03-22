use crate::traits::generate_traits;
use proc_macro::TokenStream;

mod traits;

#[proc_macro_derive(Model)]
pub fn derive_model(input: TokenStream) -> TokenStream {
  let model = syn::parse(input).unwrap();

  generate_traits(&model)
}
