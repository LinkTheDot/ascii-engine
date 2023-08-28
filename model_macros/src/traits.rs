use proc_macro::TokenStream;
use quote::quote;

pub fn generate_traits(_ast: &syn::DeriveInput) -> TokenStream {
  // let name = &ast.ident;

  quote! {
    // impl DisplayModel for #name {
    // }
  }
  .into()
}
