use super::parser;
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields};

// Integer types
const INTEGER_TYPES: [&str; 10] = [
    "u64", "i64", "u32", "i32", "u16", "i16", "u8", "i8", "usize", "isize",
];

// Special attributes
const SPECIAL_ATTRIBUTES: [&str; 9] = [
    "ignore",
    "type_name",
    "not_null",
    "default_value",
    "index_type",
    "reference",
    "comment",
    "less_than",
    "greater_than",
];

// Reserved fields
const RESERVED_FIELDS: [&str; 4] = ["created_at", "updated_at", "version", "edition"];

/// Parses the token stream for the `Schema` trait derivation.
pub(super) fn parse_token_stream(input: DeriveInput) -> TokenStream {
    // Model name
    let name = input.ident;
    let mut model_name = name.to_string();

    // Parsing struct attributes
    let mut reader_name = String::from("main");
    let mut writer_name = String::from("main");
    let mut table_name = None;
    let mut model_comment = None;
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
                    "table_name" => {
                        table_name = Some(value);
                    }
                    "comment" => {
                        model_comment = Some(value);
                    }
                    _ => (),
                }
            }
        }
    }

    // Parsing field attributes
    let mut primary_key_type = String::from("Uuid");
    let mut primary_key_name = String::from("id");
    let mut primary_key_value = None;
    let mut primary_key_column = None;
    let mut columns = Vec::new();
    let mut column_fields = Vec::new();
    let mut read_only_fields = Vec::new();
    let mut write_only_fields = Vec::new();
    if let Data::Struct(data) = input.data {
        if let Fields::Named(fields) = data.fields {
            for field in fields.named.into_iter() {
                let mut type_name = parser::get_type_name(&field.ty);
                if let Some(ident) = field.ident {
                    let name = ident.to_string().trim_start_matches("r#").to_owned();
                    let mut column_name = name.clone();
                    let mut ignore = false;
                    let mut not_null = false;
                    let mut column_type = None;
                    let mut default_value = None;
                    let mut index_type = None;
                    let mut reference = None;
                    let mut comment = None;
                    let mut extra_attributes = Vec::new();
                    'inner: for attr in field.attrs.iter() {
                        let arguments = parser::parse_schema_attr(attr);
                        for (key, value) in arguments.into_iter() {
                            let key = key.as_str();
                            if !SPECIAL_ATTRIBUTES.contains(&key) {
                                let attribute_setter = if let Some(value) = value.as_ref() {
                                    if let Ok(value) = value.parse::<i64>() {
                                        quote! { column.set_extra_attribute(#key, #value); }
                                    } else if let Ok(value) = value.parse::<bool>() {
                                        quote! { column.set_extra_attribute(#key, #value); }
                                    } else {
                                        quote! { column.set_extra_attribute(#key, #value); }
                                    }
                                } else {
                                    quote! { column.set_extra_attribute(#key, true); }
                                };
                                extra_attributes.push(attribute_setter);
                            }
                            if RESERVED_FIELDS.contains(&name.as_str()) {
                                extra_attributes.push(quote! {
                                    column.set_extra_attribute("reserved", true);
                                });
                            }
                            match key {
                                "ignore" => {
                                    ignore = true;
                                    break 'inner;
                                }
                                "type_name" => {
                                    if let Some(value) = value {
                                        type_name = value;
                                    }
                                }
                                "column_name" => {
                                    if let Some(value) = value {
                                        let table_alias = model_name.to_case(Case::Snake);
                                        column_name =
                                            format!("{column_name}:{table_alias}.{value}");
                                    }
                                }
                                "column_type" => {
                                    column_type = value;
                                }
                                "length" if type_name == "String" => {
                                    if let Some(value) = value {
                                        column_type = Some(format!("CHAR({value})"));
                                    }
                                }
                                "max_length" if type_name == "String" => {
                                    if let Some(value) = value {
                                        column_type = Some(format!("VARCHAR({value})"));
                                    }
                                }
                                "not_null" => {
                                    not_null = true;
                                }
                                "default_value" => {
                                    default_value = value;
                                }
                                "auto_increment" => {
                                    default_value = Some("auto_increment".to_owned());
                                }
                                "auto_random" => {
                                    default_value = Some("auto_random".to_owned());
                                }
                                "index_type" => {
                                    index_type = value;
                                }
                                "reference" => {
                                    reference = value;
                                }
                                "comment" => {
                                    comment = value;
                                }
                                "primary_key" => {
                                    primary_key_name.clone_from(&name);
                                }
                                "read_only" => {
                                    read_only_fields.push(quote! { #name });
                                }
                                "write_only" => {
                                    write_only_fields.push(quote! { #name });
                                }
                                "constructor" | "validator" => {
                                    extra_attributes.push(quote! {
                                        column.set_extra_attribute(#key, true);
                                    });
                                }
                                _ => (),
                            }
                        }
                    }
                    if ignore {
                        continue;
                    }
                    if primary_key_name == name {
                        primary_key_type.clone_from(&type_name);
                        not_null = true;
                        extra_attributes.push(quote! {
                            column.set_extra_attribute("primary_key", true);
                        });
                    } else if parser::check_option_type(&type_name) {
                        not_null = false;
                    } else if INTEGER_TYPES.contains(&type_name.as_str()) {
                        default_value = default_value.or_else(|| Some("0".to_owned()));
                    } else if let Some(value) = column_type {
                        extra_attributes.push(quote! {
                            column.set_extra_attribute("column_type", #value);
                        });
                    }
                    let quote_value = if let Some(value) = default_value {
                        if let Some((type_name, type_fn)) = value.split_once("::") {
                            let type_name_ident = format_ident!("{}", type_name);
                            let type_fn_ident = format_ident!("{}", type_fn);
                            extra_attributes.push(quote! {
                                let value = <#type_name_ident>::#type_fn_ident();
                                column.set_extra_attribute("default", value);
                            });
                            quote! { Some(<#type_name_ident>::#type_fn_ident().into()) }
                        } else {
                            extra_attributes.push(quote! {
                                column.set_extra_attribute("default", #value);
                            });
                            quote! { Some(#value) }
                        }
                    } else {
                        quote! { None }
                    };
                    let quote_index = parser::quote_option_string(index_type);
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
                    let quote_comment = parser::quote_option_string(comment);
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
                        if let Some(comment) = #quote_comment {
                            column.set_comment(comment);
                        }
                        #(#extra_attributes)*
                        column
                    }};
                    if primary_key_name == name {
                        let primary_key = if primary_key_type == "Uuid" {
                            quote! { self.primary_key().to_string() }
                        } else {
                            quote! { self.primary_key().clone() }
                        };
                        primary_key_value = Some(primary_key);
                        primary_key_column = Some(column.clone());
                    }
                    columns.push(column);
                    column_fields.push(quote! { #column_name });
                }
            }
        }
    }

    // Output
    let model_name_upper_snake = model_name.to_case(Case::UpperSnake);
    let schema_primary_key_type = format_ident!("{}", primary_key_type);
    let schema_primary_key = format_ident!("{}", primary_key_name);
    let schema_primary_key_column = format_ident!("{}_PRIMARY_KEY_COLUMN", model_name_upper_snake);
    let schema_columns = format_ident!("{}_COLUMNS", model_name_upper_snake);
    let schema_fields = format_ident!("{}_FIELDS", model_name_upper_snake);
    let schema_read_only_fields = format_ident!("{}_READ_ONLY_FIELDS", model_name_upper_snake);
    let schema_write_only_fields = format_ident!("{}_WRITE_ONLY_FIELDS", model_name_upper_snake);
    let schema_reader = format_ident!("{}_READER", model_name_upper_snake);
    let schema_writer = format_ident!("{}_WRITER", model_name_upper_snake);
    let schema_table_name = format_ident!("{}_TABLE_NAME", model_name_upper_snake);
    let schema_model_namespace = format_ident!("{}_MODEL_NAMESPACE", model_name_upper_snake);
    let avro_schema = format_ident!("{}_AVRO_SCHEMA", model_name_upper_snake);
    let num_columns = columns.len();
    let num_read_only_fields = read_only_fields.len();
    let num_write_only_fields = write_only_fields.len();
    let quote_table_name = parser::quote_option_string(table_name);
    let quote_model_comment = parser::quote_option_string(model_comment);
    quote! {
        use zino_core::{
            error::Error as ZinoError,
            model::{schema, Column},
            orm::{self, ConnectionPool, Schema},
        };

        static #avro_schema: zino_core::LazyLock<schema::Schema> = zino_core::LazyLock::new(|| {
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
                doc: #quote_model_comment,
                fields,
                lookup: std::collections::BTreeMap::new(),
                attributes: std::collections::BTreeMap::new(),
            };
            schema::Schema::Record(record_schema)
        });
        static #schema_primary_key_column: zino_core::LazyLock<Column> =
            zino_core::LazyLock::new(|| #primary_key_column);
        static #schema_columns: zino_core::LazyLock<[Column; #num_columns]> =
            zino_core::LazyLock::new(|| [#(#columns),*]);
        static #schema_fields: zino_core::LazyLock<[&str; #num_columns]> =
            zino_core::LazyLock::new(|| [#(#column_fields),*]);
        static #schema_read_only_fields: zino_core::LazyLock<[&str; #num_read_only_fields]> =
            zino_core::LazyLock::new(|| [#(#read_only_fields),*]);
        static #schema_write_only_fields: zino_core::LazyLock<[&str; #num_write_only_fields]> =
            zino_core::LazyLock::new(|| [#(#write_only_fields),*]);
        static #schema_reader: std::sync::OnceLock<&ConnectionPool> = std::sync::OnceLock::new();
        static #schema_writer: std::sync::OnceLock<&ConnectionPool> = std::sync::OnceLock::new();
        static #schema_table_name: std::sync::OnceLock<&str> = std::sync::OnceLock::new();
        static #schema_model_namespace: std::sync::OnceLock<&str> = std::sync::OnceLock::new();

        impl Schema for #name {
            type PrimaryKey = #schema_primary_key_type;

            const PRIMARY_KEY_NAME: &'static str = #primary_key_name;
            const READER_NAME: &'static str = #reader_name;
            const WRITER_NAME: &'static str = #writer_name;
            const TABLE_NAME: Option<&'static str> = #quote_table_name;

            #[inline]
            fn primary_key(&self) -> &Self::PrimaryKey {
                &self.#schema_primary_key
            }

            #[inline]
            fn primary_key_value(&self) -> zino_core::JsonValue {
                #primary_key_value.into()
            }

            #[inline]
            fn primary_key_column() -> &'static Column<'static> {
                zino_core::LazyLock::force(&#schema_primary_key_column)
            }

            #[inline]
            fn schema() -> &'static schema::Schema {
                zino_core::LazyLock::force(&#avro_schema)
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
            fn read_only_fields() -> &'static [&'static str] {
                #schema_read_only_fields.as_slice()
            }

            #[inline]
            fn write_only_fields() -> &'static [&'static str] {
                #schema_write_only_fields.as_slice()
            }

            async fn acquire_reader() -> Result<&'static ConnectionPool, ZinoError> {
                use zino_core::{bail, error::Error, orm::PoolManager, warn};

                if let Some(reader) = #schema_reader.get() {
                    if reader.is_available()
                        || reader.is_retryable() && reader.check_availability().await
                    {
                        Ok(*reader)
                    } else if let Ok(connection_pool) = Self::init_reader() {
                        reader.increment_missed_count();
                        Ok(connection_pool)
                    } else {
                        Ok(*reader)
                    }
                } else {
                    let model_name = Self::MODEL_NAME;
                    let connection_pool = Self::init_reader()?;
                    if let Err(err) = Self::create_table().await {
                        connection_pool.store_availability(false);
                        bail!(
                            "503 Service Unavailable: fail to acquire reader for the model `{}`: {}",
                            model_name,
                            err
                        );
                    }
                    if let Err(err) = Self::synchronize_schema().await {
                        connection_pool.store_availability(false);
                        bail!(
                            "503 Service Unavailable: fail to acquire reader for the model `{}`: {}",
                            model_name,
                            err
                        );
                    }
                    if let Err(err) = Self::create_indexes().await {
                        connection_pool.store_availability(false);
                        bail!(
                            "503 Service Unavailable: fail to acquire reader for the model `{}`: {}",
                            model_name,
                            err
                        );
                    }
                    #schema_reader.set(connection_pool).map_err(|_| {
                        warn!(
                            "503 Service Unavailable: fail to acquire reader for the model `{}`",
                            model_name
                        )
                    })?;
                    Ok(connection_pool)
                }
            }

            async fn acquire_writer() -> Result<&'static ConnectionPool, ZinoError> {
                use zino_core::{bail, error::Error, orm::PoolManager, warn};

                if let Some(writer) = #schema_writer.get() {
                    if writer.is_available()
                        || writer.is_retryable() && writer.check_availability().await
                    {
                        Ok(*writer)
                    } else if let Ok(connection_pool) = Self::init_writer() {
                        writer.increment_missed_count();
                        Ok(connection_pool)
                    } else {
                        Ok(*writer)
                    }
                } else {
                    let model_name = Self::MODEL_NAME;
                    let connection_pool = Self::init_writer()?;
                    if let Err(err) = Self::create_table().await {
                        connection_pool.store_availability(false);
                        bail!(
                            "503 Service Unavailable: fail to acquire writer for the model `{}`: {}",
                            model_name,
                            err
                        );
                    }
                    if let Err(err) = Self::synchronize_schema().await {
                        bail!(
                            "503 Service Unavailable: fail to acquire writer for the model `{}`: {}",
                            model_name,
                            err
                        );
                    }
                    if let Err(err) = Self::create_indexes().await {
                        bail!(
                            "503 Service Unavailable: fail to acquire writer for the model `{}`: {}",
                            model_name,
                            err
                        );
                    }
                    #schema_writer.set(connection_pool).map_err(|_| {
                        warn!(
                            "503 Service Unavailable: fail to acquire writer for the model `{}`",
                            model_name
                        )
                    })?;
                    Ok(connection_pool)
                }
            }

            #[inline]
            fn table_name() -> &'static str {
                Self::TABLE_NAME.unwrap_or_else(|| {
                    #schema_table_name
                        .get_or_init(|| [Self::table_prefix(), Self::MODEL_NAME].concat().leak())
                })
            }

            #[inline]
            fn model_namespace() -> &'static str {
                #schema_model_namespace
                    .get_or_init(|| [Self::namespace_prefix(), Self::MODEL_NAME].concat().leak())
            }
        }

        impl PartialEq for #name {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.#schema_primary_key == other.#schema_primary_key
            }
        }

        impl Eq for #name {}
    }
}
