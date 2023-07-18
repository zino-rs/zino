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
    let mut documentation = None;
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
                    "doc" => {
                        documentation = Some(value);
                    }
                    _ => (),
                }
            }
        }
    }

    // Parsing field attrs
    let mut primary_key_type = String::from("Uuid");
    let mut primary_key_name = String::from("id");
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
                    let arguments = parser::parse_schema_attr(attr);
                    for (key, value) in arguments.into_iter() {
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
                            "length" if type_name == "String" => {
                                if let Some(value) = value {
                                    type_name = format!("CHAR({value})");
                                }
                            }
                            "max_length" if type_name == "String" => {
                                if let Some(value) = value {
                                    type_name = format!("VARCHAR({value})");
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
                            "readonly" => {
                                readonly_fields.push(quote!{ #name });
                            }
                            "writeonly" => {
                                writeonly_fields.push(quote!{ #name });
                            }
                            _ => (),
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
                column_fields.push(quote! { #name });
            }
        }
    }

    // Output
    let model_name_snake = model_name.to_case(Case::Snake);
    let model_name_upper_snake = model_name.to_case(Case::UpperSnake);
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
    let quote_documentation = if let Some(doc) = documentation {
        quote! { Some(#doc) }
    } else {
        quote! { None }
    };
    let output = quote! {
        use zino_core::{
            database::{self, ConnectionPool, Schema},
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
            let record_schema = schema::RecordSchema {
                name: schema::Name {
                    name: #model_name.to_owned(),
                    namespace: Some(<#name>::model_namespace().to_owned()),
                },
                aliases: None,
                doc: #quote_documentation,
                fields,
                lookup: std::collections::BTreeMap::new(),
                attributes: std::collections::BTreeMap::new(),
            };
            schema::Schema::Record(record_schema)
        });
        static #schema_columns: std::sync::LazyLock<[Column; #num_columns]> =
            std::sync::LazyLock::new(|| [#(#columns),*]);
        static #schema_fields: std::sync::LazyLock<[&str; #num_columns]> =
            std::sync::LazyLock::new(|| [#(#column_fields),*]);
        static #schema_readonly_fields: std::sync::LazyLock<[&str; #num_readonly_fields]> =
            std::sync::LazyLock::new(|| [#(#readonly_fields),*]);
        static #schema_writeonly_fields: std::sync::LazyLock<[&str; #num_writeonly_fields]> =
            std::sync::LazyLock::new(|| [#(#writeonly_fields),*]);
        static #schema_reader: std::sync::OnceLock<&ConnectionPool> = std::sync::OnceLock::new();
        static #schema_writer: std::sync::OnceLock<&ConnectionPool> = std::sync::OnceLock::new();

        impl Schema for #name {
            type PrimaryKey = #schema_primary_key_type;

            const MODEL_NAME: &'static str = #model_name_snake;
            const PRIMARY_KEY_NAME: &'static str = #primary_key_name;
            const READER_NAME: &'static str = #reader_name;
            const WRITER_NAME: &'static str = #writer_name;

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
                        let message = format!("503 Service Unavailable: fail to acquire reader for the model `{model_name}`");
                        connection_pool.store_availability(false);
                        return Err(err.context(message));
                    }
                    if let Err(err) = Self::create_indexes().await {
                        let message = format!("503 Service Unavailable: fail to acquire reader for the model `{model_name}`");
                        connection_pool.store_availability(false);
                        return Err(err.context(message));
                    }
                    #schema_reader.set(connection_pool).map_err(|_| {
                        ZinoError::new(format!("503 Service Unavailable: fail to acquire reader for the model `{model_name}`"))
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
                        let message = format!("503 Service Unavailable: fail to acquire writer for the model `{model_name}`");
                        connection_pool.store_availability(false);
                        return Err(err.context(message));
                    }
                    if let Err(err) = Self::create_indexes().await {
                        let message = format!("503 Service Unavailable: fail to acquire writer for the model `{model_name}`");
                        connection_pool.store_availability(false);
                        return Err(err.context(message));
                    }
                    #schema_writer.set(connection_pool).map_err(|_| {
                        ZinoError::new(format!("503 Service Unavailable: fail to acquire writer for the model `{model_name}`"))
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

/// Derive the `DecodeRow` trait.
#[proc_macro_derive(DecodeRow, attributes(schema))]
pub fn decode_row_macro(item: TokenStream) -> TokenStream {
    /// Integer types
    const UNSIGNED_INTEGER_TYPES: [&str; 5] = ["u64", "u32", "u16", "u8", "usize"];

    // Input
    let input = parse_macro_input!(item as DeriveInput);

    // Parsing field attrs
    let name = input.ident;
    let mut decode_model_fields = Vec::new();
    let mut mysql_decode_model_fields = Vec::new();
    let mut postgres_decode_model_fields = Vec::new();
    if let Data::Struct(data) = input.data && let Fields::Named(fields) = data.fields {
        for field in fields.named.into_iter() {
            let type_name = parser::get_type_name(&field.ty);
            if let Some(ident) = field.ident && !type_name.is_empty() {
                let mut ignore = false;
                'inner: for attr in field.attrs.iter() {
                    let arguments = parser::parse_schema_attr(attr);
                    for (key, _value) in arguments.iter() {
                        if key == "ignore" || key == "writeonly" {
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
                        if let JsonValue::Object(map) = database::decode(row, #name)? {
                            model.#ident = map;
                        }
                    });
                } else if type_name.starts_with("Vec") {
                    mysql_decode_model_fields.push(quote! {
                        let value = database::decode::<JsonValue>(row, #name)?;
                        if let Some(vec) = value.parse_array() {
                            model.#ident = vec;
                        }
                    });
                    postgres_decode_model_fields.push(quote! {
                        model.#ident = database::decode(row, #name)?;
                    });
                } else if UNSIGNED_INTEGER_TYPES.contains(&type_name.as_str()) {
                    let integer_type_ident = format_ident!("{}", type_name.replace('u', "i"));
                    postgres_decode_model_fields.push(quote! {
                        let value = database::decode::<#integer_type_ident>(row, #name)?;
                        model.#ident = value.try_into()?;
                    });
                    mysql_decode_model_fields.push(quote! {
                        model.#ident = database::decode(row, #name)?;
                    });
                } else {
                    decode_model_fields.push(quote! {
                        model.#ident = database::decode(row, #name)?;
                    });
                }
            }
        }
    }

    // Output
    let output = quote! {
        use zino_core::{
            database::DatabaseRow,
            model::DecodeRow,
        };

        impl DecodeRow<DatabaseRow> for #name {
            type Error = zino_core::error::Error;

            fn decode_row(row: &DatabaseRow) -> Result<Self, Self::Error> {
                use zino_core::{extension::JsonValueExt, JsonValue};
                let mut model = <#name>::default();
                #(#decode_model_fields)*
                if cfg!(feature = "orm-mysql") {
                    #(#mysql_decode_model_fields)*
                } else {
                    #(#postgres_decode_model_fields)*
                }
                Ok(model)
            }
        }
    };

    TokenStream::from(output)
}

/// Derive the `ModelAccessor` trait.
#[proc_macro_derive(ModelAccessor, attributes(schema))]
pub fn model_accessor_macro(item: TokenStream) -> TokenStream {
    /// Primitive types
    const PRIMITIVE_TYPES: [&str; 13] = [
        "u64", "i64", "u32", "i32", "u16", "i16", "u8", "i8", "usize", "isize", "f32", "f64",
        "bool",
    ];

    // Input
    let input = parse_macro_input!(item as DeriveInput);

    // Parsing struct attrs
    let mut compound_constraints = Vec::new();
    for attr in input.attrs.iter() {
        for (key, value) in parser::parse_schema_attr(attr).into_iter() {
            if let Some(value) = value && key == "unique_on" {
                let mut fields = Vec::new();
                let column_values = value
                    .trim_start_matches('(')
                    .trim_end_matches(')')
                    .split(',')
                    .map(|s| {
                        let field = s.trim();
                        let field_ident = format_ident!("{}", field);
                        fields.push(field);
                        quote! {
                            (#field, self.#field_ident.to_string().into())
                        }
                    })
                    .collect::<Vec<_>>();
                let compound_field = fields.join("_");
                compound_constraints.push(quote! {
                    let columns = [#(#column_values),*];
                    if !self.is_unique_on(columns).await? {
                        validation.record(#compound_field, "it should be unique");
                    }
                });
            }
        }
    }

    // Parsing field attrs
    let name = input.ident;
    let mut column_methods = Vec::new();
    let mut snapshot_fields = Vec::new();
    let mut snapshot_entries = Vec::new();
    let mut field_constraints = Vec::new();
    let mut populated_queries = Vec::new();
    let mut populated_one_queries = Vec::new();
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
                    let arguments = parser::parse_schema_attr(attr);
                    let is_readable = arguments.iter().all(|arg| arg.0 != "writeonly");
                    for (key, value) in arguments.into_iter() {
                        match key.as_str() {
                            "primary_key" => {
                                primary_key_name = name.clone();
                            }
                            "snapshot" => {
                                let field = name.clone();
                                let field_ident = format_ident!("{}", field);
                                if type_name.starts_with("Vec") {
                                    snapshot_entries.push(quote! {
                                        let snapshot_value = self.#field_ident.iter()
                                            .map(|v| v.to_string())
                                            .collect::<Vec<_>>();
                                        snapshot.upsert(#field, snapshot_value);
                                    });
                                } else if type_name == "Option<Uuid>" {
                                    snapshot_entries.push(quote! {
                                        let snapshot_value = self.#field_ident
                                            .map(|v| v.to_string());
                                        snapshot.upsert(#field, snapshot_value);
                                    });
                                }  else if PRIMITIVE_TYPES.contains(&type_name.as_str()) {
                                    snapshot_entries.push(quote! {
                                        snapshot.upsert(#field, self.#field_ident);
                                    });
                                } else {
                                    snapshot_entries.push(quote! {
                                        snapshot.upsert(#field, self.#field_ident.to_string());
                                    });
                                }
                                snapshot_fields.push(field);
                            }
                            "reference" => {
                                if let Some(value) = value {
                                    let model_ident = format_ident!("{}", value);
                                    if type_name.starts_with("Vec") {
                                        field_constraints.push(quote! {
                                            let values = self.#ident
                                                .iter()
                                                .map(|value| value.to_string())
                                                .collect::<Vec<_>>();
                                            let length = values.len();
                                            if length > 0 {
                                                let data = <#model_ident>::filter(values).await?;
                                                if data.len() != length {
                                                    validation.record(#name, "there are nonexistent values");
                                                }
                                            }
                                        });
                                    } else if type_name.starts_with("Option") {
                                        field_constraints.push(quote! {
                                            if let Some(value) = self.#ident {
                                                let values = vec![value.to_string()];
                                                let data = <#model_ident>::filter(values).await?;
                                                if data.len() != 1 {
                                                    validation.record(#name, "it is a nonexistent value");
                                                }
                                            }
                                        });
                                    } else {
                                        field_constraints.push(quote! {
                                            let values = vec![self.#ident.to_string()];
                                            let data = <#model_ident>::filter(values).await?;
                                            if data.len() != 1 {
                                                validation.record(#name, "it is a nonexistent value");
                                            }
                                        });
                                    }
                                    match model_references.iter_mut().find(|r| r.0 == value) {
                                        Some(r) => r.1.push(name.clone()),
                                        None => model_references.push((value, vec![name.clone()])),
                                    }
                                }
                            }
                            "unique" => {
                                field_constraints.push(quote! {
                                    let value = self.#ident.to_string();
                                    if !value.is_empty() {
                                        let columns = [(#name, value.into())];
                                        if !self.is_unique_on(columns).await? {
                                            validation.record(#name, "it should be unique");
                                        }
                                    }
                                });
                            }
                            "not_null" if is_readable => {
                                if type_name == "String" {
                                    field_constraints.push(quote! {
                                        if self.#ident.is_empty() {
                                            validation.record(#name, "it should be nonempty");
                                        }
                                    });
                                } else if type_name == "Uuid" {
                                    field_constraints.push(quote! {
                                        if self.#ident.is_nil() {
                                            validation.record(#name, "it should not be nil");
                                        }
                                    });
                                }
                            }
                            "length" => {
                                let length: usize = value
                                    .and_then(|s| s.parse().ok())
                                    .unwrap_or_default();
                                if type_name == "String" || type_name.starts_with("Vec") {
                                    field_constraints.push(quote! {
                                        let length = #length;
                                        if self.#ident.len() != length {
                                            let message = format!("the length should be {length}");
                                            validation.record(#name, message);
                                        }
                                    });
                                } else if type_name == "Option<String>" {
                                    field_constraints.push(quote! {
                                        let length = #length;
                                        if let Some(ref s) = self.#ident && s.len() != length {
                                            let message = format!("the length should be {length}");
                                            validation.record(#name, message);
                                        }
                                    });
                                }
                            }
                            "max_length" => {
                                let length: usize = value
                                    .and_then(|s| s.parse().ok())
                                    .unwrap_or_default();
                                if type_name == "String" || type_name.starts_with("Vec") {
                                    field_constraints.push(quote! {
                                        let length = #length;
                                        if self.#ident.len() > length {
                                            let message = format!("the length should be at most {length}");
                                            validation.record(#name, message);
                                        }
                                    });
                                } else if type_name == "Option<String>" {
                                    field_constraints.push(quote! {
                                        let length = #length;
                                        if let Some(ref s) = self.#ident && s.len() > length {
                                            let message = format!("the length should be at most {length}");
                                            validation.record(#name, message);
                                        }
                                    });
                                }
                            }
                            "min_length" => {
                                let length: usize = value
                                    .and_then(|s| s.parse().ok())
                                    .unwrap_or_default();
                                if type_name == "String" || type_name.starts_with("Vec") {
                                    field_constraints.push(quote! {
                                        let length = #length;
                                        if self.#ident.len() < length {
                                            let message = format!("the length should be at least {length}");
                                            validation.record(#name, message);
                                        }
                                    });
                                } else if type_name == "Option<String>" {
                                    field_constraints.push(quote! {
                                        let length = #length;
                                        if let Some(ref s) = self.#ident && s.len() < length {
                                            let message = format!("the length should be at least {length}");
                                            validation.record(#name, message);
                                        }
                                    });
                                }
                            }
                            _ => (),
                        }
                    }
                }
                if primary_key_name == name {
                    primary_key_type = type_name;
                } else {
                    let name_ident = format_ident!("{}", name);
                    let mut snapshot_field = None;
                    match name.as_str() {
                        "name" if type_name == "String" => {
                            let method = quote! {
                                #[inline]
                                fn #name_ident(&self) -> &str {
                                    &self.#name_ident
                                }
                            };
                            column_methods.push(method);
                            snapshot_field = Some("name");
                        }
                        "namespace" | "visibility" | "description" if type_name == "String" => {
                            let method = quote! {
                                #[inline]
                                fn #name_ident(&self) -> &str {
                                    &self.#name_ident
                                }
                            };
                            column_methods.push(method);
                        }
                        "status" if type_name == "String" => {
                            let method = quote! {
                                #[inline]
                                fn #name_ident(&self) -> &str {
                                    &self.#name_ident
                                }
                            };
                            column_methods.push(method);
                            snapshot_field = Some("status");
                        }
                        "content" | "extra" if type_name == "Map" => {
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
                            let user_type_opt = type_name.strip_prefix("Option");
                            let user_type = if let Some(user_type) = user_type_opt {
                                user_type.trim_matches(|c| c == '<' || c == '>').to_owned()
                            } else {
                                type_name.clone()
                            };
                            let user_type_ident = format_ident!("{}", user_type);
                            let method = if user_type_opt.is_some() {
                                quote! {
                                    #[inline]
                                    fn #name_ident(&self) -> Option<&#user_type_ident> {
                                        self.#name_ident.as_ref()
                                    }
                                }
                            } else {
                                quote! {
                                    #[inline]
                                    fn #name_ident(&self) -> Option<&#user_type_ident> {
                                        let id = &self.#name_ident;
                                        (id != &#user_type_ident::default()).then_some(id)
                                    }
                                }
                            };
                            column_methods.push(method);
                            user_id_type = user_type;
                        }
                        "created_at" if type_name == "DateTime" => {
                            let method = quote! {
                                #[inline]
                                fn #name_ident(&self) -> DateTime {
                                    self.#name_ident
                                }
                            };
                            column_methods.push(method);
                        }
                        "updated_at" if type_name == "DateTime" => {
                            let method = quote! {
                                #[inline]
                                fn #name_ident(&self) -> DateTime {
                                    self.#name_ident
                                }
                            };
                            column_methods.push(method);
                            snapshot_field = Some("updated_at");
                        }
                        "version" if type_name == "u64" => {
                            let method = quote! {
                                #[inline]
                                fn #name_ident(&self) -> u64 {
                                    self.#name_ident
                                }
                            };
                            column_methods.push(method);
                            snapshot_field = Some("version");
                        }
                        "edition" if type_name == "u32" => {
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
                    if let Some(field) = snapshot_field {
                        let field_ident = format_ident!("{}", field);
                        snapshot_entries.push(quote! {
                            snapshot.upsert(#field, self.#field_ident.clone());
                        });
                        snapshot_fields.push(field.to_owned());
                    }
                }
            }
        }
        if model_references.is_empty() {
            populated_queries.push(quote! {
                let mut models = Self::find::<Map>(query).await?;
                for model in models.iter_mut() {
                    Self::after_decode(model).await?;
                    translate_enabled.then(|| Self::translate_model(model));
                }
            });
            populated_one_queries.push(quote! {
                let mut model = Self::find_by_id::<Map>(id)
                    .await?
                    .ok_or_else(|| ZinoError::new(format!("404 Not Found: cannot find the model `{id}`")))?;
                Self::after_decode(&mut model).await?;
                Self::translate_model(&mut model);
            });
        } else {
            populated_queries.push(quote! {
                let mut models = Self::find::<Map>(query).await?;
                for model in models.iter_mut() {
                    Self::after_decode(model).await?;
                    translate_enabled.then(|| Self::translate_model(model));
                }
            });
            populated_one_queries.push(quote! {
                let mut model = Self::find_by_id::<Map>(id)
                    .await?
                    .ok_or_else(|| ZinoError::new(format!("404 Not Found: cannot find the model `{id}`")))?;
                Self::after_decode(&mut model).await?;
            });
            for (model, fields) in model_references.into_iter() {
                let model_ident = format_ident!("{}", model);
                let populated_query = quote! {
                    let mut query = #model_ident::default_snapshot_query();
                    query.add_filter("translate", translate_enabled);
                    #model_ident::populate(&mut query, &mut models, [#(#fields),*]).await?;
                };
                let populated_one_query = quote! {
                    let mut query = #model_ident::default_query();
                    query.add_filter("translate", true);
                    #model_ident::populate_one(&mut query, &mut model, [#(#fields),*]).await?;
                };
                populated_queries.push(populated_query);
                populated_one_queries.push(populated_one_query);
            }
        }
        populated_queries.push(quote! { Ok(models) });
        populated_one_queries.push(quote! { Ok(model) });
    }

    // Output
    let model_primary_key_type = format_ident!("{}", primary_key_type);
    let model_primary_key = format_ident!("{}", primary_key_name);
    let model_user_id_type = format_ident!("{}", user_id_type);
    let output = quote! {
        use zino_core::{
            database::{ModelAccessor, ModelHelper as _},
            model::Query,
            request::Validation as ZinoValidation,
            Map as ZinoMap,
        };

        impl ModelAccessor<#model_primary_key_type, #model_user_id_type> for #name {
            #[inline]
            fn id(&self) -> &#model_primary_key_type {
                &self.#model_primary_key
            }

            #(#column_methods)*

            fn snapshot(&self) -> Map {
                let mut snapshot = Map::new();
                snapshot.upsert(Self::PRIMARY_KEY_NAME, self.id().to_string());
                #(#snapshot_entries)*
                snapshot
            }

            fn default_snapshot_query() -> Query {
                let mut query = Self::default_query();
                let fields = [
                    Self::PRIMARY_KEY_NAME,
                    #(#snapshot_fields),*
                ];
                query.allow_fields(&fields);
                query
            }

            async fn check_constraints(&self) -> Result<ZinoValidation, ZinoError> {
                let mut validation = ZinoValidation::new();
                if self.id() == &<#model_primary_key_type>::default() {
                    validation.record(Self::PRIMARY_KEY_NAME, "should not be a default value");
                }
                #(#compound_constraints)*
                #(#field_constraints)*
                Ok(validation)
            }

            async fn fetch(query: &mut Query) -> Result<Vec<ZinoMap>, ZinoError> {
                let translate_enabled = query.translate_enabled();
                #(#populated_queries)*
            }

            async fn fetch_by_id(id: &#model_primary_key_type) -> Result<ZinoMap, ZinoError> {
                #(#populated_one_queries)*
            }
        }
    };

    TokenStream::from(output)
}
