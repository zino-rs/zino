use crate::Map;
use serde::Serialize;
use serde_json::Value;
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
}

/// Extension trait for column.
pub trait ColumnExt<DB: Database> {
    /// A database row type.
    type Row: Row;

    /// Returns the corresponding column type for the database.
    fn column_type(&self) -> &str;

    /// Encodes a json value as a column value represented by `String`.
    fn encode_value(&self, value: Option<&Value>) -> String;

    /// Decodes a row and gets a column value represented by `Value`.
    fn decode_row(&self, row: &Self::Row) -> Result<Value, Error>;

    /// Formats a value.
    fn format_value(&self, value: &str) -> String;

    /// Formats a column filter.
    fn format_filter(&self, key: &str, value: &Value) -> String;

    /// Formats a string.
    fn format_string(value: &str) -> String;

    /// Parses a row as a json object.
    fn parse_row(row: &Self::Row) -> Result<Map, Error>;
}
