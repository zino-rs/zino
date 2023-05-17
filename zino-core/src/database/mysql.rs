use super::{query::QueryExt, DatabaseDriver, DatabaseRow};
use crate::{
    datetime::DateTime,
    model::{Column, DecodeRow, EncodeColumn, Query},
    request::Validation,
    Map, Record,
};
use apache_avro::types::Value as AvroValue;
use serde_json::Value as JsonValue;
use sqlx::{Column as _, Error, Row, TypeInfo};
use std::borrow::Cow;

impl<'a> EncodeColumn<'a> for DatabaseDriver {
    const DRIVER_NAME: &'static str = "mysql";

    fn column_type(column: &Column<'a>) -> &'a str {
        let type_name = column.type_name();
        match type_name {
            "bool" => "BOOLEAN",
            "u64" | "usize" => "BIGINT UNSIGNED",
            "i64" | "isize" => "BIGINT",
            "u32" => "INT UNSIGNED",
            "i32" => "INT",
            "u16" => "SMALLINT UNSIGNED",
            "i16" => "SMALLINT",
            "u8" => "TINYINT UNSIGNED",
            "i8" => "TINYINT",
            "f64" => "DOUBLE",
            "f32" => "FLOAT",
            "String" => {
                if column.default_value().or(column.index_type()).is_some() {
                    "VARCHAR(255)"
                } else {
                    "TEXT"
                }
            }
            "DateTime" => "TIMESTAMP(6)",
            "Uuid" | "Option<Uuid>" => "VARCHAR(36)",
            "Vec<u8>" => "BLOB",
            "Vec<String>" => "JSON",
            "Vec<Uuid>" => "JSON",
            "Map" => "JSON",
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
                    format!(r#"json_array({})"#, values.join(",")).into()
                }
                JsonValue::Object(_) => format!("'{value}'").into(),
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
                "epoch" => "from_unixtime(0)".into(),
                "now" => "current_timestamp(6)".into(),
                "today" => "curdate()".into(),
                "tomorrow" => "curdate() + INTERVAL 1 DAY".into(),
                "yesterday" => "curdate() - INTERVAL 1 DAY".into(),
                _ => Self::format_string(value).into(),
            },
            "Vec<u8>" => format!("'value'").into(),
            "Vec<String>" | "Vec<Uuid>" => {
                if value.contains(',') {
                    let values = value
                        .split(',')
                        .map(Self::format_string)
                        .collect::<Vec<_>>();
                    format!(r#"json_array({})"#, values.join(",")).into()
                } else {
                    let value = Self::format_string(value);
                    format!(r#"json_array({value})"#).into()
                }
            }
            "Map" => {
                let value = Self::format_string(value);
                format!("'{value}'").into()
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
                // `json_overlaps()` was added in MySQL 8.0.17.
                return format!(r#"json_overlaps({field}, {value})"#);
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
                        _ => "=",
                    };
                    if operator == "IN" || operator == "NOT IN" {
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
                                    format!(r#"json_overlaps({field}, {value})"#)
                                })
                                .collect::<Vec<_>>()
                                .join(" OR ")
                        } else {
                            value
                                .split(';')
                                .map(|v| {
                                    let value = Self::format_value(column, v);
                                    format!(r#"json_overlaps({field}, {value})"#)
                                })
                                .collect::<Vec<_>>()
                                .join(" AND ")
                        }
                    } else {
                        let value = Self::format_value(column, value);
                        format!(r#"json_overlaps({field}, {value})"#)
                    }
                } else {
                    let value = Self::encode_value(column, Some(value));
                    format!(r#"json_overlaps({field}, {value})"#)
                }
            }
            "Map" => {
                let field = Self::format_field(field);
                let value = Self::encode_value(column, Some(value));
                format!(r#"json_overlaps({field}, {value})"#)
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
                .map(|s| format!("`{s}`"))
                .collect::<Vec<_>>()
                .join(".")
        } else {
            format!("`{field}`")
        }
    }

    #[inline]
    fn format_string(value: &str) -> String {
        format!("'{}'", value.replace('\'', "''"))
    }
}

impl DecodeRow<DatabaseRow> for Map {
    type Error = Error;

    fn decode_row(row: &DatabaseRow) -> Result<Self, Self::Error> {
        let columns = row.columns();
        let mut map = Map::with_capacity(columns.len());
        for col in columns {
            let field = col.name();
            let index = col.ordinal();
            let value = match col.type_info().name() {
                "BOOLEAN" => row.try_get_unchecked::<bool, _>(index)?.into(),
                "TINYINT" => row.try_get_unchecked::<i8, _>(index)?.into(),
                "TINYINT UNSIGNED" => row.try_get_unchecked::<u8, _>(index)?.into(),
                "SMALLINT" => row.try_get_unchecked::<i16, _>(index)?.into(),
                "SMALLINT UNSIGNED" => row.try_get_unchecked::<u16, _>(index)?.into(),
                "INT" => row.try_get_unchecked::<i32, _>(index)?.into(),
                "INT UNSIGNED" => row.try_get_unchecked::<u32, _>(index)?.into(),
                "BIGINT" => row.try_get_unchecked::<i64, _>(index)?.into(),
                "BIGINT UNSIGNED" => row.try_get_unchecked::<u64, _>(index)?.into(),
                "FLOAT" => row.try_get_unchecked::<f32, _>(index)?.into(),
                "DOUBLE" => row.try_get_unchecked::<f64, _>(index)?.into(),
                "TEXT" | "VARCHAR" | "CHAR" => row.try_get_unchecked::<String, _>(index)?.into(),
                "TIMESTAMP" => row.try_get_unchecked::<DateTime, _>(index)?.into(),
                "BLOB" | "VARBINARY" | "BINARY" => {
                    row.try_get_unchecked::<Vec<u8>, _>(index)?.into()
                }
                "JSON" => row.try_get_unchecked::<JsonValue, _>(index)?,
                _ => JsonValue::Null,
            };
            map.insert(field.to_owned(), value);
        }
        Ok(map)
    }
}

impl DecodeRow<DatabaseRow> for Record {
    type Error = Error;

    fn decode_row(row: &DatabaseRow) -> Result<Self, Self::Error> {
        let columns = row.columns();
        let mut record = Record::with_capacity(columns.len());
        for col in columns {
            let field = col.name();
            let index = col.ordinal();
            let value = match col.type_info().name() {
                "BOOLEAN" => row.try_get_unchecked::<bool, _>(index)?.into(),
                "INT" | "INT UNSIGNED" => row.try_get_unchecked::<i32, _>(index)?.into(),
                "BIGINT" | "BIGINT UNSIGNED" => row.try_get_unchecked::<i64, _>(index)?.into(),
                "FLOAT" => row.try_get_unchecked::<f32, _>(index)?.into(),
                "DOUBLE" => row.try_get_unchecked::<f64, _>(index)?.into(),
                "TEXT" | "VARCHAR" | "CHAR" => row.try_get_unchecked::<String, _>(index)?.into(),
                "TIMESTAMP" => row.try_get_unchecked::<DateTime, _>(index)?.into(),
                "BLOB" | "VARBINARY" | "BINARY" => {
                    row.try_get_unchecked::<Vec<u8>, _>(index)?.into()
                }
                "JSON" => row.try_get_unchecked::<JsonValue, _>(index)?.into(),
                _ => AvroValue::Null,
            };
            record.push((field.to_owned(), value));
        }
        Ok(record)
    }
}

impl QueryExt<DatabaseDriver> for Query {
    #[inline]
    fn query_fields(&self) -> &[String] {
        self.fields()
    }

    #[inline]
    fn query_filters(&self) -> &Map {
        self.filters()
    }

    #[inline]
    fn query_order(&self) -> (&str, bool) {
        self.sort_order()
    }

    fn format_pagination(&self) -> String {
        let (sort_by, _) = self.sort_order();
        if self.filters().contains_key(sort_by) {
            format!("LIMIT {}", self.limit())
        } else {
            format!("LIMIT {}, {}", self.offset(), self.limit())
        }
    }

    fn parse_text_search(filter: &Map) -> Option<String> {
        let fields = Validation::parse_str_array(filter.get("$fields"))?;
        Validation::parse_string(filter.get("$search")).map(|search| {
            let fields = fields.join(",");
            let search = DatabaseDriver::format_string(search.as_ref());
            format!("match({fields}) against({search})")
        })
    }
}
