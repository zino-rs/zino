use super::Reference;
use crate::{extension::JsonObjectExt, JsonValue, Map};
use apache_avro::schema::{Name, RecordField, RecordFieldOrder, Schema, UnionSchema};
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
    /// Comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<&'a str>,
    /// Extra attributes.
    extra: Map,
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
            comment: None,
            extra: Map::new(),
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

    /// Sets the comment.
    #[inline]
    pub fn set_comment(&mut self, comment: &'a str) {
        self.comment = (!comment.is_empty()).then_some(comment);
    }

    /// Sets the extra attribute.
    #[inline]
    pub fn set_extra_attribute(&mut self, key: &str, value: impl Into<JsonValue>) {
        self.extra.upsert(key, value);
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

    /// Returns the comment.
    #[inline]
    pub fn comment(&self) -> Option<&'a str> {
        self.comment
    }

    /// Returns a reference to the extra attributes.
    #[inline]
    pub fn extra(&self) -> &Map {
        &self.extra
    }

    /// Returns `true` if the column has the specific attribute.
    #[inline]
    pub fn has_attribute(&self, attribute: &str) -> bool {
        self.extra.contains_key(attribute)
    }

    /// Returns `true` if the user has any of the specific attributes.
    pub fn has_any_attributes(&self, attributes: &[&str]) -> bool {
        for attribute in attributes {
            if self.has_attribute(attribute) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if the user has all of the specific attributes.
    pub fn has_all_attributes(&self, attributes: &[&str]) -> bool {
        for attribute in attributes {
            if !self.has_attribute(attribute) {
                return false;
            }
        }
        true
    }

    /// Returns the Avro schema.
    pub fn schema(&self) -> Schema {
        let type_name = self.type_name();
        match type_name {
            "bool" => Schema::Boolean,
            "i32" | "u32" | "i16" | "u16" | "i8" | "u8" => Schema::Int,
            "i64" | "u64" | "isize" | "usize" => Schema::Long,
            "f32" => Schema::Float,
            "f64" => Schema::Double,
            "String" => Schema::String,
            "Date" => Schema::Date,
            "DateTime" => Schema::TimestampMicros,
            "Uuid" => Schema::Uuid,
            "Vec<u8>" => Schema::Bytes,
            "Vec<String>" => Schema::Array(Box::new(Schema::String)),
            "Vec<Uuid>" => Schema::Array(Box::new(Schema::Uuid)),
            "Vec<i64>" | "Vec<u64>" => Schema::Array(Box::new(Schema::Long)),
            "Vec<i32>" | "Vec<u32>" => Schema::Array(Box::new(Schema::Int)),
            "Option<String>" => {
                if let Ok(union_schema) = UnionSchema::new(vec![Schema::Null, Schema::String]) {
                    Schema::Union(union_schema)
                } else {
                    Schema::String
                }
            }
            "Option<Uuid>" => {
                if let Ok(union_schema) = UnionSchema::new(vec![Schema::Null, Schema::Uuid]) {
                    Schema::Union(union_schema)
                } else {
                    Schema::Uuid
                }
            }
            "Option<i64>" | "Option<u64>" => {
                if let Ok(union_schema) = UnionSchema::new(vec![Schema::Null, Schema::Long]) {
                    Schema::Union(union_schema)
                } else {
                    Schema::Long
                }
            }
            "Option<i32>" | "Option<u32>" => {
                if let Ok(union_schema) = UnionSchema::new(vec![Schema::Null, Schema::Int]) {
                    Schema::Union(union_schema)
                } else {
                    Schema::Int
                }
            }
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
            doc: self.comment().map(|s| s.to_owned()),
            aliases: None,
            default: default_value,
            schema,
            order: RecordFieldOrder::Ascending,
            position: 0,
            custom_attributes: BTreeMap::new(),
        }
    }

    /// Returns the definition to be used in the OpenAPI schema object.
    pub fn definition(&self) -> Map {
        let mut definition = Map::new();
        let name = self.name();
        let type_name = self.type_name();
        let extra = self.extra();
        match type_name {
            "bool" => {
                definition.upsert("type", "boolean");
            }
            "i8" => {
                definition.upsert("type", "integer");
                definition.upsert("format", "int8");
            }
            "i16" => {
                definition.upsert("type", "integer");
                definition.upsert("format", "int16");
            }
            "i32" | "Option<i32>" => {
                definition.upsert("type", "integer");
                definition.upsert("format", "int32");
            }
            "i64" | "Option<i64>" | "isize" => {
                definition.upsert("type", "integer");
                definition.upsert("format", "int64");
            }
            "u8" => {
                definition.upsert("type", "integer");
                definition.upsert("format", "uint8");
            }
            "u16" => {
                definition.upsert("type", "integer");
                definition.upsert("format", "uint16");
            }
            "u32" | "Option<u32>" => {
                definition.upsert("type", "integer");
                definition.upsert("format", "uint32");
            }
            "u64" | "Option<u64>" | "usize" => {
                definition.upsert("type", "integer");
                definition.upsert("format", "uint64");
            }
            "f32" => {
                definition.upsert("type", "number");
                definition.upsert("format", "float");
            }
            "f64" => {
                definition.upsert("type", "number");
                definition.upsert("format", "double");
            }
            "String" | "Option<String>" => {
                definition.upsert("type", "string");
                if name == "password" {
                    definition.upsert("format", "password");
                }
            }
            "Date" => {
                definition.upsert("type", "string");
                definition.upsert("format", "date");
            }
            "DateTime" => {
                definition.upsert("type", "string");
                definition.upsert("format", "date-time");
            }
            "Uuid" | "Option<Uuid>" => {
                definition.upsert("type", "string");
                definition.upsert("format", "uuid");
            }
            "Vec<u8>" => {
                definition.upsert("type", "string");
                definition.upsert("format", "binary");
            }
            "Vec<String>" => {
                let items = Map::from_entry("type", "string");
                definition.upsert("type", "array");
                definition.upsert("items", items);
            }
            "Vec<Uuid>" => {
                let mut items = Map::with_capacity(2);
                items.upsert("type", "string");
                items.upsert("format", "uuid");
                definition.upsert("type", "array");
                definition.upsert("items", items);
            }
            "Vec<i64>" => {
                let mut items = Map::with_capacity(2);
                items.upsert("type", "integer");
                items.upsert("format", "int64");
                definition.upsert("type", "array");
                definition.upsert("items", items);
            }
            "Vec<u64>" => {
                let mut items = Map::with_capacity(2);
                items.upsert("type", "integer");
                items.upsert("format", "uint64");
                definition.upsert("type", "array");
                definition.upsert("items", items);
            }
            "Vec<i32>" => {
                let mut items = Map::with_capacity(2);
                items.upsert("type", "integer");
                items.upsert("format", "int32");
                definition.upsert("type", "array");
                definition.upsert("items", items);
            }
            "Vec<u32>" => {
                let mut items = Map::with_capacity(2);
                items.upsert("type", "integer");
                items.upsert("format", "uint32");
                definition.upsert("type", "array");
                definition.upsert("items", items);
            }
            "Map" => {
                definition.upsert("type", "object");
            }
            _ => {
                definition.upsert("type", type_name);
            }
        };
        if let Some(comment) = self.comment() {
            definition.upsert("description", comment);
        }
        if self.has_attribute("readonly") {
            definition.upsert("readOnly", true);
        }
        if self.has_attribute("writeonly") {
            definition.upsert("writeOnly", true);
        }
        if self.has_attribute("deprecated") {
            definition.upsert("deprecated", true);
        }
        if self.has_attribute("unique_items") {
            definition.upsert("uniqueItems", true);
        }
        if self.has_attribute("nonempty") {
            let key = match definition.get_str("type") {
                Some("array") => "minItems",
                Some("object") => "minProperties",
                _ => "minLength",
            };
            definition.upsert(key, 1);
        }
        if let Some(value) = extra.get_str("title") {
            definition.upsert("title", value);
        }
        if let Some(value) = extra.get_str("description") {
            definition.upsert("description", value);
        }
        if let Some(value) = extra.get_str("format") {
            definition.upsert("format", value);
        }
        if let Some(value) = extra.get_str("pattern") {
            definition.upsert("pattern", value);
        }
        if let Some(value) = extra.get("default") {
            definition.upsert("default", value.clone());
        }
        if let Some(value) = extra.get("example") {
            definition.upsert("example", value.clone());
        }
        if let Some(values) = extra.parse_enum_values("examples") {
            definition.upsert("examples", values);
        }
        if let Some(values) = extra.parse_enum_values("enum_values") {
            definition.upsert("enum", values);
        }
        if let Some(Ok(value)) = extra.parse_usize("max_length") {
            definition.upsert("maxLength", value);
        }
        if let Some(Ok(value)) = extra.parse_usize("min_length") {
            definition.upsert("minLength", value);
        }
        if let Some(Ok(value)) = extra.parse_usize("max_properties") {
            definition.upsert("maxProperties", value);
        }
        if let Some(Ok(value)) = extra.parse_usize("min_properties") {
            definition.upsert("minProperties", value);
        }
        if let Some(Ok(value)) = extra.parse_usize("max_items") {
            definition.upsert("maxItems", value);
        }
        if let Some(Ok(value)) = extra.parse_usize("min_items") {
            definition.upsert("minItems", value);
        }
        if let Some(Ok(value)) = extra.parse_i64("multiple_of") {
            definition.upsert("multipleOf", value);
        }
        if let Some(Ok(value)) = extra.parse_i64("minimum") {
            definition.upsert("minimum", value);
        }
        if let Some(Ok(value)) = extra.parse_i64("maximum") {
            definition.upsert("maximum", value);
        }
        if let Some(Ok(value)) = extra.parse_i64("exclusive_minimum") {
            definition.upsert("exclusiveMinimum", value);
        }
        if let Some(Ok(value)) = extra.parse_i64("exclusive_maximum") {
            definition.upsert("exclusiveMaximum", value);
        }
        definition
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
