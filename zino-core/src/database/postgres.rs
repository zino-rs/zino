use crate::{
    model::{Column, DecodeRow, EncodeColumn},
    Map, Record, Uuid,
};
use apache_avro::types::Value as AvroValue;
use chrono::{DateTime, Local, SecondsFormat};
use serde_json::Value as JsonValue;
use sqlx::{postgres::PgRow, Column as _, Error, Postgres, Row, TypeInfo};
use std::borrow::Cow;

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

    fn encode_value<'b>(column: &Column<'a>, value: Option<&'b JsonValue>) -> Cow<'b, str> {
        if let Some(value) = value {
            match value {
                JsonValue::Null => "NULL".into(),
                JsonValue::Bool(value) => {
                    let value = if *value { "TRUE" } else { "FALSE" };
                    value.into()
                }
                JsonValue::Number(value) => value.to_string().into(),
                JsonValue::String(value) => {
                    if value.is_empty() {
                        if let Some(value) = column.default_value() {
                            Self::format_value(column, value).into_owned().into()
                        } else {
                            "''".into()
                        }
                    } else if value == "null" {
                        "NULL".into()
                    } else {
                        Self::format_value(column, value)
                    }
                }
                JsonValue::Array(value) => {
                    let values = value
                        .iter()
                        .map(|v| match v {
                            JsonValue::String(v) => Self::format_string(v),
                            _ => Self::encode_value(column, Some(v)).into_owned(),
                        })
                        .collect::<Vec<_>>();
                    format!("ARRAY[{}]::{}", values.join(","), Self::column_type(column)).into()
                }
                JsonValue::Object(_) => {
                    format!("'{}'::{}", value, Self::column_type(column)).into()
                }
            }
        } else if column.default_value().is_some() {
            "DEFAULT".into()
        } else {
            "NULL".into()
        }
    }

    fn format_value<'b>(column: &Column<'a>, value: &'b str) -> Cow<'b, str> {
        match column.type_name() {
            "bool" => {
                let value = if value == "true" { "TRUE" } else { "FALSE" };
                value.into()
            }
            "u64" | "u32" | "u16" | "u8" | "usize" => {
                if value.parse::<u64>().is_ok() {
                    value.into()
                } else {
                    "NULL".into()
                }
            }
            "i64" | "i32" | "i16" | "i8" | "isize" => {
                if value.parse::<i64>().is_ok() {
                    value.into()
                } else {
                    "NULL".into()
                }
            }
            "f64" | "f32" => {
                if value.parse::<f64>().is_ok() {
                    value.into()
                } else {
                    "NULL".into()
                }
            }
            "String" | "Uuid" | "Option<Uuid>" => Self::format_string(value).into(),
            "DateTime" => match value {
                "epoch" => "'epoch'".into(),
                "now" => "now()".into(),
                "today" => "date_trunc('day', now())".into(),
                "tomorrow" => "date_trunc('day', now()) + '1 day'::interval".into(),
                "yesterday" => "date_trunc('day', now()) - '1 day'::interval".into(),
                _ => Self::format_string(value).into(),
            },
            "Vec<u8>" => format!(r"'\x{value}'").into(),
            "Vec<String>" | "Vec<Uuid>" => {
                let column_type = Self::column_type(column);
                if value.contains(',') {
                    let values = value
                        .split(',')
                        .map(Self::format_string)
                        .collect::<Vec<_>>();
                    format!("ARRAY[{}]::{}", values.join(","), column_type).into()
                } else {
                    let value = Self::format_string(value);
                    format!("ARRAY[{value}]::{column_type}").into()
                }
            }
            "Map" => {
                let value = Self::format_string(value);
                format!("{value}::jsonb").into()
            }
            _ => "NULL".into(),
        }
    }

    fn format_filter(column: &Column<'a>, field: &str, value: &serde_json::Value) -> String {
        let type_name = column.type_name();
        if let Some(filter) = value.as_object() {
            if type_name == "Map" {
                let field = Self::format_field(field);
                let value = Self::encode_value(column, Some(value));
                return format!(r#"{field} @> {value}"#);
            } else {
                let mut conditions = Vec::with_capacity(filter.len());
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
                        let field = Self::format_field(field);
                        let value = Self::encode_value(column, Some(value));
                        let condition = format!(r#"array_length({field}, 1) = {value}"#);
                        conditions.push(condition);
                    } else if operator == "IN" || operator == "NOT IN" {
                        if let Some(value) = value.as_array() && !value.is_empty() {
                            let field = Self::format_field(field);
                            let value = value
                                .iter()
                                .map(|v| Self::encode_value(column, Some(v)))
                                .collect::<Vec<_>>()
                                .join(",");
                            let condition = format!(r#"{field} {operator} ({value})"#);
                            conditions.push(condition);
                        }
                    } else {
                        let field = Self::format_field(field);
                        let value = Self::encode_value(column, Some(value));
                        let condition = format!(r#"{field} {operator} {value}"#);
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
                let field = Self::format_field(field);
                let value = Self::encode_value(column, Some(value));
                if value == "TRUE" {
                    format!(r#"{field} IS TRUE"#)
                } else {
                    format!(r#"{field} IS NOT TRUE"#)
                }
            }
            "u64" | "i64" | "u32" | "i32" | "u16" | "i16" | "u8" | "i8" | "usize" | "isize"
            | "f64" | "f32" | "DateTime" => {
                let field = Self::format_field(field);
                if let Some(value) = value.as_str() {
                    if let Some((min_value, max_value)) = value.split_once(',') {
                        let min_value = Self::format_value(column, min_value);
                        let max_value = Self::format_value(column, max_value);
                        format!(r#"{field} >= {min_value} AND {field} < {max_value}"#)
                    } else {
                        let index = value.find(|ch| !"<>=".contains(ch)).unwrap_or(0);
                        if index > 0 {
                            let (operator, value) = value.split_at(index);
                            let value = Self::format_value(column, value);
                            format!(r#"{field} {operator} {value}"#)
                        } else {
                            let value = Self::format_value(column, value);
                            format!(r#"{field} = {value}"#)
                        }
                    }
                } else {
                    let value = Self::encode_value(column, Some(value));
                    format!(r#"{field} = {value}"#)
                }
            }
            "String" => {
                let field = Self::format_field(field);
                if let Some(value) = value.as_str() {
                    if value == "null" {
                        // either NULL or empty
                        format!(r#"({field} = '') IS NOT FALSE"#)
                    } else if value == "notnull" {
                        format!(r#"({field} = '') IS FALSE"#)
                    } else {
                        let index = value.find(|ch| !"!~*".contains(ch)).unwrap_or(0);
                        if index > 0 {
                            let (operator, value) = value.split_at(index);
                            let value = Self::format_string(value);
                            format!(r#"{field} {operator} {value}"#)
                        } else {
                            let value = Self::format_string(value);
                            format!(r#"{field} = {value}"#)
                        }
                    }
                } else {
                    let value = Self::encode_value(column, Some(value));
                    format!(r#"{field} = {value}"#)
                }
            }
            "Uuid" | "Option<Uuid>" => {
                let field = Self::format_field(field);
                if let Some(value) = value.as_str() {
                    if value == "null" {
                        format!(r#"{field} IS NULL"#)
                    } else if value == "notnull" {
                        format!(r#"{field} IS NOT NULL"#)
                    } else if value.contains(',') {
                        let value = value
                            .split(',')
                            .map(Self::format_string)
                            .collect::<Vec<_>>()
                            .join(",");
                        format!(r#"{field} IN ({value})"#)
                    } else {
                        let value = Self::format_string(value);
                        format!(r#"{field} = {value}"#)
                    }
                } else {
                    let value = Self::encode_value(column, Some(value));
                    format!(r#"{field} = {value}"#)
                }
            }
            "Vec<String>" | "Vec<Uuid>" => {
                let field = Self::format_field(field);
                if let Some(value) = value.as_str() {
                    if value.contains(';') {
                        if value.contains(',') {
                            value
                                .split(',')
                                .map(|v| {
                                    let s = v.replace(';', ",");
                                    let value = Self::format_value(column, &s);
                                    format!(r#"{field} @> {value}"#)
                                })
                                .collect::<Vec<_>>()
                                .join(" OR ")
                        } else {
                            let s = value.replace(';', ",");
                            let value = Self::format_value(column, &s);
                            format!(r#"{field} @> {value}"#)
                        }
                    } else {
                        let value = Self::format_value(column, value);
                        format!(r#"{field} && {value}"#)
                    }
                } else {
                    let value = Self::encode_value(column, Some(value));
                    format!(r#"{field} && {value}"#)
                }
            }
            "Map" => {
                let field = Self::format_field(field);
                if let Some(value) = value.as_str() {
                    // JSON path operator is supported in Postgres 12+
                    let value = Self::format_string(value);
                    format!(r#"{field} @@ {value}"#)
                } else {
                    let value = Self::encode_value(column, Some(value));
                    format!(r#"{field} @> {value}"#)
                }
            }
            _ => {
                let field = Self::format_field(field);
                let value = Self::encode_value(column, Some(value));
                format!(r#"{field} = {value}"#)
            }
        }
    }

    fn format_field(field: &str) -> String {
        if field.contains('.') {
            field
                .split('.')
                .map(|s| format!(r#""{s}""#))
                .collect::<Vec<_>>()
                .join(".")
        } else {
            format!(r#""{field}""#)
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
