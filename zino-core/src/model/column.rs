use super::Reference;
use crate::JsonValue;
use apache_avro::schema::{Name, RecordField, RecordFieldOrder, Schema};
use serde::Serialize;
use std::{borrow::Cow, collections::BTreeMap};

/// A model field with associated metadata.
#[derive(Debug, Clone, Serialize)]
pub struct Column<'a> {
    /// Column name.
    name: &'a str,
    /// Column type name.
    type_name: &'a str,
    /// A flag which indicates whether the column is not null.
    not_null: bool,
    /// A str representation of the default value.
    #[serde(skip_serializing_if = "Option::is_none")]
    default_value: Option<&'a str>,
    /// Index type.
    #[serde(skip_serializing_if = "Option::is_none")]
    index_type: Option<&'a str>,
    /// Reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<Reference<'a>>,
}

impl<'a> Column<'a> {
    /// Creates a new instance.
    pub fn new(name: &'a str, type_name: &'a str, not_null: bool) -> Self {
        Self {
            name,
            type_name,
            not_null,
            default_value: None,
            index_type: None,
            reference: None,
        }
    }

    /// Sets the default value.
    #[inline]
    pub fn set_default_value(&mut self, default_value: &'a str) {
        self.default_value = (!default_value.is_empty()).then_some(default_value);
    }

    /// Sets the index type.
    #[inline]
    pub fn set_index_type(&mut self, index_type: &'a str) {
        self.index_type = (!index_type.is_empty()).then_some(index_type);
    }

    /// Sets the reference.
    #[inline]
    pub fn set_reference(&mut self, reference: Reference<'a>) {
        self.reference = Some(reference);
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

    /// Returns `true` if the column can not be null.
    #[inline]
    pub fn is_not_null(&self) -> bool {
        self.not_null
    }

    /// Returns `true` if the column has an `auto_increment` default.
    #[inline]
    pub fn auto_increment(&self) -> bool {
        self.default_value
            .is_some_and(|value| value == "auto_increment")
    }

    /// Returns the default value.
    #[inline]
    pub fn default_value(&self) -> Option<&'a str> {
        self.default_value
    }

    /// Returns the index type.
    #[inline]
    pub fn index_type(&self) -> Option<&'a str> {
        self.index_type
    }

    /// Returns the reference.
    #[inline]
    pub fn reference(&self) -> Option<&Reference<'a>> {
        self.reference.as_ref()
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
            "String" | "Option<String>" => Schema::String,
            "DateTime" => Schema::TimestampMicros,
            "Uuid" | "Option<Uuid>" => Schema::Uuid,
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

    /// Returns a field for the record Avro schema.
    pub fn record_field(&self) -> RecordField {
        let schema = self.schema();
        let default_value = self.default_value().and_then(|s| match schema {
            Schema::Boolean => s.parse::<bool>().ok().map(|b| b.into()),
            Schema::Int => s.parse::<i32>().ok().map(|i| i.into()),
            Schema::Long => s.parse::<i64>().ok().map(|i| i.into()),
            Schema::Float => s.parse::<f32>().ok().map(|f| f.into()),
            Schema::Double => s.parse::<f64>().ok().map(|f| f.into()),
            _ => Some(s.into()),
        });
        RecordField {
            name: self.name().to_owned(),
            doc: None,
            aliases: None,
            default: default_value,
            schema,
            order: RecordFieldOrder::Ascending,
            position: 0,
            custom_attributes: BTreeMap::new(),
        }
    }
}

/// Encodes the column to be sent to the database.
pub trait EncodeColumn<DB> {
    /// Returns the corresponding column type in the database.
    fn column_type(&self) -> &str;

    /// Encodes a json value as a column value represented by a str.
    fn encode_value<'a>(&self, value: Option<&'a JsonValue>) -> Cow<'a, str>;

    /// Formats a string value for the column.
    fn format_value<'a>(&self, value: &'a str) -> Cow<'a, str>;

    /// Formats a column filter.
    fn format_filter(&self, key: &str, value: &JsonValue) -> String;
}
