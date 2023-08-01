use super::{query::QueryExt, DatabaseDriver, DatabaseRow};
use crate::{
    datetime::DateTime,
    error::Error,
    extension::{AvroRecordExt, JsonObjectExt, JsonValueExt},
    model::{Column, DecodeRow, EncodeColumn, Query},
    AvroValue, JsonValue, Map, Record, SharedString, Uuid,
};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use sqlx::{types::Decimal, Column as _, Row, TypeInfo, ValueRef};
use std::borrow::Cow;

impl<'c> EncodeColumn<DatabaseDriver> for Column<'c> {
    fn column_type(&self) -> &str {
        let type_name = self.type_name();
        match type_name {
            "bool" => "BOOLEAN",
            "u64" | "usize" | "Option<u64>" => "BIGINT UNSIGNED",
            "i64" | "isize" | "Option<i64>" => "BIGINT",
            "u32" | "Option<u32>" => "INT UNSIGNED",
            "i32" | "Option<i32>" => "INT",
            "u16" => "SMALLINT UNSIGNED",
            "i16" => "SMALLINT",
            "u8" => "TINYINT UNSIGNED",
            "i8" => "TINYINT",
            "f64" => "DOUBLE",
            "f32" => "FLOAT",
            "Decimal" => "NUMERIC",
            "String" | "Option<String>" => {
                if self.default_value().or(self.index_type()).is_some() {
                    "VARCHAR(255)"
                } else {
                    "TEXT"
                }
            }
            "DateTime" => "TIMESTAMP(6)",
            "NaiveDateTime" => "DATETIME(6)",
            "NaiveDate" | "Date" => "DATE",
            "NaiveTime" | "Time" => "TIME",
            "Uuid" | "Option<Uuid>" => "VARCHAR(36)",
            "Vec<u8>" => "BLOB",
            "Vec<String>" => "JSON",
            "Vec<Uuid>" => "JSON",
            "Map" => "JSON",
            _ => type_name,
        }
    }

