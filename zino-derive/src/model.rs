use super::parser;
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

/// Reserved fields
const RESERVED_FIELDS: [&str; 8] = [
    "is_deleted",
    "is_locked",
    "is_archived",
    "created_at",
    "updated_at",
    "deleted_at",
    "version",
    "edition",
];

/// Parses the token stream for the `Model` trait derivation.
pub(super) fn parse_token_stream(input: DeriveInput) -> TokenStream {
    // Model name
    let name = input.ident;

    // Parsing field attributes
    let mut field_constructors = Vec::new();
    let mut field_setters = Vec::new();
    for field in parser::parse_struct_fields(input.data) {
        let type_name = parser::get_type_name(&field.ty);
        if type_name.is_empty() {
            continue;
        }
        if let Some(ident) = field.ident {
            let name = ident.to_string();
            let mut enable_setter = true;
            let mut is_inherent = false;
            for attr in field.attrs.iter() {
                let arguments = parser::parse_schema_attr(attr);
                for (key, value) in arguments.into_iter() {
                    match key.as_str() {
                        "constructor" => {
                            if let Some(value) = value {
                                if let Some((cons_name, cons_fn)) = value.split_once("::") {
                                    let cons_name_ident = format_ident!("{}", cons_name);
                                    let cons_fn_ident = format_ident!("{}", cons_fn);
                                    let constructor = if type_name == "String" {
                                        quote! {
                                            model.#ident = <#cons_name_ident>::#cons_fn_ident().to_string();
                                        }
                                    } else {
                                        quote! {
                                            model.#ident = <#cons_name_ident>::#cons_fn_ident().into();
                                        }
                                    };
                                    field_constructors.push(constructor);
                                }
                            }
                        }
                        "read_only" | "generated" | "reserved" => {
                            enable_setter = false;
                        }
                        "inherent" => {
                            is_inherent = true;
                        }
                        _ => (),
                    }
                }
            }
            if enable_setter && !RESERVED_FIELDS.contains(&name.as_str()) {
                let setter = if type_name == "String" {
                    if is_inherent {
                        let parser_ident = format_ident!("parse_{}", name.to_case(Case::Snake));
                        quote! {
                            if let Some(value) = data.parse_string(#name) {
                                match Self::#parser_ident(&value) {
                                    Ok(value) => self.#ident = value,
                                    Err(err) => validation.record_fail(#name, err),
                                }
                            }
                        }
                    } else if name == "password" {
                        quote! {
                            if let Some(password) = data.parse_string(#name) {
                                use zino_core::orm::ModelHelper;
                                match Self::encrypt_password(&password) {
                                    Ok(password) => self.password = password,
                                    Err(err) => validation.record_fail(#name, err),
                                }
                            }
                        }
                    } else {
                        quote! {
                            if let Some(value) = data.parse_string(#name) {
                                self.#ident = value.into_owned();
                            }
                        }
                    }
                } else if type_name == "Vec<String>" {
                    quote! {
                        if let Some(values) = data.parse_str_array(#name) {
                            self.#ident = values.into_iter().map(|s| s.to_owned()).collect();
                        }
                    }
                } else if type_name == "Option<String>" {
                    quote! {
                        if let Some(value) = data.parse_string(#name) {
                            self.#ident = Some(value.into_owned());
                        }
                    }
                } else if type_name == "Map" {
                    quote! {
                        if let Some(values) = data.parse_object(#name) {
                            self.#ident = values.clone();
                        }
                    }
                } else if parser::check_vec_type(&type_name) {
                    quote! {
                        if let Some(values) = data.parse_array(#name) {
                            self.#ident = values;
                        }
                    }
                } else if let Some(type_generics) = parser::parse_option_type(&type_name) {
                    let parser_ident = format_ident!("parse_{}", type_generics.to_lowercase());
                    quote! {
                        if let Some(result) = data.#parser_ident(#name) {
                            match result {
                                Ok(value) => self.#ident = Some(value),
                                Err(err) => {
                                    let raw_value = data.parse_string(#name);
                                    let raw_value_str = raw_value
                                        .as_deref()
                                        .unwrap_or_default();
                                    let message = format!("{err}: `{raw_value_str}`");
                                    validation.record(#name, message);
                                },
                            }
                        }
                    }
                } else {
                    let parser_ident = format_ident!("parse_{}", type_name.to_lowercase());
                    quote! {
                        if let Some(result) = data.#parser_ident(#name) {
                            match result {
                                Ok(value) => self.#ident = value,
                                Err(err) => {
                                    let raw_value = data.parse_string(#name);
                                    let raw_value_str = raw_value
                                        .as_deref()
                                        .unwrap_or_default();
                                    let message = format!("{err}: `{raw_value_str}`");
                                    validation.record(#name, message);
                                },
                            }
                        }
                    }
                };
                field_setters.push(setter);
            }
        }
    }

    let model_constructor = if field_constructors.is_empty() {
        quote! { Self::default() }
    } else {
        quote! {
            let mut model = Self::default();
            #(#field_constructors)*
            model
        }
    };
    quote! {
        use zino_core::validation::Validation;

        impl zino_core::model::Model for #name {
            #[inline]
            fn new() -> Self {
                #model_constructor
            }

            #[must_use]
            fn read_map(&mut self, data: &Map) -> Validation {
                let mut validation = Validation::new();
                if data.is_empty() {
                    validation.record("data", "should be nonempty");
                } else {
                    #(#field_setters)*
                }
                validation
            }
        }
    }
}
