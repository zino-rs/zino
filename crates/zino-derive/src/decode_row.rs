use super::parser;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

/// Integer types
const UNSIGNED_INTEGER_TYPES: [&str; 5] = ["u64", "u32", "u16", "u8", "usize"];

/// Parses the token stream for the `DecodeRow` trait derivation.
pub(super) fn parse_token_stream(input: DeriveInput) -> TokenStream {
    // Model name
    let name = input.ident;

    // Parsing struct attributes
    let mut auto_coalesce = false;
    for attr in input.attrs.iter() {
        for (key, _value) in parser::parse_schema_attr(attr).into_iter() {
            if key == "auto_coalesce" {
                auto_coalesce = true;
            }
        }
    }

    // Parsing field attributes
    let mut decode_model_fields = Vec::new();
    'outer: for field in parser::parse_struct_fields(input.data) {
        let type_name = parser::get_type_name(&field.ty);
        if let Some(ident) = field.ident {
            let name = ident.to_string().trim_start_matches("r#").to_owned();
            for attr in field.attrs.iter() {
                let arguments = parser::parse_schema_attr(attr);
                for (key, _value) in arguments.into_iter() {
                    match key.as_str() {
                        "ignore" | "write_only" => continue 'outer,
                        _ => (),
                    }
                }
            }
            if type_name == "Uuid" {
                decode_model_fields.push(quote! {
                    model.#ident = zino_orm::decode_uuid(row, #name)?;
                });
            } else if type_name == "Option<Uuid>" {
                decode_model_fields.push(quote! {
                    let value = zino_orm::decode_uuid(row, #name)?;
                    model.#ident = (!value.is_nil()).then_some(value);
                });
            } else if type_name == "Decimal" {
                decode_model_fields.push(quote! {
                    model.#ident = zino_orm::decode_decimal(row, #name)?;
                });
            } else if type_name == "Map" {
                let field_decoder = if auto_coalesce {
                    quote! {
                        if let Some(JsonValue::Object(map)) = zino_orm::decode_optional(row, #name)? {
                            model.#ident = map;
                        }
                    }
                } else {
                    quote! {
                        if let JsonValue::Object(map) = zino_orm::decode(row, #name)? {
                            model.#ident = map;
                        }
                    }
                };
                decode_model_fields.push(field_decoder);
            } else if parser::check_option_type(&type_name) {
                decode_model_fields.push(quote! {
                    model.#ident = zino_orm::decode_optional(row, #name)?;
                });
            } else if parser::check_vec_type(&type_name) {
                decode_model_fields.push(quote! {
                    model.#ident = zino_orm::decode_array(row, #name)?;
                });
            } else if UNSIGNED_INTEGER_TYPES.contains(&type_name.as_str()) {
                let integer_type_ident = format_ident!("{}", type_name.replace('u', "i"));
                let field_decoder = if auto_coalesce {
                    quote! {
                        if let Some(value) = zino_orm::decode_optional::<#integer_type_ident>(row, #name)? {
                            model.#ident = value.try_into()?;
                        }
                    }
                } else {
                    quote! {
                        let value = zino_orm::decode::<#integer_type_ident>(row, #name)?;
                        model.#ident = value.try_into()?;
                    }
                };
                decode_model_fields.push(field_decoder);
            } else {
                let field_decoder = if auto_coalesce {
                    quote! {
                        if let Some(value) = zino_orm::decode_optional(row, #name)? {
                            model.#ident = value;
                        }
                    }
                } else {
                    quote! {
                        model.#ident = zino_orm::decode(row, #name)?;
                    }
                };
                decode_model_fields.push(field_decoder);
            }
        }
    }
    quote! {
        impl zino_orm::DecodeRow<zino_orm::DatabaseRow> for #name {
            type Error = zino_core::error::Error;

            fn decode_row(row: &zino_orm::DatabaseRow) -> Result<Self, Self::Error> {
                use zino_core::{extension::JsonValueExt, JsonValue};

                let mut model = Self::default();
                #(#decode_model_fields)*
                Ok(model)
            }
        }
    }
}
