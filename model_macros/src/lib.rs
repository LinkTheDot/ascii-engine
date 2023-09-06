use proc_macro::TokenStream;

mod traits;

#[proc_macro_derive(DisplayModel)]
pub fn derive_model(input: TokenStream) -> TokenStream {
  let x = syn::parse(input).unwrap();

  traits::generate_displaymodel_traits(&x)
}
