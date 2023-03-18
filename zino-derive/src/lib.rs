//! [![github]](https://github.com/photino/zino)
//! [![crates-io]](https://crates.io/crates/zino-derive)
//! [![docs-rs]](https://docs.rs/zino-derive)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?labelColor=555555&logo=docs.rs
//!
//! Derived traits for [`zino`].
//!
//! [`zino`]: https://github.com/photino/zino

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
    let mut primary_key_name = String::from("id");
    let mut reader_name = String::from("main");
    let mut writer_name = String::from("main");
    let mut distribution_column = None;
    for attr in input.attrs.iter() {
        for (key, value) in parser::parse_attr(attr).into_iter() {
            if let Some(value) = value {
                match key.as_str() {
                    "type_name" => {
                        type_name = value;
                    }
                    "primary_key" => {
                        primary_key_name = value;
                    }
                    "reader_name" => {
                        reader_name = value;
                    }
                    "writer_name" => {
                        writer_name = value;
                    }
                    "distribution_column" => {
                        distribution_column = Some(value);
                    }
                    _ => panic!("struct attribute `{key}` is not supported"),
                }
            }
        }
    }

    // Columns
    let mut columns = Vec::new();
    let mut column_fields = Vec::new();
    let mut readonly_fields = Vec::new();
    let mut writeonly_fields = Vec::new();
    if let Data::Struct(data) = input.data && let Fields::Named(fields) = data.fields {
        for field in fields.named.into_iter() {
            let mut type_name = parser::get_type_name(&field.ty);
            if let Some(ident) = field.ident && !type_name.is_empty() {
                let name = ident.to_string();
                let mut default_value = None;
                let mut not_null = false;
                let mut index_type = None;
                for attr in field.attrs.iter() {
                    for (key, value) in parser::parse_attr(attr).into_iter() {
                        match key.as_str() {
                            "type_name" => {
                                if let Some(value) = value {
                                    type_name = value;
                                }
                            }
                            "not_null" => {
                                not_null = true;
                            }
                            "default" => {
                                default_value = value;
                            }
                            "index" => {
                                index_type = value;
                            }
                            "readonly" => {
                                readonly_fields.push(quote!{ #name });
                            }
                            "writeonly" => {
                                writeonly_fields.push(quote!{ #name });
                            }
                            "readwrite" => (),
                            "internal" => {
                                readonly_fields.push(quote!{ #name });
                                writeonly_fields.push(quote!{ #name });
                            },
                            _ => panic!("field attribute `{key}` is not supported"),
                        }
                    }
                }
                if type_name.starts_with("Option") {
                    not_null = false;
                } else if type_name == "Uuid" {
                    not_null = true;
                } else if INTEGER_TYPES.contains(&type_name.as_str()) {
                    default_value = default_value.or_else(|| Some("0".to_owned()));
                }
                let quote_value = if let Some(value) = default_value {
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
                } else {
                    quote! { None }
                };
                let quote_index = if let Some(index) = index_type {
                    quote! { Some(#index) }
                } else {
                    quote! { None }
                };
                let column = quote! {
                    zino_core::model::Column::new(#name, #type_name, #quote_value, #not_null, #quote_index)
                };
                columns.push(column);
                column_fields.push(quote!{ #name });
            }
        }
    }

    // Output
    let type_name_lowercase = type_name.to_ascii_lowercase();
    let type_name_uppercase = type_name.to_ascii_uppercase();
    let quote_distribution_column = if let Some(column_name) = distribution_column {
        quote! { Some(#column_name) }
    } else {
        quote! { None }
    };
    let schema_primary_key = format_ident!("{}", primary_key_name);
    let schema_columns = format_ident!("{}_COLUMNS", type_name_uppercase);
    let schema_fields = format_ident!("{}_FIELDS", type_name_uppercase);
    let schema_readonly_fields = format_ident!("{}_READONLY_FIELDS", type_name_uppercase);
    let schema_writeonly_fields = format_ident!("{}_WRITEONLY_FIELDS", type_name_uppercase);
    let schema_reader = format_ident!("{}_READER", type_name_uppercase);
    let schema_writer = format_ident!("{}_WRITER", type_name_uppercase);
    let avro_schema = format_ident!("{}_AVRO_SCHEMA", type_name_uppercase);
    let columns_len = columns.len();
    let readonly_fields_len = readonly_fields.len();
    let writeonly_fields_len = writeonly_fields.len();
    let output = quote! {
        use zino_core::{
            database::{ConnectionPool, Schema},
            error::Error as ZinoError,
            model::Column,
        };

        static #avro_schema: std::sync::LazyLock<apache_avro::Schema> = std::sync::LazyLock::new(|| {
            use apache_avro::schema::{Name, RecordField, RecordFieldOrder, Schema};
            let mut fields = #schema_columns.iter().enumerate()
                .map(|(index, col)| {
                    let schema = col.schema();
                    let default_value = col.default_value().and_then(|s| {
                        match schema {
                           Schema::Boolean => s.parse::<bool>().ok().map(|b| b.into()),
                           Schema::Int => s.parse::<i32>().ok().map(|i| i.into()),
                           Schema::Long => s.parse::<i64>().ok().map(|i| i.into()),
                           Schema::Float => s.parse::<f32>().ok().map(|f| f.into()),
                           Schema::Double => s.parse::<f64>().ok().map(|f| f.into()),
                           _ => Some(s.into()),
                        }
                    });
                    RecordField {
                        name: col.name().to_owned(),
                        doc: None,
                        default: default_value,
                        schema,
                        order: RecordFieldOrder::Ascending,
                        position: index,
                    }
                })
                .collect::<Vec<_>>();
            Schema::Record {
                name: Name {
                    name: #type_name.to_owned(),
                    namespace: None,
                },
                aliases: None,
                doc: None,
                fields,
                lookup: std::collections::BTreeMap::new(),
            }
        });
        static #schema_columns: std::sync::LazyLock<[Column; #columns_len]> =
            std::sync::LazyLock::new(|| [#(#columns),*]);
        static #schema_fields: std::sync::LazyLock<[&'static str; #columns_len]> =
            std::sync::LazyLock::new(|| [#(#column_fields),*]);
        static #schema_readonly_fields: std::sync::LazyLock<[&'static str; #readonly_fields_len]> =
            std::sync::LazyLock::new(|| [#(#readonly_fields),*]);
        static #schema_writeonly_fields: std::sync::LazyLock<[&'static str; #writeonly_fields_len]> =
            std::sync::LazyLock::new(|| [#(#writeonly_fields),*]);
        static #schema_reader: std::sync::OnceLock<&ConnectionPool> = std::sync::OnceLock::new();
        static #schema_writer: std::sync::OnceLock<&ConnectionPool> = std::sync::OnceLock::new();

        impl Schema for #name {
            const TYPE_NAME: &'static str = #type_name_lowercase;
            const PRIMARY_KEY_NAME: &'static str = #primary_key_name;
            const READER_NAME: &'static str = #reader_name;
            const WRITER_NAME: &'static str = #writer_name;
            const DISTRIBUTION_COLUMN: Option<&'static str> = #quote_distribution_column;

            fn schema() -> &'static apache_avro::Schema {
                std::sync::LazyLock::force(&#avro_schema)
            }

            #[inline]
            fn columns() -> &'static [Column<'static>] {
                #schema_columns.as_slice()
            }

            #[inline]
            fn fields() -> &'static [&'static str] {
                #schema_fields.as_slice()
            }

            #[inline]
            fn readonly_fields() -> &'static [&'static str] {
                #schema_readonly_fields.as_slice()
            }

            #[inline]
            fn writeonly_fields() -> &'static [&'static str] {
                #schema_writeonly_fields.as_slice()
            }

            #[inline]
            fn primary_key(&self) -> String {
                self.#schema_primary_key.to_string()
            }

            async fn acquire_reader() -> Result<&'static ConnectionPool, ZinoError> {
                if let Some(connection_pool) = #schema_reader.get() {
                    Ok(*connection_pool)
                } else {
                    let connection_pool = Self::init_reader()?;
                    if let Err(err) = Self::create_table().await {
                        let message = format!("fail to acquire reader for the model `{}`", Self::TYPE_NAME);
                        connection_pool.store_availability(false);
                        return Err(err.context(message));
                    }
                    if let Err(err) = Self::create_indexes().await {
                        let message = format!("fail to acquire reader for the model `{}`", Self::TYPE_NAME);
                        connection_pool.store_availability(false);
                        return Err(err.context(message));
                    }
                    #schema_reader.set(connection_pool).map_err(|_| {
                        ZinoError::new(format!("fail to acquire reader for the model `{}`", Self::TYPE_NAME))
                    })?;
                    Ok(connection_pool)
                }
            }

            async fn acquire_writer() -> Result<&'static ConnectionPool, ZinoError> {
                if let Some(connection_pool) = #schema_writer.get() {
                    Ok(*connection_pool)
                } else {
                    let connection_pool = Self::init_writer()?;
                    if let Err(err) = Self::create_table().await {
                        let message = format!("fail to acquire writer for the model `{}`", Self::TYPE_NAME);
                        connection_pool.store_availability(false);
                        return Err(err.context(message));
                    }
                    if let Err(err) = Self::create_indexes().await {
                        let message = format!("fail to acquire writer for the model `{}`", Self::TYPE_NAME);
                        connection_pool.store_availability(false);
                        return Err(err.context(message));
                    }
                    #schema_writer.set(connection_pool).map_err(|_| {
                        ZinoError::new(format!("fail to acquire writer for the model `{}`", Self::TYPE_NAME))
                    })?;
                    Ok(connection_pool)
                }
            }
        }

        impl PartialEq for #name {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.#schema_primary_key == other.#schema_primary_key
            }
        }

        impl Eq for #name {}
    };

    TokenStream::from(output)
}
