use super::parser;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

/// Parses the token stream for the `ModelHooks` trait derivation.
pub(super) fn parse_token_stream(input: DeriveInput) -> TokenStream {
    // Model name
    let name = input.ident;

    // Parsing struct attributes
    let mut model_hooks = Vec::new();
    let mut field_case = None;
    for attr in input.attrs.iter() {
        for (key, value) in parser::parse_schema_attr(attr).into_iter() {
            if let Some(value) = value {
                if key == "rename_all" {
                    field_case = match value.as_str() {
                        "lowercase" => Some("Lower"),
                        "UPPERCASE" => Some("Upper"),
                        "PascalCase" => Some("Pascal"),
                        "camelCase" => Some("Camel"),
                        "snake_case" => Some("Snake"),
                        "SCREAMING_SNAKE_CASE" => Some("Constant"),
                        "kebab-case" => Some("Kebab"),
                        "SCREAMING-KEBAB-CASE" => Some("Cobol"),
                        _ => None,
                    }
                }
            }
        }
    }
    if let Some(case) = field_case {
        let case_variant = format_ident!("{}", case);
        model_hooks.push(quote! {
            #[inline]
            async fn after_decode(model: &mut zino_core::Map) -> Result<(), zino_core::error::Error> {
                use convert_case::Case;
                use zino_core::extension::JsonObjectExt;

                model.rename_keys(Case::#case_variant);
                Ok(())
            }
        });
    }

    // Parsing field attributes
    let mut protected_fields = Vec::new();
    for field in parser::parse_struct_fields(input.data) {
        if let Some(ident) = field.ident {
            let mut protected = false;
            for attr in field.attrs.iter() {
                let arguments = parser::parse_schema_attr(attr);
                for (key, _value) in arguments.into_iter() {
                    if key == "protected" {
                        protected = true;
                    }
                }
            }
            if protected {
                let name = ident.to_string().trim_start_matches("r#").to_owned();
                protected_fields.push(name);
            }
        }
    }
    if !protected_fields.is_empty() {
        model_hooks.push(quote! {
            #[inline]
            async fn after_populate(model: &mut zino_core::Map) -> Result<(), zino_core::error::Error> {
                use zino_core::extension::JsonObjectExt;

                model.remove_all(&[#(#protected_fields),*]);
                Ok(())
            }
        });
    }

    quote! {
        impl zino_core::model::ModelHooks for #name {
            type Data = ();
            type Extension = ();

            #(#model_hooks)*
        }
    }
}
