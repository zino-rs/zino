//! Derived traits for zino.

#![feature(let_chains)]
#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

mod parser;

/// Derive the `Schema` trait.
#[proc_macro_derive(Schema, attributes(schema))]
pub fn schema_macro(item: TokenStream) -> TokenStream {
    /// Integer types
    const INTEGER_TYPES: [&str; 6] = ["u64", "i64", "u32", "i32", "u16", "i16"];

    // Input
    let input = parse_macro_input!(item as DeriveInput);

    // Type name
    let name = input.ident;
    let mut type_name = name.to_string();

    // Reader and writer
    let mut reader_name = String::from("main");
    let mut writer_name = String::from("main");
    for attr in input.attrs.iter() {
        for (key, value) in parser::parse_attr(attr).into_iter() {
            if let Some(value) = value {
                if key == "type_name" {
                    type_name = value;
                } else if key == "reader_name" {
                    reader_name = value;
                } else if key == "writer_name" {
                    writer_name = value;
                }
            }
        }
    }

    // Columns
    let type_name_lowercase = type_name.to_ascii_lowercase();
    let type_name_uppercase = type_name.to_ascii_uppercase();
    let mut primary_key = String::from("id");
    let mut columns = Vec::new();
    if let Data::Struct(data) = input.data && let Fields::Named(fields) = data.fields {
        for field in fields.named.into_iter() {
            let name = field.ident.unwrap().to_string();
            let mut type_name = parser::get_type_name(&field.ty);
            if !type_name.is_empty() {
                let mut default_value = None;
                let mut not_null = false;
                let mut index_type = None;
                for attr in field.attrs.iter() {
                    for (key, value) in parser::parse_attr(attr).into_iter() {
                        if key == "type_name" {
                            if let Some(value) = value {
                                type_name = value;
                            }
                        } else if key == "primary_key" {
                            primary_key.clone_from(&name);
                        } else if key == "not_null" {
                            not_null = true;
                        } else if key == "default" {
                            default_value = value;
                        } else if key == "index" {
                            index_type = value;
                        }
                    }
                }
                if type_name.starts_with("Option") {
                    not_null = false;
                } else if type_name == "Uuid" {
                    not_null = true;
                } else if INTEGER_TYPES.contains(&type_name.as_str()) {
                    default_value = default_value.or_else(|| Some("0".to_string()));
                }
                let quote_value = match default_value {
                    Some(value) => {
                        if value.contains("::") {
                            if let Some((type_name, type_fn)) = value.split_once("::") {
                                let type_name_ident = format_ident!("{}", type_name);
                                let type_fn_ident = format_ident!("{}", type_fn);
                                quote! { Some(<#type_name_ident>::#type_fn_ident()) }
                            } else {
                                quote! { Some(#value) }
                            }
                        } else {
                            quote! { Some(#value) }
                        }
                    }
                    None => quote! { None },
                };
                let quote_index = match index_type {
                    Some(index) => quote! { Some(#index) },
                    None => quote! { None },
                };
                let column = quote! {
                    zino_core::Column::new(#name, #type_name, #quote_value, #not_null, #quote_index)
                };
                columns.push(column);
            }
        }
    }

    // Output
    let schema_primary_key = format_ident!("{}", primary_key);
    let schema_columns = format_ident!("{}_COLUMNS", type_name_uppercase);
    let schema_reader = format_ident!("{}_READER", type_name_uppercase);
    let schema_writer = format_ident!("{}_WRITER", type_name_uppercase);
    let output = quote! {
        use std::sync::{LazyLock, OnceLock};

        static #schema_columns: LazyLock<Vec<zino_core::Column>> = LazyLock::new(|| {
            vec![#(#columns),*]
        });
        static #schema_reader: OnceLock<&zino_core::ConnectionPool> = OnceLock::new();
        static #schema_writer: OnceLock<&zino_core::ConnectionPool> = OnceLock::new();

        impl zino_core::Schema for #name {
            /// Type name as a str.
            const TYPE_NAME: &'static str = #type_name_lowercase;
            /// Primary key name as a str.
            const PRIMARY_KEY_NAME: &'static str = #primary_key;
            /// Reader name.
            const READER_NAME: &'static str = #reader_name;
            /// Writer name.
            const WRITER_NAME: &'static str = #writer_name;

            /// Returns a reference to the columns.
            #[inline]
            fn columns() -> &'static[zino_core::Column<'static>] {
                std::sync::LazyLock::force(&#schema_columns).as_slice()
            }

            /// Returns the primary key value as a `String`.
            #[inline]
            fn primary_key(&self) -> String {
                self.#schema_primary_key.to_string()
            }

            /// Initializes model reader.
            async fn init_reader() -> Option<&'static zino_core::ConnectionPool> {
                match #schema_reader.get() {
                    Some(connection_pool) => Some(*connection_pool),
                    None => {
                        let connection_pool = Self::get_reader()?;
                        let _ = Self::create_table().await.ok()?;
                        let _ = Self::create_indexes().await.ok()?;
                        let _ = #schema_reader.set(connection_pool).ok()?;
                        Some(connection_pool)
                    },
                }
            }

            /// Initializes model writer.
            async fn init_writer() -> Option<&'static zino_core::ConnectionPool> {
                match #schema_writer.get() {
                    Some(connection_pool) => Some(*connection_pool),
                    None => {
                        let connection_pool = Self::get_writer()?;
                        let _ = Self::create_table().await.ok()?;
                        let _ = Self::create_indexes().await.ok()?;
                        let _ = #schema_writer.set(connection_pool).ok()?;
                        Some(connection_pool)
                    },
                }
            }
        }

        impl std::cmp::PartialEq for #name {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.#schema_primary_key == other.#schema_primary_key
            }
        }

        impl std::cmp::Eq for #name {}
    };

    TokenStream::from(output)
}
