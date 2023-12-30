#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://photino.github.io/zino-docs-zh/assets/zino-logo.png")]
#![doc(html_logo_url = "https://photino.github.io/zino-docs-zh/assets/zino-logo.svg")]
#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod decode_row;
mod model;
mod model_accessor;
mod model_hooks;
mod parser;
mod schema;

#[doc = include_str!("../docs/schema.md")]
#[proc_macro_derive(Schema, attributes(schema))]
pub fn derive_schema(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let output = schema::parse_token_stream(input);
    TokenStream::from(output)
}

#[doc = include_str!("../docs/model_accessor.md")]
#[proc_macro_derive(ModelAccessor, attributes(schema))]
pub fn derive_model_accessor(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let output = model_accessor::parse_token_stream(input);
    TokenStream::from(output)
}

#[doc = include_str!("../docs/decode_row.md")]
#[proc_macro_derive(DecodeRow, attributes(schema))]
pub fn derive_decode_row(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let output = decode_row::parse_token_stream(input);
    TokenStream::from(output)
}

#[doc = include_str!("../docs/model_hooks.md")]
#[proc_macro_derive(ModelHooks)]
pub fn derive_model_hooks(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let output = model_hooks::parse_token_stream(input);
    TokenStream::from(output)
}

#[doc = include_str!("../docs/model.md")]
#[proc_macro_derive(Model)]
pub fn derive_model(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let output = model::parse_token_stream(input);
    TokenStream::from(output)
}
