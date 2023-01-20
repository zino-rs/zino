use crate::{Map, Uuid};
use chrono::{DateTime, Local, SecondsFormat};
use serde::Serialize;
use serde_json::Value;
use sqlx::{postgres::PgRow, Column as _, Error, Row, TypeInfo};

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

    /// Returns the corresponding Postgres type.
    pub(crate) fn postgres_type(&self) -> &'a str {
        let type_name = self.type_name;
        match type_name {
            "u64" | "i64" => "bigint",
            "u32" | "i32" => "int",
            "u16" | "i16" => "smallint",
            "f64" => "double precision",
            "f32" => "real",
            "String" => "text",
            "DateTime" => "timestamptz",
            "Uuid" | "Option<Uuid>" => "uuid",
            "Vec<u8>" => "bytea",
            "Vec<String>" => "text[]",
            "Vec<Uuid>" => "uuid[]",
            "Map" => "jsonb",
            _ => type_name,
        }
    }

    /// Encodes a json value as a Postgres value represented by `String`.
    pub(crate) fn encode_postgres_value<'q>(&self, value: impl Into<Option<&'q Value>>) -> String {
        match value.into() {
            Some(value) => match value {
                Value::Null => "NULL".to_owned(),
                Value::Bool(value) => {
                    let value = if *value { "TRUE" } else { "FALSE" };
                    value.to_owned()
                }
                Value::Number(value) => value.to_string(),
                Value::String(value) => {
                    if value.is_empty() {
                        match self.default_value {
                            Some(value) => self.format_postgres_value(value),
                            None => "''".to_owned(),
                        }
                    } else if value == "null" {
                        "NULL".to_owned()
                    } else {
                        self.format_postgres_value(value)
                    }
                }
                Value::Array(value) => {
                    let values = value
                        .iter()
                        .map(|v| match v {
                            Value::String(v) => Self::format_postgres_string(v),
                            _ => self.encode_postgres_value(v),
                        })
                        .collect::<Vec<_>>();
                    format!("ARRAY[{}]::{}", values.join(","), self.postgres_type())
                }
                Value::Object(_) => format!("'{}'::{}", value, self.postgres_type()),
            },
            None => match self.default_value {
                Some(_) => "DEFAULT".to_owned(),
                None => "NULL".to_owned(),
            },
        }
    }

    /// Decodes a Postgres row and gets a column value represented by `Value`.
    pub(crate) fn decode_postgres_row(&self, row: &PgRow) -> Result<Value, Error> {
        let key = self.name;
        let value = match self.type_name {
            "u64" | "i64" => row.try_get_unchecked::<i64, _>(key)?.into(),
            "u32" | "i32" => row.try_get_unchecked::<i32, _>(key)?.into(),
            "u16" | "i16" => row.try_get_unchecked::<i16, _>(key)?.into(),
            "f64" => row.try_get_unchecked::<f64, _>(key)?.into(),
            "f32" => row.try_get_unchecked::<f32, _>(key)?.into(),
            "bool" => row.try_get_unchecked::<bool, _>(key)?.into(),
            "String" => row.try_get_unchecked::<String, _>(key)?.into(),
            "DateTime" => {
                let datetime = row.try_get_unchecked::<DateTime<Local>, _>(key)?;
                datetime
                    .to_rfc3339_opts(SecondsFormat::Micros, false)
                    .into()
            }
            "Uuid" | "Option<Uuid>" => row.try_get_unchecked::<Uuid, _>(key)?.to_string().into(),
            "Vec<u8>" => row.try_get_unchecked::<Vec<u8>, _>(key)?.into(),
            "Vec<String>" => row.try_get_unchecked::<Vec<String>, _>(key)?.into(),
            "Vec<Uuid>" => {
                let values = row.try_get_unchecked::<Vec<Uuid>, _>(key)?;
                values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .into()
            }
            "Map" => row.try_get_unchecked::<Value, _>(key)?,
            _ => Value::Null,
        };
        Ok(value)
    }

    /// Parses a Postgres row as a json object.
    pub(crate) fn parse_postgres_row(row: &PgRow) -> Result<Map, Error> {
        let columns = row.columns();
        let mut map = Map::with_capacity(columns.len());
        for col in columns {
            let key = col.name();
            let value = match col.type_info().name() {
                "INT8" => row.try_get_unchecked::<i64, _>(key)?.into(),
                "INT4" => row.try_get_unchecked::<i32, _>(key)?.into(),
                "INT2" => row.try_get_unchecked::<i16, _>(key)?.into(),
                "FLOAT8" => row.try_get_unchecked::<f64, _>(key)?.into(),
                "FLOAT4" => row.try_get_unchecked::<f32, _>(key)?.into(),
                "BOOL" => row.try_get_unchecked::<bool, _>(key)?.into(),
                "TEXT" | "VARCHAR" => row.try_get_unchecked::<String, _>(key)?.into(),
                "TIMESTAMPTZ" => {
                    let datetime = row.try_get_unchecked::<DateTime<Local>, _>(key)?;
                    datetime
                        .to_rfc3339_opts(SecondsFormat::Micros, false)
                        .into()
                }
                "UUID" => row.try_get_unchecked::<Uuid, _>(key)?.to_string().into(),
                "BYTEA" => row.try_get_unchecked::<Vec<u8>, _>(key)?.into(),
                "TEXT[]" => row.try_get_unchecked::<Vec<String>, _>(key)?.into(),
                "UUID[]" => {
                    let values = row.try_get_unchecked::<Vec<Uuid>, _>(key)?;
                    values
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .into()
                }
                "JSONB" | "JSON" => row.try_get_unchecked::<Value, _>(key)?,
                _ => Value::Null,
            };
            map.insert(key.to_owned(), value);
        }
        Ok(map)
    }

    /// Formats a Postgres string.
    #[inline]
    pub(crate) fn format_postgres_string(value: &str) -> String {
        format!("'{}'", value.replace('\'', "''"))
    }

    /// Formats a Postgres value.
    pub(crate) fn format_postgres_value(&self, value: &str) -> String {
        match self.type_name {
            "u64" | "u32" | "u16" => {
                let value = if value.parse::<u64>().is_ok() {
                    value
                } else {
                    "NULL"
                };
                value.to_owned()
            }
            "i64" | "i32" | "i16" => {
                let value = if value.parse::<i64>().is_ok() {
                    value
                } else {
                    "NULL"
                };
                value.to_owned()
            }
            "f64" | "f32" => {
                let value = if value.parse::<f64>().is_ok() {
                    value
                } else {
                    "NULL"
                };
                value.to_owned()
            }
            "bool" => {
                let value = if value == "true" { "TRUE" } else { "FALSE" };
                value.to_owned()
            }
            "String" | "DateTime" | "Uuid" | "Option<Uuid>" => Self::format_postgres_string(value),
            "Vec<u8>" => format!("'\\x{value}'"),
            "Vec<String>" | "Vec<Uuid>" => {
                let postgres_type = self.postgres_type();
                if value.contains(',') {
                    let values = value
                        .split(',')
                        .map(Self::format_postgres_string)
                        .collect::<Vec<_>>();
                    format!("ARRAY[{}]::{}", values.join(","), postgres_type)
                } else {
                    let value = Self::format_postgres_string(value);
                    format!("ARRAY[{value}]::{postgres_type}")
                }
            }
            "Map" => {
                let value = Self::format_postgres_string(value);
                format!("{value}::jsonb")
            }
            _ => "NULL".to_owned(),
        }
    }

    /// Formats a column filter.
    pub(crate) fn format_postgres_filter(&self, key: &str, value: &Value) -> String {
        let type_name = self.type_name;
        if let Some(filter) = value.as_object() {
            if type_name == "Map" {
                let value = self.encode_postgres_value(value);
                return format!("{key} @> {value}");
            } else {
                let mut conditions = Vec::new();
                for (name, value) in filter {
                    let operator = match name.as_str() {
                        "$eq" => "=",
                        "$ne" => "<>",
                        "$lt" => "<",
                        "$lte" => "<=",
                        "$gt" => ">",
                        "$gte" => ">=",
                        "$in" => "IN",
                        "$nin" => "NOT IN",
                        "$all" => "@>",
                        "$size" => "array_length",
                        _ => "=",
                    };
                    if operator == "array_length" {
                        let value = self.encode_postgres_value(value);
                        let condition = format!("array_length({key}, 1) = {value}");
                        conditions.push(condition);
                    } else if operator == "IN" || operator == "NOT IN" {
                        if let Some(value) = value.as_array() {
                            if !value.is_empty() {
                                let value = value
                                    .iter()
                                    .map(|v| self.encode_postgres_value(v))
                                    .collect::<Vec<_>>()
                                    .join(",");
                                let condition = format!("{key} {operator} ({value})");
                                conditions.push(condition);
                            }
                        }
                    } else {
                        let value = self.encode_postgres_value(value);
                        let condition = format!("{key} {operator} {value}");
                        conditions.push(condition);
                    }
                }
                if conditions.is_empty() {
                    return String::new();
                } else {
                    return format!("({})", conditions.join(" AND "));
                }
            }
        }
        match type_name {
            "u64" | "i64" | "u32" | "i32" | "u16" | "i16" | "f64" | "f32" | "DateTime" => {
                if let Some(value) = value.as_str() {
                    if let Some((min_value, max_value)) = value.split_once(',') {
                        let min_value = self.format_postgres_value(min_value);
                        let max_value = self.format_postgres_value(max_value);
                        format!("{key} >= {min_value} AND {key} < {max_value}")
                    } else {
                        let index = value.find(|ch| !"<>=".contains(ch)).unwrap_or(0);
                        if index > 0 {
                            let (operator, value) = value.split_at(index);
                            let value = self.format_postgres_value(value);
                            format!("{key} {operator} {value}")
                        } else {
                            let value = self.format_postgres_value(value);
                            format!("{key} = {value}")
                        }
                    }
                } else {
                    let value = self.encode_postgres_value(value);
                    format!("{key} = {value}")
                }
            }
            "bool" => {
                let value = self.encode_postgres_value(value);
                if value == "TRUE" {
                    format!("{key} IS TRUE")
                } else {
                    format!("{key} IS NOT TRUE")
                }
            }
            "String" => {
                if let Some(value) = value.as_str() {
                    if value == "null" {
                        // either NULL or empty
                        format!("({key} = '') IS NOT FALSE")
                    } else if value == "notnull" {
                        format!("({key} = '') IS FALSE")
                    } else {
                        let index = value.find(|ch| !"!~*".contains(ch)).unwrap_or(0);
                        if index > 0 {
                            let (operator, value) = value.split_at(index);
                            let value = Self::format_postgres_string(value);
                            format!("{key} {operator} {value}")
                        } else {
                            let value = Self::format_postgres_string(value);
                            format!("{key} = {value}")
                        }
                    }
                } else {
                    let value = self.encode_postgres_value(value);
                    format!("{key} = {value}")
                }
            }
            "Uuid" | "Option<Uuid>" => {
                if let Some(value) = value.as_str() {
                    if value == "null" {
                        format!("{key} IS NULL")
                    } else if value == "notnull" {
                        format!("{key} IS NOT NULL")
                    } else if value.contains(',') {
                        let value = value
                            .split(',')
                            .map(Self::format_postgres_string)
                            .collect::<Vec<_>>()
                            .join(",");
                        format!("{key} IN ({value})")
                    } else {
                        let value = Self::format_postgres_string(value);
                        format!("{key} = {value}")
                    }
                } else {
                    let value = self.encode_postgres_value(value);
                    format!("{key} = {value}")
                }
            }
            "Vec<String>" | "Vec<Uuid>" => {
                if let Some(value) = value.as_str() {
                    if value.contains(';') {
                        if value.contains(',') {
                            value
                                .split(',')
                                .map(|v| {
                                    let value = self.format_postgres_value(&v.replace(';', ","));
                                    format!("{key} @> {value}")
                                })
                                .collect::<Vec<_>>()
                                .join(" OR ")
                        } else {
                            let value = self.format_postgres_value(&value.replace(';', ","));
                            format!("{key} @> {value}")
                        }
                    } else {
                        let value = self.format_postgres_value(value);
                        format!("{key} && {value}")
                    }
                } else {
                    let value = self.encode_postgres_value(value);
                    format!("{key} && {value}")
                }
            }
            "Map" => {
                if let Some(value) = value.as_str() {
                    // JSON path operator is supported in Postgres 12+
                    let value = Self::format_postgres_string(value);
                    format!("{key} @@ {value}")
                } else {
                    let value = self.encode_postgres_value(value);
                    format!("{key} @> {value}")
                }
            }
            _ => {
                let value = self.encode_postgres_value(value);
                format!("{key} = {value}")
            }
        }
    }
}
