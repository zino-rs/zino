use super::parser;
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

/// Parses the token stream for the `Entity` trait derivation.
pub(super) fn parse_token_stream(input: DeriveInput) -> TokenStream {
    // Model name
    let name = input.ident;

    let mut primary_key_name = String::from("id");
    let mut model_column_variants = Vec::new();
    let mut model_column_mappings = Vec::new();
    for field in parser::parse_struct_fields(input.data) {
        if let Some(ident) = field.ident {
            let mut name = ident.to_string().trim_start_matches("r#").to_owned();
            let variant = format_ident!("{}", name.to_case(Case::Pascal));
            'inner: for attr in field.attrs.iter() {
                let arguments = parser::parse_schema_attr(attr);
                for (key, value) in arguments.into_iter() {
                    match key.as_str() {
                        "ignore" => break 'inner,
                        "primary_key" => {
                            primary_key_name.clone_from(&name);
                        }
                        "column_name" => {
                            if let Some(value) = value {
                                name = value;
                            }
                        }
                        _ => (),
                    }
                }
            }
            model_column_variants.push(quote! {
                #variant,
            });
            model_column_mappings.push(quote! {
                #variant => #name,
            });
        }
    }

    let model_column_type = format_ident!("{}Column", name);
    let primary_key_variant = format_ident!("{}", primary_key_name.to_case(Case::Pascal));
    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum #model_column_type {
            #(#model_column_variants)*
        }

        impl AsRef<str> for #model_column_type {
            #[inline]
            fn as_ref(&self) -> &str {
                use #model_column_type::*;
                match self {
                    #(#model_column_mappings)*
                }
            }
        }

        impl std::fmt::Display for #model_column_type {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <#name as zino_orm::Entity>::format_column(self).fmt(f)
            }
        }

        impl zino_orm::ModelColumn<#name> for #model_column_type {
            #[inline]
            fn into_column_expr(self) -> String {
                <#name as zino_orm::Entity>::format_column(&self)
            }
        }

        impl zino_orm::Entity for #name {
            type Column = #model_column_type;
            const PRIMARY_KEY: Self::Column = <#model_column_type>::#primary_key_variant;
        }
    }
}
