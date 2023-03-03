use crate::{
    model::{Column, DecodeRow, EncodeColumn},
    Map, Record, Uuid,
};
use apache_avro::types::Value as AvroValue;
use chrono::{DateTime, Local, SecondsFormat};
use serde_json::Value as JsonValue;
use sqlx::{postgres::PgRow, Column as _, Error, Postgres, Row, TypeInfo};

impl<'a> EncodeColumn<'a> for Postgres {
    fn column_type(column: &Column<'a>) -> &'a str {
        let type_name = column.type_name();
        match type_name {
            "bool" => "boolean",
            "u64" | "i64" | "usize" | "isize" => "bigint",
            "u32" | "i32" => "int",
            "u16" | "i16" | "u8" | "i8" => "smallint",
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

    fn encode_value(column: &Column<'a>, value: Option<&JsonValue>) -> String {
        if let Some(value) = value {
            match value {
                JsonValue::Null => "NULL".to_owned(),
                JsonValue::Bool(value) => {
                    let value = if *value { "TRUE" } else { "FALSE" };
                    value.to_owned()
                }
                JsonValue::Number(value) => value.to_string(),
                JsonValue::String(value) => {
                    if value.is_empty() {
                        if let Some(value) = column.default_value() {
                            Self::format_value(column, value)
                        } else {
                            "''".to_owned()
                        }
                    } else if value == "null" {
                        "NULL".to_owned()
                    } else {
                        Self::format_value(column, value)
                    }
                }
                JsonValue::Array(value) => {
                    let values = value
                        .iter()
                        .map(|v| match v {
                            JsonValue::String(v) => Self::format_string(v),
                            _ => Self::encode_value(column, Some(v)),
                        })
                        .collect::<Vec<_>>();
                    format!("ARRAY[{}]::{}", values.join(","), Self::column_type(column))
                }
                JsonValue::Object(_) => format!("'{}'::{}", value, Self::column_type(column)),
            }
        } else if column.default_value().is_some() {
            "DEFAULT".to_owned()
        } else {
            "NULL".to_owned()
        }
    }

    fn format_value(column: &Column<'a>, value: &str) -> String {
        match column.type_name() {
            "bool" => {
                let value = if value == "true" { "TRUE" } else { "FALSE" };
                value.to_owned()
            }
            "u64" | "u32" | "u16" | "u8" | "usize" => {
                let value = if value.parse::<u64>().is_ok() {
                    value
                } else {
                    "NULL"
                };
                value.to_owned()
            }
            "i64" | "i32" | "i16" | "i8" | "isize" => {
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
            "String" | "DateTime" | "Uuid" | "Option<Uuid>" => Self::format_string(value),
            "Vec<u8>" => format!("'\\x{value}'"),
            "Vec<String>" | "Vec<Uuid>" => {
                let column_type = Self::column_type(column);
                if value.contains(',') {
                    let values = value
                        .split(',')
                        .map(Self::format_string)
                        .collect::<Vec<_>>();
                    format!("ARRAY[{}]::{}", values.join(","), column_type)
                } else {
                    let value = Self::format_string(value);
                    format!("ARRAY[{value}]::{column_type}")
                }
            }
            "Map" => {
                let value = Self::format_string(value);
                format!("{value}::jsonb")
            }
            _ => "NULL".to_owned(),
        }
    }

    fn format_filter(column: &Column<'a>, field: &str, value: &serde_json::Value) -> String {
        let type_name = column.type_name();
        if let Some(filter) = value.as_object() {
            if type_name == "Map" {
                let value = Self::encode_value(column, Some(value));
                return format!("{field} @> {value}");
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
                        let value = Self::encode_value(column, Some(value));
                        let condition = format!("array_length({field}, 1) = {value}");
                        conditions.push(condition);
                    } else if operator == "IN" || operator == "NOT IN" {
                        if let Some(value) = value.as_array() {
                            if !value.is_empty() {
                                let value = value
                                    .iter()
                                    .map(|v| Self::encode_value(column, Some(v)))
                                    .collect::<Vec<_>>()
                                    .join(",");
                                let condition = format!("{field} {operator} ({value})");
                                conditions.push(condition);
                            }
                        }
                    } else {
                        let value = Self::encode_value(column, Some(value));
                        let condition = format!("{field} {operator} {value}");
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
            "bool" => {
                let value = Self::encode_value(column, Some(value));
                if value == "TRUE" {
                    format!("{field} IS TRUE")
                } else {
                    format!("{field} IS NOT TRUE")
                }
            }
            "u64" | "i64" | "u32" | "i32" | "u16" | "i16" | "u8" | "i8" | "usize" | "isize"
            | "f64" | "f32" | "DateTime" => {
                if let Some(value) = value.as_str() {
                    if let Some((min_value, max_value)) = value.split_once(',') {
                        let min_value = Self::format_value(column, min_value);
                        let max_value = Self::format_value(column, max_value);
                        format!("{field} >= {min_value} AND {field} < {max_value}")
                    } else {
                        let index = value.find(|ch| !"<>=".contains(ch)).unwrap_or(0);
                        if index > 0 {
                            let (operator, value) = value.split_at(index);
                            let value = Self::format_value(column, value);
                            format!("{field} {operator} {value}")
                        } else {
                            let value = Self::format_value(column, value);
                            format!("{field} = {value}")
                        }
                    }
                } else {
                    let value = Self::encode_value(column, Some(value));
                    format!("{field} = {value}")
                }
            }
            "String" => {
                if let Some(value) = value.as_str() {
                    if value == "null" {
                        // either NULL or empty
                        format!("({field} = '') IS NOT FALSE")
                    } else if value == "notnull" {
                        format!("({field} = '') IS FALSE")
                    } else {
                        let index = value.find(|ch| !"!~*".contains(ch)).unwrap_or(0);
                        if index > 0 {
                            let (operator, value) = value.split_at(index);
                            let value = Self::format_string(value);
                            format!("{field} {operator} {value}")
                        } else {
                            let value = Self::format_string(value);
                            format!("{field} = {value}")
                        }
                    }
                } else {
                    let value = Self::encode_value(column, Some(value));
                    format!("{field} = {value}")
                }
            }
            "Uuid" | "Option<Uuid>" => {
                if let Some(value) = value.as_str() {
                    if value == "null" {
                        format!("{field} IS NULL")
                    } else if value == "notnull" {
                        format!("{field} IS NOT NULL")
                    } else if value.contains(',') {
                        let value = value
                            .split(',')
                            .map(Self::format_string)
                            .collect::<Vec<_>>()
                            .join(",");
                        format!("{field} IN ({value})")
                    } else {
                        let value = Self::format_string(value);
                        format!("{field} = {value}")
                    }
                } else {
                    let value = Self::encode_value(column, Some(value));
                    format!("{field} = {value}")
                }
            }
            "Vec<String>" | "Vec<Uuid>" => {
                if let Some(value) = value.as_str() {
                    if value.contains(';') {
                        if value.contains(',') {
                            value
                                .split(',')
                                .map(|v| {
                                    let value = Self::format_value(column, &v.replace(';', ","));
                                    format!("{field} @> {value}")
                                })
                                .collect::<Vec<_>>()
                                .join(" OR ")
                        } else {
                            let value = Self::format_value(column, &value.replace(';', ","));
                            format!("{field} @> {value}")
                        }
                    } else {
                        let value = Self::format_value(column, value);
                        format!("{field} && {value}")
                    }
                } else {
                    let value = Self::encode_value(column, Some(value));
                    format!("{field} && {value}")
                }
            }
            "Map" => {
                if let Some(value) = value.as_str() {
                    // JSON path operator is supported in Postgres 12+
                    let value = Self::format_string(value);
                    format!("{field} @@ {value}")
                } else {
                    let value = Self::encode_value(column, Some(value));
                    format!("{field} @> {value}")
                }
            }
            _ => {
                let value = Self::encode_value(column, Some(value));
                format!("{field} = {value}")
            }
        }
    }

    #[inline]
    fn format_string(value: &str) -> String {
        format!("'{}'", value.replace('\'', "''"))
    }
}

impl DecodeRow<PgRow> for Map {
    type Error = Error;

    fn decode_row(row: &PgRow) -> Result<Self, Self::Error> {
        let columns = row.columns();
        let mut map = Map::with_capacity(columns.len());
        for col in columns {
            let key = col.name();
            let value = match col.type_info().name() {
                "BOOL" => row.try_get_unchecked::<bool, _>(key)?.into(),
                "INT2" => row.try_get_unchecked::<i16, _>(key)?.into(),
                "INT4" => row.try_get_unchecked::<i32, _>(key)?.into(),
                "INT8" => row.try_get_unchecked::<i64, _>(key)?.into(),
                "FLOAT4" => row.try_get_unchecked::<f32, _>(key)?.into(),
                "FLOAT8" => row.try_get_unchecked::<f64, _>(key)?.into(),
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
                "JSONB" | "JSON" => row.try_get_unchecked::<JsonValue, _>(key)?,
                _ => JsonValue::Null,
            };
            map.insert(key.to_owned(), value);
        }
        Ok(map)
    }
}

impl DecodeRow<PgRow> for Record {
    type Error = Error;

    fn decode_row(row: &PgRow) -> Result<Self, Self::Error> {
        let columns = row.columns();
        let mut record = Record::with_capacity(columns.len());
        for col in columns {
            let field = col.name();
            let value = match col.type_info().name() {
                "BOOL" => row.try_get_unchecked::<bool, _>(field)?.into(),
                "INT4" => row.try_get_unchecked::<i32, _>(field)?.into(),
                "INT8" => row.try_get_unchecked::<i64, _>(field)?.into(),
                "FLOAT4" => row.try_get_unchecked::<f32, _>(field)?.into(),
                "FLOAT8" => row.try_get_unchecked::<f64, _>(field)?.into(),
                "TEXT" | "VARCHAR" => row.try_get_unchecked::<String, _>(field)?.into(),
                "TIMESTAMPTZ" => {
                    let datetime = row.try_get_unchecked::<DateTime<Local>, _>(field)?;
                    datetime
                        .to_rfc3339_opts(SecondsFormat::Micros, false)
                        .into()
                }
                // deserialize Avro Uuid value wasn't supported in 0.14.0
                "UUID" => row.try_get_unchecked::<Uuid, _>(field)?.to_string().into(),
                "BYTEA" => row.try_get_unchecked::<Vec<u8>, _>(field)?.into(),
                "TEXT[]" => {
                    let values = row.try_get_unchecked::<Vec<String>, _>(field)?;
                    let vec = values
                        .into_iter()
                        .map(AvroValue::String)
                        .collect::<Vec<_>>();
                    AvroValue::Array(vec)
                }
                "UUID[]" => {
                    // deserialize Avro Uuid value wasn't supported in 0.14.0
                    let values = row.try_get_unchecked::<Vec<Uuid>, _>(field)?;
                    let vec = values
                        .into_iter()
                        .map(|u| AvroValue::String(u.to_string()))
                        .collect::<Vec<_>>();
                    AvroValue::Array(vec)
                }
                "JSONB" | "JSON" => row.try_get_unchecked::<JsonValue, _>(field)?.into(),
                _ => AvroValue::Null,
            };
            record.push((field.to_owned(), value));
        }
        Ok(record)
    }
}
