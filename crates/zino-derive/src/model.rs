use super::parser;
use convert_case::{Boundary::LowerUpper, Case, Casing};
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
    let mut model_name = name.to_string();

    // Parsing struct attributes
    let mut item_name = "entry".to_owned();
    let mut item_name_plural = "entries".to_owned();
    for attr in input.attrs.iter() {
        for (key, value) in parser::parse_schema_attr(attr).into_iter() {
            if let Some(value) = value {
                match key.as_str() {
                    "model_name" => {
                        model_name = value;
                    }
                    "item_name" => {
                        item_name = value;
                    }
                    "item_name_plural" => {
                        item_name_plural = value;
                    }
                    _ => (),
                }
            }
        }
    }

    // Parsing field attributes
    let mut field_constructors = Vec::new();
    let mut field_setters = Vec::new();
    for field in parser::parse_struct_fields(input.data) {
        let type_name = parser::get_type_name(&field.ty);
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
                        "composable" => {
                            let setter = if parser::check_vec_type(&type_name) {
                                quote! {
                                    if let Some(objects) = data.get_map_array(#name) {
                                        let num_objects = objects.len();
                                        let mut models = Vec::with_capacity(num_objects);
                                        let mut errors = Vec::new();
                                        for (index, object) in objects.iter().enumerate() {
                                            match object.read_as_model() {
                                                Ok(model) => models.push(model),
                                                Err(err) => {
                                                    let message = format!("#{index}: {err}");
                                                    errors.push(message);
                                                },
                                            }
                                        }
                                        if !errors.is_empty() {
                                            validation.record(#name, errors.join(";"));
                                        }
                                        self.#ident = models;
                                    }
                                }
                            } else if parser::check_option_type(&type_name) {
                                quote! {
                                    if let Some(object) = data.parse_object(#name) {
                                        match object.read_as_model() {
                                            Ok(model) => self.#ident = Some(model),
                                            Err(err) => validation.record(#name, err.to_string()),
                                        }
                                    }
                                }
                            } else {
                                quote! {
                                    if let Some(object) = data.parse_object(#name) {
                                        match object.read_as_model() {
                                            Ok(model) => self.#ident = model,
                                            Err(err) => {
                                                validation.record(#name, err.to_string());
                                            },
                                        }
                                    }
                                }
                            };
                            field_setters.push(setter);
                        }
                        "default_value" => {
                            if let Some(value) = value {
                                if let Some((type_name, type_fn)) = value.split_once("::") {
                                    let type_name_ident = format_ident!("{}", type_name);
                                    let type_fn_ident = format_ident!("{}", type_fn);
                                    field_constructors.push(quote! {
                                        model.#ident = <#type_name_ident>::#type_fn_ident().into();
                                    });
                                } else {
                                    match type_name.as_str() {
                                        "String" => {
                                            field_constructors.push(quote! {
                                                model.#ident = #value.to_owned();
                                            });
                                        }
                                        "u64" => {
                                            if let Ok(value) = value.parse::<u64>() {
                                                field_constructors.push(quote! {
                                                    model.#ident = #value;
                                                });
                                            }
                                        }
                                        "i64" => {
                                            if let Ok(value) = value.parse::<i64>() {
                                                field_constructors.push(quote! {
                                                    model.#ident = #value;
                                                });
                                            }
                                        }
                                        "u32" => {
                                            if let Ok(value) = value.parse::<u32>() {
                                                field_constructors.push(quote! {
                                                    model.#ident = #value;
                                                });
                                            }
                                        }
                                        "i32" => {
                                            if let Ok(value) = value.parse::<i32>() {
                                                field_constructors.push(quote! {
                                                    model.#ident = #value;
                                                });
                                            }
                                        }
                                        "u16" => {
                                            if let Ok(value) = value.parse::<u16>() {
                                                field_constructors.push(quote! {
                                                    model.#ident = #value;
                                                });
                                            }
                                        }
                                        "i16" => {
                                            if let Ok(value) = value.parse::<i16>() {
                                                field_constructors.push(quote! {
                                                    model.#ident = #value;
                                                });
                                            }
                                        }
                                        "u8" => {
                                            if let Ok(value) = value.parse::<u8>() {
                                                field_constructors.push(quote! {
                                                    model.#ident = #value;
                                                });
                                            }
                                        }
                                        "i8" => {
                                            if let Ok(value) = value.parse::<i8>() {
                                                field_constructors.push(quote! {
                                                    model.#ident = #value;
                                                });
                                            }
                                        }
                                        _ => (),
                                    }
                                }
                            }
                        }
                        "ignore" | "read_only" | "generated" | "reserved" => {
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
                        let name_snake = name.with_boundaries(&[LowerUpper]).to_case(Case::Snake);
                        let parser_ident = format_ident!("parse_{}", name_snake);
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
                        if let Some(object) = data.parse_object(#name) {
                            self.#ident = object.clone();
                        }
                    }
                } else if parser::check_vec_type(&type_name) {
                    quote! {
                        if let Some(result) = data.parse_array(#name) {
                            match result {
                                Ok(values) => self.#ident = values,
                                Err(err) => validation.record_fail(#name, err),
                            }
                        }
                    }
                } else if let Some(type_generics) = parser::parse_option_type(&type_name) {
                    let type_generics_snake = type_generics
                        .with_boundaries(&[LowerUpper])
                        .to_case(Case::Snake);
                    let parser_ident = format_ident!("parse_{}", type_generics_snake);
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
                    let type_name_snake = type_name
                        .with_boundaries(&[LowerUpper])
                        .to_case(Case::Snake);
                    let parser_ident = format_ident!("parse_{}", type_name_snake);
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

    let model_name_snake = model_name.to_case(Case::Snake);
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
        impl zino_core::model::Model for #name {
            const MODEL_NAME: &'static str = #model_name_snake;
            const ITEM_NAME: (&'static str, &'static str) = (#item_name, #item_name_plural);

            #[inline]
            fn new() -> Self {
                #model_constructor
            }

            #[must_use]
            fn read_map(&mut self, data: &zino_core::Map) -> zino_core::validation::Validation {
                use zino_core::extension::JsonObjectExt;

                let mut validation = zino_core::validation::Validation::new();
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
