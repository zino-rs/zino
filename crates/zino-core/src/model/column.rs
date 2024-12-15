use super::Reference;
use crate::{
    datetime::{Date, DateTime, Time},
    extension::{JsonObjectExt, JsonValueExt},
    mock, Decimal, JsonValue, Map, Uuid,
};
use apache_avro::schema::{
    ArraySchema, MapSchema, Name, RecordField, RecordFieldOrder, Schema, UnionSchema,
};
use rand::{
    distributions::{Alphanumeric, DistString, Distribution, Standard},
    random,
    seq::SliceRandom,
    thread_rng, Rng,
};
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

    /// Appends the extra attributes.
    #[inline]
    pub fn append_extra_attributes(&mut self, attrs: &mut Map) {
        self.extra.append(attrs);
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

    /// Returns `true` if the column has an `auto_random` default.
    #[inline]
    pub fn auto_random(&self) -> bool {
        self.default_value
            .is_some_and(|value| value == "auto_random")
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

    /// Returns `true` if the column has any of the specific attributes.
    pub fn has_any_attributes(&self, attributes: &[&str]) -> bool {
        for attribute in attributes {
            if self.has_attribute(attribute) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if the column has all of the specific attributes.
    pub fn has_all_attributes(&self, attributes: &[&str]) -> bool {
        for attribute in attributes {
            if !self.has_attribute(attribute) {
                return false;
            }
        }
        true
    }

    /// Returns `true` if the column is a primary key.
    #[inline]
    pub fn is_primary_key(&self) -> bool {
        self.has_attribute("primary_key")
    }

    /// Returns `true` if the column is read-only.
    #[inline]
    pub fn is_read_only(&self) -> bool {
        self.has_attribute("read_only")
    }

    /// Returns `true` if the column is write-only.
    #[inline]
    pub fn is_write_only(&self) -> bool {
        self.has_attribute("write_only")
    }

    /// Returns `true` if the column is an option type.
    ///
    /// Only supports `Option<Uuid>` | `Option<String>` | `Option<i64>` | `Option<u64>`
    /// | `Vec<i32>` | `Vec<u32>`.
    #[inline]
    pub fn is_option_type(&self) -> bool {
        matches!(
            self.type_name(),
            "Option<Uuid>"
                | "Option<String>"
                | "Option<i64>"
                | "Option<u64>"
                | "Option<i32>"
                | "Option<u32>"
        )
    }

    /// Returns `true` if the column is an array type.
    ///
    /// Only supports `Vec<Uuid>` | `Vec<String>` | `Vec<i64>` | `Vec<u64>`
    /// | `Vec<i32>` | `Vec<u32>`.
    #[inline]
    pub fn is_array_type(&self) -> bool {
        matches!(
            self.type_name(),
            "Vec<Uuid>" | "Vec<String>" | "Vec<i64>" | "Vec<u64>" | "Vec<i32>" | "Vec<u32>"
        )
    }

    /// Returns `true` if the column has a type of `DateTime`, `Date`, `Time`,
    /// or `String` with a format `date-time`, `date`, `time`.
    pub fn is_datetime_type(&self) -> bool {
        match self.type_name() {
            "DateTime" | "Date" | "Time" | "NaiveDateTime" | "NaiveDate" | "NaiveTime" => true,
            "String" => {
                if let Some(format) = self.extra.get_str("format") {
                    matches!(format, "date-time" | "date" | "time")
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Returns `true` if the column supports fuzzy search.
    #[inline]
    pub fn fuzzy_search(&self) -> bool {
        self.index_type() == Some("text") || self.has_attribute("fuzzy_search")
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
            "Vec<String>" => Schema::Array(ArraySchema {
                items: Box::new(Schema::String),
                attributes: BTreeMap::new(),
            }),
            "Vec<Uuid>" => Schema::Array(ArraySchema {
                items: Box::new(Schema::Uuid),
                attributes: BTreeMap::new(),
            }),
            "Vec<i64>" | "Vec<u64>" => Schema::Array(ArraySchema {
                items: Box::new(Schema::Long),
                attributes: BTreeMap::new(),
            }),
            "Vec<i32>" | "Vec<u32>" => Schema::Array(ArraySchema {
                items: Box::new(Schema::Int),
                attributes: BTreeMap::new(),
            }),
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
            "Map" => Schema::Map(MapSchema {
                types: Box::new(Schema::Ref {
                    name: Name {
                        name: "Json".to_owned(),
                        namespace: None,
                    },
                }),
                attributes: BTreeMap::new(),
            }),
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
            "Decimal" => {
                definition.upsert("type", "number");
                definition.upsert("format", "double");
                if let Some(scale) = extra.get_u32("scale") {
                    definition.upsert("multipleOf", Decimal::new(1, scale).to_string());
                }
            }
            "String" | "Option<String>" => {
                definition.upsert("type", "string");
                if name == "password" {
                    definition.upsert("format", "password");
                }
            }
            "Date" | "NaiveDate" => {
                definition.upsert("type", "string");
                definition.upsert("format", "date");
            }
            "Time" | "NaiveTime" => {
                definition.upsert("type", "string");
                definition.upsert("format", "time");
            }
            "DateTime" | "NaiveDateTime" => {
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
                let mut items = Map::new();
                items.upsert("type", "string");
                items.upsert("format", "uuid");
                definition.upsert("type", "array");
                definition.upsert("items", items);
            }
            "Vec<i64>" => {
                let mut items = Map::new();
                items.upsert("type", "integer");
                items.upsert("format", "int64");
                definition.upsert("type", "array");
                definition.upsert("items", items);
            }
            "Vec<u64>" => {
                let mut items = Map::new();
                items.upsert("type", "integer");
                items.upsert("format", "uint64");
                definition.upsert("type", "array");
                definition.upsert("items", items);
            }
            "Vec<i32>" => {
                let mut items = Map::new();
                items.upsert("type", "integer");
                items.upsert("format", "int32");
                definition.upsert("type", "array");
                definition.upsert("items", items);
            }
            "Vec<u32>" => {
                let mut items = Map::new();
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
        if self.is_read_only() {
            definition.upsert("readOnly", true);
        }
        if self.is_write_only() {
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
            if self.is_array_type() {
                let values = value.parse_str_array();
                definition.upsert("example", values);
            } else {
                definition.upsert("example", value.clone());
            }
        }
        if let Some(values) = extra.parse_enum_values("examples") {
            definition.upsert("examples", values);
        }
        if let Some(values) = extra.parse_enum_values("enum_values") {
            if type_name == "Vec<String>" {
                let mut items = Map::new();
                items.upsert("type", "string");
                items.upsert("enum", values);
                definition.upsert("items", items);
            } else {
                definition.upsert("enum", values);
            }
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

    /// Generates a random size of the items.
    pub fn random_size(&self) -> usize {
        if self.is_array_type() {
            let extra = self.extra();
            let mut min_items = extra.get_usize("min_items").unwrap_or(0);
            if self.has_attribute("nonempty") {
                min_items = min_items.max(1);
            }

            let max_items = extra.get_usize("max_items").unwrap_or(8);
            let mut rng = thread_rng();
            rng.gen_range(min_items..=max_items)
        } else if self.is_option_type() {
            random::<bool>().into()
        } else {
            1
        }
    }

    /// Generates a mocked Json value for the column.
    pub fn mock_value(&self) -> JsonValue {
        if self.reference().is_some() {
            return JsonValue::Null;
        }
        match self.type_name() {
            "bool" => random::<bool>().into(),
            "i8" => self.mock_integer::<i8>(),
            "i16" => self.mock_integer::<i16>(),
            "i32" => self.mock_integer::<i32>(),
            "i64" => self.mock_integer::<i64>(),
            "isize" => self.mock_integer::<isize>(),
            "u8" => self.mock_integer::<u8>(),
            "u16" => self.mock_integer::<u16>(),
            "u32" => self.mock_integer::<u32>(),
            "u64" => self.mock_integer::<u64>(),
            "usize" => self.mock_integer::<usize>(),
            "f32" => random::<f32>().into(),
            "f64" => random::<f64>().into(),
            "String" => self.mock_string(),
            "Date" => Date::today().into(),
            "Time" => Time::now().into(),
            "DateTime" => DateTime::now().into(),
            "Uuid" => Uuid::now_v7().to_string().into(),
            "Option<i32>" => {
                if random::<bool>() {
                    self.mock_integer::<i32>()
                } else {
                    JsonValue::Null
                }
            }
            "Option<i64>" => {
                if random::<bool>() {
                    self.mock_integer::<i64>()
                } else {
                    JsonValue::Null
                }
            }
            "Option<u32>" => {
                if random::<bool>() {
                    self.mock_integer::<u32>()
                } else {
                    JsonValue::Null
                }
            }
            "Option<u64>" => {
                if random::<bool>() {
                    self.mock_integer::<u64>()
                } else {
                    JsonValue::Null
                }
            }
            "Option<String>" => {
                if random::<bool>() {
                    self.mock_string()
                } else {
                    JsonValue::Null
                }
            }
            "Option<Uuid>" => random::<bool>().then(|| Uuid::now_v7().to_string()).into(),
            "Vec<i32>" => self.mock_integer_array::<i32>().into(),
            "Vec<i64>" => self.mock_integer_array::<i64>().into(),
            "Vec<u32>" => self.mock_integer_array::<u32>().into(),
            "Vec<u64>" => self.mock_integer_array::<u64>().into(),
            "Vec<String>" => self.mock_string_array().into(),
            _ => JsonValue::Null,
        }
    }

    /// Generates an integer for the column.
    fn mock_integer<T>(&self) -> JsonValue
    where
        Standard: Distribution<T>,
        T: Into<JsonValue>,
    {
        let extra = self.extra();
        if let Some(values) = extra.parse_enum_values("enum_values") {
            let mut rng = thread_rng();
            values.choose(&mut rng).cloned().into()
        } else {
            random::<T>().into()
        }
    }

    /// Generates a string for the column.
    fn mock_string(&self) -> JsonValue {
        let extra = self.extra();
        if let Some(values) = extra.parse_enum_values("enum_values") {
            let mut rng = thread_rng();
            values.choose(&mut rng).cloned().into()
        } else if let Some(format) = extra.get_str("format") {
            mock::gen_format(format, extra.get_usize("length")).into()
        } else if self.index_type() == Some("hash") {
            let mut rng = thread_rng();
            let min_length = extra.get_usize("min_length").unwrap_or(1);
            let max_length = extra.get_usize("max_length").unwrap_or(16);
            let num_chars = rng.gen_range(min_length..=max_length);
            Alphanumeric.sample_string(&mut rng, num_chars).into()
        } else {
            let locale = extra.get_str("locale").unwrap_or_default();
            let min_length = extra.get_usize("min_length").unwrap_or(1);
            let max_length = extra.get_usize("max_length").unwrap_or(32);
            mock::gen_random_sentence(locale, min_length, max_length).into()
        }
    }

    /// Generates an integer array for the column.
    fn mock_integer_array<T>(&self) -> Vec<JsonValue>
    where
        Standard: Distribution<T>,
        T: Into<JsonValue>,
    {
        let extra = self.extra();
        let mut rng = thread_rng();
        let mut min_items = extra.get_usize("min_items").unwrap_or(0);
        if self.has_attribute("nonempty") {
            min_items = min_items.max(1);
        }
        if let Some(values) = extra.parse_enum_values("enum_values") {
            let max_items = extra.get_usize("max_items").unwrap_or(values.len());
            let num_items = rng.gen_range(min_items..=max_items);
            values
                .choose_multiple(&mut rng, num_items)
                .cloned()
                .collect()
        } else {
            let max_items = extra.get_usize("max_items").unwrap_or(8);
            let num_items = rng.gen_range(min_items..=max_items);
            (0..num_items).map(|_| self.mock_integer::<T>()).collect()
        }
    }

    /// Generates a string array for the column.
    fn mock_string_array(&self) -> Vec<JsonValue> {
        let extra = self.extra();
        let mut rng = thread_rng();
        let mut min_items = extra.get_usize("min_items").unwrap_or(0);
        if self.has_attribute("nonempty") {
            min_items = min_items.max(1);
        }
        if let Some(values) = extra.parse_enum_values("enum_values") {
            let max_items = extra.get_usize("max_items").unwrap_or(values.len());
            let num_items = rng.gen_range(min_items..=max_items);
            values
                .choose_multiple(&mut rng, num_items)
                .cloned()
                .collect()
        } else {
            let max_items = extra.get_usize("max_items").unwrap_or(8);
            let num_items = rng.gen_range(min_items..=max_items);
            (0..num_items).map(|_| self.mock_string()).collect()
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
