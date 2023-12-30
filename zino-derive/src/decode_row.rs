use super::parser;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

/// Integer types
const UNSIGNED_INTEGER_TYPES: [&str; 5] = ["u64", "u32", "u16", "u8", "usize"];

/// Parses the token stream for the `DecodeRow` trait derivation.
pub(super) fn parse_token_stream(input: DeriveInput) -> TokenStream {
    // Parsing field attributes
    let name = input.ident;
    let mut decode_model_fields = Vec::new();
    for field in parser::parse_struct_fields(input.data) {
        let type_name = parser::get_type_name(&field.ty);
        if type_name.is_empty() {
            continue;
        }
        if let Some(ident) = field.ident {
            let name = ident.to_string();
            let mut ignore = false;
            'inner: for attr in field.attrs.iter() {
                let arguments = parser::parse_schema_attr(attr);
                for (key, _value) in arguments.iter() {
                    if key == "ignore" || key == "write_only" {
                        ignore = true;
                        break 'inner;
                    }
                }
            }
            if ignore {
                continue;
            }
            if type_name == "Map" {
                decode_model_fields.push(quote! {
                    if let JsonValue::Object(map) = orm::decode(row, #name)? {
                        model.#ident = map;
                    }
                });
            } else if parser::check_vec_type(&type_name) {
                decode_model_fields.push(quote! {
                    model.#ident = orm::decode_array(row, #name)?;
                });
            } else if UNSIGNED_INTEGER_TYPES.contains(&type_name.as_str()) {
                let integer_type_ident = format_ident!("{}", type_name.replace('u', "i"));
                decode_model_fields.push(quote! {
                    let value = orm::decode::<#integer_type_ident>(row, #name)?;
                    model.#ident = value.try_into()?;
                });
            } else {
                decode_model_fields.push(quote! {
                    model.#ident = orm::decode(row, #name)?;
                });
            }
        }
    }
    quote! {
        impl zino_core::model::DecodeRow<zino_core::orm::DatabaseRow> for #name {
            type Error = zino_core::error::Error;

            fn decode_row(row: &zino_core::orm::DatabaseRow) -> Result<Self, Self::Error> {
                use zino_core::{extension::JsonValueExt, orm, JsonValue};

                let mut model = Self::default();
                #(#decode_model_fields)*
                Ok(model)
            }
        }
    }
}
