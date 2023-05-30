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

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

mod parser;

/// Derive the `Schema` trait.
#[proc_macro_derive(Schema, attributes(schema))]
pub fn schema_macro(item: TokenStream) -> TokenStream {
    /// Integer types
    const INTEGER_TYPES: [&str; 10] = [
        "u64", "i64", "u32", "i32", "u16", "i16", "u8", "i8", "usize", "isize",
    ];

    // Input
    let input = parse_macro_input!(item as DeriveInput);

    // Model name
    let name = input.ident;
    let mut model_name = name.to_string();

    // Parsing struct attrs
    let mut reader_name = String::from("main");
    let mut writer_name = String::from("main");
    for attr in input.attrs.iter() {
        for (key, value) in parser::parse_schema_attr(attr).into_iter() {
            if let Some(value) = value {
                match key.as_str() {
                    "model_name" => {
                        model_name = value;
                    }
                    "reader_name" => {
                        reader_name = value;
                    }
                    "writer_name" => {
                        writer_name = value;
                    }
                    _ => panic!("struct attribute `{key}` is not supported"),
                }
            }
        }
    }

    // Parsing field attrs
    let mut primary_key_type = String::from("Uuid");
    let mut primary_key_name = String::from("id");
    let mut distribution_column = None;
    let mut columns = Vec::new();
    let mut column_fields = Vec::new();
    let mut readonly_fields = Vec::new();
    let mut writeonly_fields = Vec::new();
    if let Data::Struct(data) = input.data && let Fields::Named(fields) = data.fields {
        for field in fields.named.into_iter() {
            let mut type_name = parser::get_type_name(&field.ty);
            if let Some(ident) = field.ident && !type_name.is_empty() {
                let mut ignore = false;
                let mut name = ident.to_string();
                let mut not_null = false;
                let mut default_value = None;
                let mut index_type = None;
                let mut reference = None;
                'inner: for attr in field.attrs.iter() {
                    for (key, value) in parser::parse_schema_attr(attr).into_iter() {
                        match key.as_str() {
                            "ignore" => {
                                ignore = true;
                                break 'inner;
                            }
                            "column_name" => {
                                if let Some(value) = value {
                                    name = value;
                                }
                            }
                            "column_type" => {
                                if let Some(value) = value {
                                    type_name = value;
                                }
                            }
                            "not_null" => {
                                not_null = true;
                            }
                            "default_value" => {
                                default_value = value;
                            }
                            "index_type" => {
                                index_type = value;
                            }
                            "reference" => {
                                reference = value;
                            }
                            "primary_key" => {
                                primary_key_name = name.clone();
                            }
                            "distribution_column" => {
                                distribution_column = Some(name.clone());
                            }
                            "readonly" => {
                                readonly_fields.push(quote!{ #name });
                            }
                            "writeonly" => {
                                writeonly_fields.push(quote!{ #name });
                            }
                            _ => panic!("field attribute `{key}` is not supported"),
                        }
                    }
                }
                if ignore {
                    continue;
                }
                if primary_key_name == name {
                    primary_key_type = type_name.clone();
                    not_null = true;
                } else if type_name.starts_with("Option") {
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
                let quote_reference = if let Some(ref model_name) = reference {
                    let model_ident = format_ident!("{}", model_name);
                    quote! {{
                        let table_name = <#model_ident>::table_name();
                        let column_name = <#model_ident>::PRIMARY_KEY_NAME;
                        Some(zino_core::model::Reference::new(table_name, column_name))
                    }}
                } else {
                    quote! { None }
                };
                let column = quote! {{
                    let mut column = zino_core::model::Column::new(#name, #type_name, #not_null);
                    if let Some(default_value) = #quote_value {
                        column.set_default_value(default_value);
                    }
                    if let Some(index_type) = #quote_index {
                        column.set_index_type(index_type);
                    }
                    if let Some(reference) = #quote_reference {
                        column.set_reference(reference);
                    }
                    column
                }};
                columns.push(column);
                column_fields.push(quote!{ #name });
            }
        }
    }

    // Output
    let model_name_snake = model_name.to_case(Case::Snake);
    let model_name_upper_snake = model_name.to_case(Case::UpperSnake);
    let quote_distribution_column = if let Some(column_name) = distribution_column {
        quote! { Some(#column_name) }
    } else {
        quote! { None }
    };
    let schema_primary_key_type = format_ident!("{}", primary_key_type);
    let schema_primary_key = format_ident!("{}", primary_key_name);
    let schema_columns = format_ident!("{}_COLUMNS", model_name_upper_snake);
    let schema_fields = format_ident!("{}_FIELDS", model_name_upper_snake);
    let schema_readonly_fields = format_ident!("{}_READONLY_FIELDS", model_name_upper_snake);
    let schema_writeonly_fields = format_ident!("{}_WRITEONLY_FIELDS", model_name_upper_snake);
    let schema_reader = format_ident!("{}_READER", model_name_upper_snake);
    let schema_writer = format_ident!("{}_WRITER", model_name_upper_snake);
    let avro_schema = format_ident!("{}_AVRO_SCHEMA", model_name_upper_snake);
    let num_columns = columns.len();
    let num_readonly_fields = readonly_fields.len();
    let num_writeonly_fields = writeonly_fields.len();
    let output = quote! {
        use zino_core::{
            database::{ConnectionPool, Schema},
            error::Error as ZinoError,
            model::{schema, Column},
        };

        static #avro_schema: std::sync::LazyLock<schema::Schema> = std::sync::LazyLock::new(|| {
            let mut fields = #schema_columns.iter().enumerate()
                .map(|(index, col)| {
                    let mut field = col.record_field();
                    field.position = index;
                    field
                })
                .collect::<Vec<_>>();
            schema::Schema::Record {
                name: schema::Name {
                    name: #model_name.to_owned(),
                    namespace: None,
                },
                aliases: None,
                doc: None,
                fields,
                lookup: std::collections::BTreeMap::new(),
            }
        });
        static #schema_columns: std::sync::LazyLock<[Column; #num_columns]> =
            std::sync::LazyLock::new(|| [#(#columns),*]);
        static #schema_fields: std::sync::LazyLock<[&'static str; #num_columns]> =
            std::sync::LazyLock::new(|| [#(#column_fields),*]);
        static #schema_readonly_fields: std::sync::LazyLock<[&'static str; #num_readonly_fields]> =
            std::sync::LazyLock::new(|| [#(#readonly_fields),*]);
        static #schema_writeonly_fields: std::sync::LazyLock<[&'static str; #num_writeonly_fields]> =
            std::sync::LazyLock::new(|| [#(#writeonly_fields),*]);
        static #schema_reader: std::sync::OnceLock<&ConnectionPool> = std::sync::OnceLock::new();
        static #schema_writer: std::sync::OnceLock<&ConnectionPool> = std::sync::OnceLock::new();

        impl Schema for #name {
            type PrimaryKey = #schema_primary_key_type;

            const MODEL_NAME: &'static str = #model_name_snake;
            const PRIMARY_KEY_NAME: &'static str = #primary_key_name;
            const READER_NAME: &'static str = #reader_name;
            const WRITER_NAME: &'static str = #writer_name;
            const DISTRIBUTION_COLUMN: Option<&'static str> = #quote_distribution_column;

            #[inline]
            fn primary_key(&self) -> &Self::PrimaryKey {
                &self.#schema_primary_key
            }

            #[inline]
            fn schema() -> &'static schema::Schema {
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

            async fn acquire_reader() -> Result<&'static ConnectionPool, ZinoError> {
                if let Some(connection_pool) = #schema_reader.get() {
                    Ok(*connection_pool)
                } else {
                    let model_name = Self::MODEL_NAME;
                    let connection_pool = Self::init_reader()?;
                    if let Err(err) = Self::create_table().await {
                        let message = format!("fail to acquire reader for the model `{model_name}`");
                        connection_pool.store_availability(false);
                        return Err(err.context(message));
                    }
                    if let Err(err) = Self::create_indexes().await {
                        let message = format!("fail to acquire reader for the model `{model_name}`");
                        connection_pool.store_availability(false);
                        return Err(err.context(message));
                    }
                    #schema_reader.set(connection_pool).map_err(|_| {
                        ZinoError::new(format!("fail to acquire reader for the model `{model_name}`"))
                    })?;
                    Ok(connection_pool)
                }
            }

            async fn acquire_writer() -> Result<&'static ConnectionPool, ZinoError> {
                if let Some(connection_pool) = #schema_writer.get() {
                    Ok(*connection_pool)
                } else {
                    let model_name = Self::MODEL_NAME;
                    let connection_pool = Self::init_writer()?;
                    if let Err(err) = Self::create_table().await {
                        let message = format!("fail to acquire writer for the model `{model_name}`");
                        connection_pool.store_availability(false);
                        return Err(err.context(message));
                    }
                    if let Err(err) = Self::create_indexes().await {
                        let message = format!("fail to acquire writer for the model `{model_name}`");
                        connection_pool.store_availability(false);
                        return Err(err.context(message));
                    }
                    #schema_writer.set(connection_pool).map_err(|_| {
                        ZinoError::new(format!("fail to acquire writer for the model `{model_name}`"))
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

/// Derive the `ModelAccessor` trait.
#[proc_macro_derive(ModelAccessor, attributes(schema))]
pub fn model_accessor_macro(item: TokenStream) -> TokenStream {
    // Input
    let input = parse_macro_input!(item as DeriveInput);

    // Parsing field attrs
    let name = input.ident;
    let mut column_methods = Vec::new();
    let mut snapshot_fields = Vec::new();
    let mut related_queries = Vec::new();
    let mut related_one_queries = Vec::new();
    let mut primary_key_type = String::from("Uuid");
    let mut primary_key_name = String::from("id");
    let mut user_id_type = String::from("Uuid");
    if let Data::Struct(data) = input.data && let Fields::Named(fields) = data.fields {
        let mut model_references: Vec<(String, Vec<String>)> = Vec::new();
        for field in fields.named.into_iter() {
            let type_name = parser::get_type_name(&field.ty);
            if let Some(ident) = field.ident && !type_name.is_empty() {
                let name = ident.to_string();
                for attr in field.attrs.iter() {
                    for (key, value) in parser::parse_schema_attr(attr).into_iter() {
                        if key == "primary_key" {
                            primary_key_name = name.clone();
                        } else if key == "reference" {
                            if let Some(value) = value {
                                match model_references.iter_mut().find(|r| r.0 == value) {
                                    Some(r) => r.1.push(name.clone()),
                                    None => model_references.push((value, vec![name.clone()])),
                                }
                            }
                        }
                    }
                }
                if primary_key_name == name {
                    primary_key_type = type_name;
                } else {
                    let name_ident = format_ident!("{}", name);
                    match name.as_str() {
                        "name" => {
                            let method = quote! {
                                #[inline]
                                fn #name_ident(&self) -> &str {
                                    &self.#name_ident
                                }
                            };
                            column_methods.push(method);
                            snapshot_fields.push("name");
                        }
                        "namespace" | "visibility" | "description" => {
                            let method = quote! {
                                #[inline]
                                fn #name_ident(&self) -> &str {
                                    &self.#name_ident
                                }
                            };
                            column_methods.push(method);
                        }
                        "status" => {
                            let method = quote! {
                                #[inline]
                                fn #name_ident(&self) -> &str {
                                    &self.#name_ident
                                }
                            };
                            column_methods.push(method);
                            snapshot_fields.push("status");
                        }
                        "content" | "extra" => {
                            let method = quote! {
                                #[inline]
                                fn #name_ident(&self) -> Option<&Map> {
                                    let map = &self.#name_ident;
                                    (!map.is_empty()).then_some(map)
                                }
                            };
                            column_methods.push(method);
                        }
                        "owner_id" | "maintainer_id" => {
                            let type_name_ident = format_ident!("{}", type_name);
                            let method = quote! {
                                #[inline]
                                fn #name_ident(&self) -> Option<&#type_name_ident> {
                                    let user_id = &self.#name_ident;
                                    (user_id != &#type_name_ident::default()).then_some(user_id)
                                }
                            };
                            column_methods.push(method);
                            user_id_type = type_name;
                        }
                        "created_at" => {
                            let method = quote! {
                                #[inline]
                                fn #name_ident(&self) -> DateTime {
                                    self.#name_ident
                                }
                            };
                            column_methods.push(method);
                        }
                        "updated_at" => {
                            let method = quote! {
                                #[inline]
                                fn #name_ident(&self) -> DateTime {
                                    self.#name_ident
                                }
                            };
                            column_methods.push(method);
                            snapshot_fields.push("updated_at");
                        }
                        "version" => {
                            let method = quote! {
                                #[inline]
                                fn #name_ident(&self) -> u64 {
                                    self.#name_ident
                                }
                            };
                            column_methods.push(method);
                            snapshot_fields.push("version");
                        }
                        "edition" => {
                            let method = quote! {
                                #[inline]
                                fn #name_ident(&self) -> u32 {
                                    self.#name_ident
                                }
                            };
                            column_methods.push(method);
                        }
                        _ => (),
                    }
                }
            }
        }
        if model_references.is_empty() {
            related_queries.push(quote! {
                let models = Self::find(query).await?;
            });
            related_one_queries.push(quote! {
                let model: Map = Self::find_by_id(id)
                    .await?
                    .ok_or_else(|| ZinoError::new(format!("cannot find the model `{id}`")))?;
            });
        } else {
            related_queries.push(quote! {
                let mut models = Self::find(query).await?;
            });
            related_one_queries.push(quote! {
                let mut model: Map = Self::find_by_id(id)
                    .await?
                    .ok_or_else(|| ZinoError::new(format!("cannot find the model `{id}`")))?;
            });
            for (model, fields) in model_references.into_iter() {
                let model_ident = format_ident!("{}", model);
                let related_query = quote! {
                    let mut query = #model_ident::default_snapshot_query();
                    #model_ident::find_related(&mut query, &mut models, [#(#fields),*]).await?;
                };
                let related_one_query = quote! {
                    let mut query = #model_ident::default_query();
                    #model_ident::find_related_one(&mut query, &mut model, [#(#fields),*]).await?;
                };
                related_queries.push(related_query);
                related_one_queries.push(related_one_query);
            }
        }
        related_queries.push(quote! { Ok(models) });
        related_one_queries.push(quote! { Ok(model) });
    }

    // Output
    let model_primary_key_type = format_ident!("{}", primary_key_type);
    let model_primary_key = format_ident!("{}", primary_key_name);
    let model_user_id_type = format_ident!("{}", user_id_type);
    let output = quote! {
        use zino_core::{
            database::ModelAccessor,
            model::Query,
            Map as ZinoMap,
        };

        impl ModelAccessor<#model_primary_key_type, #model_user_id_type> for #name {
            #[inline]
            fn id(&self) -> &#model_primary_key_type {
                &self.#model_primary_key
            }

            #(#column_methods)*

            fn default_snapshot_query() -> Query {
                let mut query = Self::default_query();
                let fields = [
                    Self::PRIMARY_KEY_NAME,
                    #(#snapshot_fields),*
                ];
                query.allow_fields(&fields);
                query
            }

            async fn fetch(query: &Query) -> Result<Vec<ZinoMap>, ZinoError> {
                #(#related_queries)*
            }

            async fn fetch_by_id(id: &#model_primary_key_type) -> Result<ZinoMap, ZinoError> {
                #(#related_one_queries)*
            }
        }
    };

    TokenStream::from(output)
}