    fn encode_value<'a>(&self, value: Option<&'a JsonValue>) -> Cow<'a, str> {
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
                        if let Some(value) = self.default_value() {
                            self.format_value(value).into_owned().into()
                        } else {
                            "''".into()
                        }
                    } else if value == "null" {
                        "NULL".into()
                    } else {
                        self.format_value(value)
                    }
                }
                JsonValue::Array(value) => {
                    let values = value
                        .iter()
                        .map(|v| match v {
                            JsonValue::String(v) => Query::escape_string(v),
                            _ => self.encode_value(Some(v)).into_owned(),
                        })
                        .collect::<Vec<_>>();
                    format!(r#"json_array({})"#, values.join(",")).into()
                }
                JsonValue::Object(_) => Query::escape_string(value).into(),
            }
        } else if self.default_value().is_some() {
            "DEFAULT".into()
        } else {
            "NULL".into()
        }
    }

    fn format_value<'a>(&self, value: &'a str) -> Cow<'a, str> {
        match self.type_name() {
            "bool" => {
                let value = if value == "true" { "TRUE" } else { "FALSE" };
                value.into()
            }
            "u64" | "u32" | "u16" | "u8" | "usize" | "Option<u64>" | "Option<u32>" => {
                if value.parse::<u64>().is_ok() {
                    value.into()
                } else {
                    "NULL".into()
                }
            }
            "i64" | "i32" | "i16" | "i8" | "isize" | "Option<i64>" | "Option<i32>" => {
                if value.parse::<i64>().is_ok() {
                    value.into()
                } else {
                    "NULL".into()
                }
            }
            "f64" | "f32" | "Decimal" => {
                if value.parse::<f64>().is_ok() {
                    value.into()
                } else {
                    "NULL".into()
                }
            }
            "String" | "Option<String>" | "Uuid" | "Option<Uuid>" => {
                Query::escape_string(value).into()
            }
            "DateTime" | "NaiveDateTime" => match value {
                "epoch" => "from_unixtime(0)".into(),
                "now" => "current_timestamp(6)".into(),
                "today" => "curdate()".into(),
                "tomorrow" => "curdate() + INTERVAL 1 DAY".into(),
                "yesterday" => "curdate() - INTERVAL 1 DAY".into(),
                _ => Query::escape_string(value).into(),
            },
            "Date" | "NaiveDate" => match value {
                "epoch" => "'1970-01-01'".into(),
                "today" => "curdate()".into(),
                "tomorrow" => "curdate() + INTERVAL 1 DAY".into(),
                "yesterday" => "curdate() - INTERVAL 1 DAY".into(),
                _ => Query::escape_string(value).into(),
            },
            "Time" | "NaiveTime" => match value {
                "now" => "curtime()".into(),
                "midnight" => "'00:00:00'".into(),
                _ => Query::escape_string(value).into(),
            },
            "Vec<u8>" => format!("'{value}'").into(),
            "Vec<String>" | "Vec<Uuid>" => {
                if value.contains(',') {
                    let values = value
                        .split(',')
                        .map(Query::escape_string)
                        .collect::<Vec<_>>();
                    format!(r#"json_array({})"#, values.join(",")).into()
                } else {
                    let value = Query::escape_string(value);
                    format!(r#"json_array({value})"#).into()
                }
            }
            "Map" => Query::escape_string(value).into(),
            _ => "NULL".into(),
        }
    }

    fn format_filter(&self, field: &str, value: &JsonValue) -> String {
        let type_name = self.type_name();
        if let Some(filter) = value.as_object() {
            if type_name == "Map" {
                let field = Query::format_field(field);
                let value = self.encode_value(Some(value));
                // `json_overlaps()` was added in MySQL 8.0.17.
                return format!(r#"json_overlaps({field}, {value})"#);
            } else {
                let mut conditions = Vec::with_capacity(filter.len());
                for (name, value) in filter {
                    let operator = match name.as_str() {
                        "$eq" => "=",
                        "$ne" => "<>",
                        "$lt" => "<",
                        "$le" => "<=",
                        "$gt" => ">",
                        "$ge" => ">=",
                        "$in" => "IN",
                        "$nin" => "NOT IN",
                        "$like" => "LIKE",
                        "$ilike" => "ILIKE",
                        "$rlike" => "RLIKE",
                        "$is" => "IS",
                        _ => "=",
                    };
                    if operator == "IN" || operator == "NOT IN" {
                        if let Some(values) = value.as_array() && !values.is_empty() {
                            let field = Query::format_field(field);
                            let value = values
                                .iter()
                                .map(|v| self.encode_value(Some(v)))
                                .collect::<Vec<_>>()
                                .join(",");
                            let condition = format!(r#"{field} {operator} ({value})"#);
                            conditions.push(condition);
                        }
                    } else {
                        let field = Query::format_field(field);
                        let value = self.encode_value(Some(value));
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

        let field = Query::format_field(field);
        match type_name {
            "bool" => {
                let value = self.encode_value(Some(value));
                if value == "TRUE" {
                    format!(r#"{field} IS TRUE"#)
                } else {
                    format!(r#"{field} IS NOT TRUE"#)
                }
            }
            "DateTime" | "Date" | "Time" | "NaiveDateTime" | "NaiveDate" | "NaiveTime" => {
                if let Some(value) = value.as_str() {
                    if let Some((min_value, max_value)) = value.split_once(',') {
                        let min_value = self.format_value(min_value);
                        let max_value = self.format_value(max_value);
                        format!(r#"{field} >= {min_value} AND {field} < {max_value}"#)
                    } else {
                        let index = value.find(|ch| !"<>=".contains(ch)).unwrap_or(0);
                        if index > 0 {
                            let (operator, value) = value.split_at(index);
                            let value = self.format_value(value);
                            format!(r#"{field} {operator} {value}"#)
                        } else {
                            let value = self.format_value(value);
                            format!(r#"{field} = {value}"#)
                        }
                    }
                } else {
                    let value = self.encode_value(Some(value));
                    format!(r#"{field} = {value}"#)
                }
            }
            "u64" | "i64" | "u32" | "i32" | "Option<u64>" | "Option<i64>" | "Option<u32>"
            | "Option<i32>" => {
                if let Some(value) = value.as_str() {
                    if value == "null" {
                        format!(r#"{field} IS NULL"#)
                    } else if value == "notnull" {
                        format!(r#"{field} IS NOT NULL"#)
                    } else if value.contains(',') {
                        let value = value.split(',').collect::<Vec<_>>().join(",");
                        format!(r#"{field} IN ({value})"#)
                    } else {
                        let value = self.format_value(value);
                        format!(r#"{field} = {value}"#)
                    }
                } else {
                    let value = self.encode_value(Some(value));
                    format!(r#"{field} = {value}"#)
                }
            }
            "String" | "Option<String>" => {
                if let Some(value) = value.as_str() {
                    if value == "null" {
                        // either NULL or empty
                        format!(r#"({field} = '') IS NOT FALSE"#)
                    } else if value == "notnull" {
                        format!(r#"({field} = '') IS FALSE"#)
                    } else if value.contains(',') {
                        let value = value
                            .split(',')
                            .map(Query::escape_string)
                            .collect::<Vec<_>>()
                            .join(",");
                        format!(r#"{field} IN ({value})"#)
                    } else {
                        let index = value.find(|ch| !"!~*".contains(ch)).unwrap_or(0);
                        if index > 0 {
                            let (operator, value) = value.split_at(index);
                            let value = Query::escape_string(value);
                            format!(r#"{field} {operator} {value}"#)
                        } else {
                            let operator = self.default_value().map(|_| "=").unwrap_or("RLIKE");
                            let value = Query::escape_string(value);
                            format!(r#"{field} {operator} {value}"#)
                        }
                    }
                } else {
                    let value = self.encode_value(Some(value));
                    format!(r#"{field} = {value}"#)
                }
            }
            "Uuid" | "Option<Uuid>" => {
                if let Some(value) = value.as_str() {
                    if value == "null" {
                        format!(r#"{field} IS NULL"#)
                    } else if value == "notnull" {
                        format!(r#"{field} IS NOT NULL"#)
                    } else if value.contains(',') {
                        let value = value
                            .split(',')
                            .map(Query::escape_string)
                            .collect::<Vec<_>>()
                            .join(",");
                        format!(r#"{field} IN ({value})"#)
                    } else {
                        let value = Query::escape_string(value);
                        format!(r#"{field} = {value}"#)
                    }
                } else {
                    let value = self.encode_value(Some(value));
                    format!(r#"{field} = {value}"#)
                }
            }
            "Vec<String>" | "Vec<Uuid>" | "Vec<u64>" | "Vec<i64>" | "Vec<u32>" | "Vec<i32>" => {
                if let Some(value) = value.as_str() {
                    if value.contains(';') {
                        if value.contains(',') {
                            value
                                .split(',')
                                .map(|v| {
                                    let s = v.replace(';', ",");
                                    let value = self.format_value(&s);
                                    format!(r#"json_overlaps({field}, {value})"#)
                                })
                                .collect::<Vec<_>>()
                                .join(" OR ")
                        } else {
                            value
                                .split(';')
                                .map(|v| {
                                    let value = self.format_value(v);
                                    format!(r#"json_overlaps({field}, {value})"#)
                                })
                                .collect::<Vec<_>>()
                                .join(" AND ")
                        }
                    } else {
                        let value = self.format_value(value);
                        format!(r#"json_overlaps({field}, {value})"#)
                    }
                } else {
                    let value = self.encode_value(Some(value));
                    format!(r#"json_overlaps({field}, {value})"#)
                }
            }
            "Map" => {
                let value = self.encode_value(Some(value));
                format!(r#"json_overlaps({field}, {value})"#)
            }
            _ => {
                let value = self.encode_value(Some(value));
                format!(r#"{field} = {value}"#)
            }
        }
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
            let raw_value = row.try_get_raw(index)?;
            let value = if raw_value.is_null() {
                JsonValue::Null
            } else {
                use super::decode::decode_column;
                match col.type_info().name() {
                    "BOOLEAN" => decode_column::<bool>(field, raw_value)?.into(),
                    "TINYINT" => decode_column::<i8>(field, raw_value)?.into(),
                    "TINYINT UNSIGNED" => decode_column::<u8>(field, raw_value)?.into(),
                    "SMALLINT" => decode_column::<i16>(field, raw_value)?.into(),
                    "SMALLINT UNSIGNED" => decode_column::<u16>(field, raw_value)?.into(),
                    "INT" => decode_column::<i32>(field, raw_value)?.into(),
                    "INT UNSIGNED" => decode_column::<u32>(field, raw_value)?.into(),
                    "BIGINT" => decode_column::<i64>(field, raw_value)?.into(),
                    "BIGINT UNSIGNED" => decode_column::<u64>(field, raw_value)?.into(),
                    "FLOAT" => decode_column::<f32>(field, raw_value)?.into(),
                    "DOUBLE" => decode_column::<f64>(field, raw_value)?.into(),
                    "NUMERIC" => {
                        let value = decode_column::<Decimal>(field, raw_value)?;
                        serde_json::to_value(value)?
                    }
                    "TEXT" | "VARCHAR" | "CHAR" => {
                        decode_column::<String>(field, raw_value)?.into()
                    }
                    "TIMESTAMP" => decode_column::<DateTime>(field, raw_value)?.into(),
                    "DATETIME" => decode_column::<NaiveDateTime>(field, raw_value)?
                        .to_string()
                        .into(),
                    "DATE" => decode_column::<NaiveDate>(field, raw_value)?
                        .to_string()
                        .into(),
                    "TIME" => decode_column::<NaiveTime>(field, raw_value)?
                        .to_string()
                        .into(),
                    "BYTE" | "BINARY" | "VARBINARY" | "BLOB" => {
                        let bytes = decode_column::<Vec<u8>>(field, raw_value)?;
                        if bytes.len() == 16 {
                            if let Ok(value) = Uuid::from_slice(&bytes) {
                                value.to_string().into()
                            } else {
                                bytes.into()
                            }
                        } else {
                            bytes.into()
                        }
                    }
                    "JSON" => decode_column::<JsonValue>(field, raw_value)?,
                    _ => JsonValue::Null,
                }
            };
            if !value.is_ignorable() {
                map.insert(field.to_owned(), value);
            }
        }
        Ok(map)
    }

    #[inline]
    fn update(&mut self, field: &str, value: JsonValue) {
        self.upsert(field, value);
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
            let raw_value = row.try_get_raw(index)?;
            let value = if raw_value.is_null() {
                AvroValue::Null
            } else {
                use super::decode::decode_column;
                match col.type_info().name() {
                    "BOOLEAN" => decode_column::<bool>(field, raw_value)?.into(),
                    "TINYINT" => i32::from(decode_column::<i8>(field, raw_value)?).into(),
                    "TINYINT UNSIGNED" => i32::from(decode_column::<u8>(field, raw_value)?).into(),
                    "SMALLINT" => i32::from(decode_column::<i16>(field, raw_value)?).into(),
                    "SMALLINT UNSIGNED" => {
                        i32::from(decode_column::<u16>(field, raw_value)?).into()
                    }
                    "INT" => decode_column::<i32>(field, raw_value)?.into(),
                    "INT UNSIGNED" => {
                        i32::try_from(decode_column::<u32>(field, raw_value)?)?.into()
                    }
                    "BIGINT" => decode_column::<i64>(field, raw_value)?.into(),
                    "BIGINT UNSIGNED" => {
                        i64::try_from(decode_column::<u64>(field, raw_value)?)?.into()
                    }
                    "FLOAT" => decode_column::<f32>(field, raw_value)?.into(),
                    "DOUBLE" => decode_column::<f64>(field, raw_value)?.into(),
                    "NUMERIC" => decode_column::<Decimal>(field, raw_value)?
                        .to_string()
                        .into(),
                    "TEXT" | "VARCHAR" | "CHAR" => {
                        decode_column::<String>(field, raw_value)?.into()
                    }
                    "TIMESTAMP" => decode_column::<DateTime>(field, raw_value)?.into(),
                    "DATETIME" => decode_column::<NaiveDateTime>(field, raw_value)?
                        .to_string()
                        .into(),
                    "DATE" => decode_column::<NaiveDate>(field, raw_value)?
                        .to_string()
                        .into(),
                    "TIME" => decode_column::<NaiveTime>(field, raw_value)?
                        .to_string()
                        .into(),
                    "BYTE" | "BINARY" | "VARBINARY" | "BLOB" => {
                        let bytes = decode_column::<Vec<u8>>(field, raw_value)?;
                        if bytes.len() == 16 {
                            if let Ok(value) = Uuid::from_slice(&bytes) {
                                value.into()
                            } else {
                                bytes.into()
                            }
                        } else {
                            bytes.into()
                        }
                    }
                    "JSON" => decode_column::<JsonValue>(field, raw_value)?.into(),
                    _ => AvroValue::Null,
                }
            };
            record.push((field.to_owned(), value));
        }
        Ok(record)
    }

    #[inline]
    fn update(&mut self, field: &str, value: JsonValue) {
        self.upsert(field, value);
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
    fn query_order(&self) -> &[(SharedString, bool)] {
        self.sort_order()
    }

    #[inline]
    fn query_offset(&self) -> usize {
        self.offset()
    }

    #[inline]
    fn query_limit(&self) -> usize {
        self.limit()
    }

    #[inline]
    fn placeholder(_n: usize) -> SharedString {
        "?".into()
    }

    #[inline]
    fn prepare_query<'a>(
        query: &'a str,
        params: Option<&'a Map>,
    ) -> (Cow<'a, str>, Vec<&'a JsonValue>) {
        crate::helper::prepare_sql_query(query, params, '?')
    }

    fn format_field(field: &str) -> Cow<'_, str> {
        if field.contains('.') {
            field
                .split('.')
                .map(|s| format!("`{s}`"))
                .collect::<Vec<_>>()
                .join(".")
                .into()
        } else {
            format!("`{field}`").into()
        }
    }

    fn parse_text_search(filter: &Map) -> Option<String> {
        let fields = filter.parse_str_array("$fields")?;
        filter.parse_string("$search").map(|search| {
            let fields = fields.join(",");
            let search = Query::escape_string(search.as_ref());
            format!("match({fields}) against({search})")
        })
    }
}
