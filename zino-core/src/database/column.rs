use crate::Record;
use apache_avro::{
    schema::{Name, Schema},
    types::Value,
};
use serde::Serialize;
use serde_json::Value as JsonValue;
use sqlx::{Database, Error, Row};

/// A model field with associated metadata.
#[derive(Debug, Clone, Serialize)]
pub struct Column<'a> {
    /// Column name.
    name: &'a str,
    /// Column type name.
    type_name: &'a str,
    /// A str representation of the default value.
    default_value: Option<&'a str>,
    /// `NOT NULL` constraint.
    not_null: bool,
    /// Index type.
    index_type: Option<&'a str>,
}

impl<'a> Column<'a> {
    /// Creates a new instance.
    pub const fn new(
        name: &'a str,
        type_name: &'a str,
        default_value: Option<&'a str>,
        not_null: bool,
        index_type: Option<&'a str>,
    ) -> Self {
        Self {
            name,
            type_name,
            default_value,
            not_null,
            index_type,
        }
    }

    /// Returns the name.
    #[inline]
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Returns the type name.
    #[inline]
    pub fn type_name(&self) -> &'a str {
        self.type_name
    }

    /// Returns the default value.
    #[inline]
    pub fn default_value(&self) -> Option<&'a str> {
        self.default_value
    }

    /// Returns `true` if the column can not be null.
    #[inline]
    pub fn is_not_null(&self) -> bool {
        self.not_null
    }

    /// Returns the index type.
    #[inline]
    pub fn index_type(&self) -> Option<&'a str> {
        self.index_type
    }

    /// Returns the [Avro schema](apache_avro::schema::Schema).
    pub fn schema(&self) -> Schema {
        let type_name = self.type_name;
        match type_name {
            "bool" => Schema::Boolean,
            "i32" | "u32" | "i16" | "u16" => Schema::Int,
            "i64" | "u64" => Schema::Long,
            "f32" => Schema::Float,
            "f64" => Schema::Double,
            "String" => Schema::String,
            "DateTime" => Schema::TimestampMicros,
            "Uuid" => Schema::Uuid,
            "Vec<u8>" => Schema::Bytes,
            "Vec<String>" => Schema::Array(Box::new(Schema::String)),
            "Vec<Uuid>" => Schema::Array(Box::new(Schema::Uuid)),
            "Map" => Schema::Map(Box::new(Schema::Ref {
                name: Name {
                    name: "Json".to_owned(),
                    namespace: None,
                },
            })),
            _ => Schema::Ref {
                name: Name {
                    name: type_name.to_owned(),
                    namespace: None,
                },
            },
        }
    }
}

/// Extension trait for [`Column`](crate::database::Column).
pub(super) trait ColumnExt<DB: Database> {
    /// A database row type.
    type Row: Row;

    /// Returns the corresponding column type for the database.
    fn column_type(&self) -> &str;

    /// Encodes a json value as a column value represented by `String`.
    fn encode_value(&self, value: Option<&JsonValue>) -> String;

    /// Decodes a row and gets a column value represented by `Value`.
    fn decode_row(&self, row: &Self::Row) -> Result<Value, Error>;

    /// Parses a row as a json object.
    fn parse_row(row: &Self::Row) -> Result<Record, Error>;

    /// Formats a value.
    fn format_value(&self, value: &str) -> String;

    /// Formats a column filter.
    fn format_filter(&self, key: &str, value: &JsonValue) -> String;

    /// Formats a string.
    #[inline]
    fn format_string(value: &str) -> String {
        format!("'{}'", value.replace('\'', "''"))
    }
}
