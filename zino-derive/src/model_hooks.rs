use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

/// Parses the token stream for the `ModelHooks` trait derivation.
pub(super) fn parse_token_stream(input: DeriveInput) -> TokenStream {
    // Model name
    let name = input.ident;

    quote! {
        impl zino_core::model::ModelHooks for #name {
            type Data = ();
            type Extension = ();
        }
    }
}
