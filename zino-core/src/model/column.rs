use apache_avro::schema::{Name, Schema};
use serde::Serialize;
use serde_json::Value;
use std::borrow::Cow;

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
            "i32" | "u32" | "i16" | "u16" | "i8" | "u8" => Schema::Int,
            "i64" | "u64" | "isize" | "usize" => Schema::Long,
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
                    name: "json".to_owned(),
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

/// A backend type for encoding the column.
pub trait EncodeColumn<'a> {
    /// Driver name.
    const DRIVER_NAME: &'static str;

    /// Returns the corresponding column type.
    fn column_type(column: &Column<'a>) -> &'a str;

    /// Encodes a json value as a column value represented by a str.
    fn encode_value<'b>(column: &Column<'a>, value: Option<&'b Value>) -> Cow<'b, str>;

    /// Formats a string value for the column.
    fn format_value<'b>(column: &Column<'a>, value: &'b str) -> Cow<'b, str>;

    /// Formats a column filter.
    fn format_filter(column: &Column<'a>, key: &str, value: &Value) -> String;

    /// Formats a field.
    fn format_field(field: &str) -> String;

    /// Formats a string.
    fn format_string(value: &str) -> String;
}
