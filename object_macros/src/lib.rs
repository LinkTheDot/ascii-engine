Use crate::traits::generate_traits;
use proc_macro::TokenStream;

mod traits;

#[proc_macro_derive(Object)]
pub fn derive_object(input: TokenStream) -> TokenStream {
  let object = syn::parse(input).unwrap();

  generate_traits(&object)
}
