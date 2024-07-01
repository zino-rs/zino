use super::{query::QueryExt, DatabaseDriver, DatabaseRow, Schema};
use crate::{
    datetime::{Date, DateTime, Time},
    error::Error,
    extension::{JsonObjectExt, JsonValueExt},
    model::{Column, DecodeRow, EncodeColumn, Query},
    AvroValue, JsonValue, Map, Record, SharedString, Uuid,
};
use std::borrow::Cow;

#[cfg(feature = "orm-sqlx")]
use sqlx::{Column as _, Row, TypeInfo, ValueRef};

impl<'c> EncodeColumn<DatabaseDriver> for Column<'c> {
    fn column_type(&self) -> &str {
        if let Some(column_type) = self.extra().get_str("column_type") {
            return column_type;
        }
        match self.type_name() {
            "bool" => "BOOLEAN",
            "u64" | "i64" | "usize" | "isize" | "Option<u64>" | "Option<i64>" | "u32" | "i32"
            | "u16" | "i16" | "u8" | "i8" | "Option<u32>" | "Option<i32>" => "INTEGER",
            "f64" | "f32" => "REAL",
            "Date" => "DATE",
            "Time" => "TIME",
            "DateTime" => "DATETIME",
            "Vec<u8>" => "BLOB",
            _ => "TEXT",
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
                    } else if value == "not_null" {
                        "NOT NULL".into()
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
            "u64" | "i64" | "u32" | "i32" | "u16" | "i16" | "u8" | "i8" | "usize" | "isize"
            | "Option<u64>" | "Option<i64>" | "Option<u32>" | "Option<i32>" => {
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
            "DateTime" | "NaiveDateTime" => match value {
                "epoch" => "datetime(0, 'unixepoch')".into(),
                "now" => "datetime('now', 'localtime')".into(),
                "today" => "datetime('now', 'start of day')".into(),
                "tomorrow" => "datetime('now', 'start of day', '+1 day')".into(),
                "yesterday" => "datetime('now', 'start of day', '-1 day')".into(),
                _ => Query::escape_string(value).into(),
            },
            "Date" | "NaiveDate" => match value {
                "epoch" => "'1970-01-01'".into(),
                "today" => "date('now', 'localtime')".into(),
                "tomorrow" => "date('now', '+1 day')".into(),
                "yesterday" => "date('now', '-1 day')".into(),
                _ => Query::escape_string(value).into(),
            },
            "Time" | "NaiveTime" => match value {
                "now" => "time('now', 'localtime')".into(),
                "midnight" => "'00:00:00'".into(),
                _ => Query::escape_string(value).into(),
            },
            "Vec<u8>" => format!("'{value}'").into(),
            "Vec<String>" | "Vec<Uuid>" | "Vec<u64>" | "Vec<i64>" | "Vec<u32>" | "Vec<i32>" => {
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
            _ => Query::escape_string(value).into(),
        }
    }

    fn format_filter(&self, field: &str, value: &JsonValue) -> String {
        let type_name = self.type_name();
        let field = Query::format_field(field);
        if let Some(filter) = value.as_object() {
            let mut conditions = Vec::with_capacity(filter.len());
            if type_name == "Map" {
                for (key, value) in filter {
                    let key = Query::escape_string(key);
                    let value = self.encode_value(Some(value));
                    let condition =
                        format!(r#"json_tree.key = {key} AND json_tree.value = {value}"#);
                    conditions.push(condition);
                }
                if conditions.is_empty() {
                    return String::new();
                } else {
                    return format!("({})", conditions.join(" OR "));
                }
            } else {
                for (name, value) in filter {
                    let name = name.as_str();
                    let operator = match name {
                        "$eq" => "=",
                        "$ne" => "<>",
                        "$lt" => "<",
                        "$le" => "<=",
                        "$gt" => ">",
                        "$ge" => ">=",
                        "$in" => "IN",
                        "$nin" => "NOT IN",
                        "$betw" => "BETWEEN",
                        "$like" => "LIKE",
                        "$rlike" => "REGEXP",
                        "$glob" => "GLOB",
                        "$is" => "IS",
                        "$size" => "json_array_length",
                        _ => {
                            if cfg!(debug_assertions) && name.starts_with('$') {
                                tracing::warn!("unsupported operator `{name}` for SQLite");
                            }
                            name
                        }
                    };
                    if operator == "IN" || operator == "NOT IN" {
                        if let Some(values) = value.as_array() {
                            if values.is_empty() {
                                let condition = if operator == "IN" { "FALSE" } else { "TRUE" };
                                conditions.push(condition.to_owned());
                            } else {
                                let value = values
                                    .iter()
                                    .map(|v| self.encode_value(Some(v)))
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                let condition = format!(r#"{field} {operator} ({value})"#);
                                conditions.push(condition);
                            }
                        }
                    } else if operator == "BETWEEN" {
                        if let Some(values) = value.parse_str_array() {
                            if let [min_value, max_value] = values.as_slice() {
                                let min_value = self.format_value(min_value);
                                let max_value = self.format_value(max_value);
                                let condition =
                                    format!(r#"({field} BETWEEN {min_value} AND {max_value})"#);
                                conditions.push(condition);
                            }
                        }
                    } else if operator == "json_array_length" {
                        if let Some(Ok(length)) = value.parse_usize() {
                            let condition = format!(r#"json_array_length({field}) = {length}"#);
                            conditions.push(condition);
                        }
                    } else {
                        let value = self.encode_value(Some(value));
                        let condition = format!(r#"{field} {operator} {value}"#);
                        conditions.push(condition);
                    }
                }
                if conditions.is_empty() {
                    return String::new();
                } else {
                    return conditions.join(" AND ");
                }
            }
        } else if let Some([min_value, max_value]) = value.as_array().map(|v| v.as_slice()) {
            let min_value = self.encode_value(Some(min_value));
            let max_value = self.encode_value(Some(max_value));
            return format!(r#"{field} >= {min_value} AND {field} < {max_value}"#);
        } else if let Some((min_value, max_value)) = value
            .as_str()
            .and_then(|value| value.split_once(','))
            .filter(|_| self.is_datetime_type())
        {
            let min_value = self.format_value(min_value);
            let max_value = self.format_value(max_value);
            return format!(r#"{field} >= {min_value} AND {field} < {max_value}"#);
        } else if value.is_null() {
            return format!(r#"{field} IS NULL"#);
        }

        match type_name {
            "bool" => {
                let value = self.encode_value(Some(value));
                if value == "TRUE" {
                    format!(r#"{field} IS TRUE"#)
                } else {
                    format!(r#"{field} IS NOT TRUE"#)
                }
            }
            "u64" | "i64" | "u32" | "i32" | "u16" | "i16" | "u8" | "i8" | "usize" | "isize"
            | "Option<u64>" | "Option<i64>" | "Option<u32>" | "Option<i32>" => {
                if let Some(value) = value.as_str() {
                    if value == "null" {
                        format!(r#"{field} IS NULL"#)
                    } else if value == "not_null" {
                        format!(r#"{field} IS NOT NULL"#)
                    } else if value == "nonzero" {
                        format!(r#"{field} <> 0"#)
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
                    } else if value == "not_null" {
                        format!(r#"({field} = '') IS FALSE"#)
                    } else if self.fuzzy_search() {
                        if value.contains(',') {
                            let exprs = value
                                .split(',')
                                .map(|s| {
                                    let value = Query::escape_string(format!("%{s}%"));
                                    format!(r#"{field} LIKE {value}"#)
                                })
                                .collect::<Vec<_>>();
                            format!("({})", exprs.join(" OR "))
                        } else {
                            let value = Query::escape_string(format!("%{value}%"));
                            format!(r#"{field} LIKE {value}"#)
                        }
                    } else if value.contains(',') {
                        let value = value
                            .split(',')
                            .map(Query::escape_string)
                            .collect::<Vec<_>>()
                            .join(", ");
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
            "DateTime" | "NaiveDateTime" => {
                if let Some(value) = value.as_str() {
                    let length = value.len();
                    let value = self.format_value(value);
                    match length {
                        4 => format!(r#"strftime('%Y', {field}) = {value}"#),
                        7 => format!(r#"strftime('%Y-%m', {field}) = {value}"#),
                        10 => format!(r#"strftime('%Y-%m-%d', {field}) = {value}"#),
                        _ => format!(r#"{field} = {value}"#),
                    }
                } else {
                    let value = self.encode_value(Some(value));
                    format!(r#"{field} = {value}"#)
                }
            }
            "Date" | "NaiveDate" => {
                if let Some(value) = value.as_str() {
                    let length = value.len();
                    let value = self.format_value(value);
                    match length {
                        4 => format!(r#"strftime('%Y', {field}) = {value}"#),
                        7 => format!(r#"strftime('%Y-%m', {field}) = {value}"#),
                        _ => format!(r#"{field} = {value}"#),
                    }
                } else {
                    let value = self.encode_value(Some(value));
                    format!(r#"{field} = {value}"#)
                }
            }
            "Time" | "NaiveTime" => {
                if let Some(value) = value.as_str() {
                    let length = value.len();
                    let value = self.format_value(value);
                    match length {
                        2 => format!(r#"strftime('%H', {field}) = {value}"#),
                        5 => format!(r#"strftime('%H:%M', {field}) = {value}"#),
                        8 => format!(r#"strftime('%H:%M:%S', {field}) = {value}"#),
                        _ => format!(r#"{field} = {value}"#),
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
                    } else if value == "not_null" {
                        format!(r#"{field} IS NOT NULL"#)
                    } else if value.contains(',') {
                        let value = value
                            .split(',')
                            .map(Query::escape_string)
                            .collect::<Vec<_>>()
                            .join(", ");
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
                    if value == "nonempty" {
                        format!(r#"json_array_length({field}) > 0"#)
                    } else {
                        let exprs = value
                            .split(',')
                            .map(|v| {
                                let value = Query::escape_string(v);
                                format!(r#"json_each.value = {value}"#)
                            })
                            .collect::<Vec<_>>();
                        format!("({})", exprs.join(" OR "))
                    }
                } else if let Some(values) = value.as_array() {
                    let exprs = values
                        .iter()
                        .map(|v| {
                            let value = self.encode_value(Some(v));
                            format!(r#"json_each.value = {value}"#)
                        })
                        .collect::<Vec<_>>();
                    format!("({})", exprs.join(" OR "))
                } else {
                    let value = self.encode_value(Some(value));
                    format!(r#"{field} = {value}"#)
                }
            }
            _ => {
                let value = self.encode_value(Some(value));
                format!(r#"{field} = {value}"#)
            }
        }
    }
}

#[cfg(feature = "orm-sqlx")]
impl DecodeRow<DatabaseRow> for Map {
    type Error = Error;

    fn decode_row(row: &DatabaseRow) -> Result<Self, Self::Error> {
        let mut map = Map::new();
        for col in row.columns() {
            let field = col.name();
            let index = col.ordinal();
            let raw_value = row.try_get_raw(index)?;
            let value = if raw_value.is_null() {
                JsonValue::Null
            } else {
                use super::decode::decode_raw;
                match col.type_info().name() {
                    "BOOLEAN" => decode_raw::<bool>(field, raw_value)?.into(),
                    "INTEGER" | "BIGINT" => decode_raw::<i64>(field, raw_value)?.into(),
                    "REAL" => decode_raw::<f64>(field, raw_value)?.into(),
                    "TEXT" => {
                        let value = decode_raw::<String>(field, raw_value)?;
                        if value.starts_with('[') && value.ends_with(']')
                            || value.starts_with('{') && value.ends_with('}')
                        {
                            serde_json::from_str(&value)?
                        } else {
                            value.into()
                        }
                    }
                    "DATETIME" => decode_raw::<DateTime>(field, raw_value)?.into(),
                    "DATE" => decode_raw::<Date>(field, raw_value)?.into(),
                    "TIME" => decode_raw::<Time>(field, raw_value)?.into(),
                    "BLOB" => {
                        let bytes = decode_raw::<Vec<u8>>(field, raw_value)?;
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
                    _ => decode_raw::<String>(field, raw_value)?.into(),
                }
            };
            if !value.is_ignorable() {
                map.insert(field.to_owned(), value);
            }
        }
        Ok(map)
    }
}

#[cfg(feature = "orm-sqlx")]
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
                use super::decode::decode_raw;
                match col.type_info().name() {
                    "BOOLEAN" => decode_raw::<bool>(field, raw_value)?.into(),
                    "INTEGER" | "BIGINT" => decode_raw::<i64>(field, raw_value)?.into(),
                    "REAL" => decode_raw::<f64>(field, raw_value)?.into(),
                    "TEXT" => {
                        let value = decode_raw::<String>(field, raw_value)?;
                        if value.starts_with('[') && value.ends_with(']')
                            || value.starts_with('{') && value.ends_with('}')
                        {
                            serde_json::from_str::<JsonValue>(&value)?.into()
                        } else {
                            value.into()
                        }
                    }
                    "DATETIME" => decode_raw::<DateTime>(field, raw_value)?.to_string().into(),
                    "DATE" => decode_raw::<Date>(field, raw_value)?.into(),
                    "TIME" => decode_raw::<Time>(field, raw_value)?.into(),
                    "BLOB" => {
                        let bytes = decode_raw::<Vec<u8>>(field, raw_value)?;
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
                    _ => decode_raw::<String>(field, raw_value)?.into(),
                }
            };
            record.push((field.to_owned(), value));
        }
        Ok(record)
    }
}

#[cfg(feature = "orm-sqlx")]
impl QueryExt<DatabaseDriver> for Query {
    type QueryResult = sqlx::sqlite::SqliteQueryResult;

    #[inline]
    fn parse_query_result(query_result: Self::QueryResult) -> (Option<i64>, u64) {
        let last_insert_id = query_result.last_insert_rowid();
        let rows_affected = query_result.rows_affected();
        (Some(last_insert_id), rows_affected)
    }

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

    fn format_table_fields<M: Schema>(&self) -> Cow<'_, str> {
        let model_name = M::model_name();
        let fields = self.query_fields();
        if fields.is_empty() {
            "*".into()
        } else {
            fields
                .iter()
                .map(|field| {
                    if let Some((alias, expr)) = field.split_once(':') {
                        let alias = Self::format_field(alias.trim());
                        format!(r#"{expr} AS {alias}"#)
                    } else if field.contains('.') {
                        field
                            .split('.')
                            .map(|s| format!("`{s}`"))
                            .collect::<Vec<_>>()
                            .join(".")
                    } else {
                        format!(r#"`{model_name}`.`{field}`"#)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
                .into()
        }
    }

    fn format_table_name<M: Schema>(&self) -> String {
        let table_name = M::table_name();
        let model_name = M::model_name();
        let filters = self.query_filters();
        let mut virtual_tables = Vec::new();
        for col in M::columns() {
            let col_name = col.name();
            if filters.contains_key(col_name) {
                match col.type_name() {
                    "Vec<String>" | "Vec<Uuid>" | "Vec<u64>" | "Vec<i64>" | "Vec<u32>"
                    | "Vec<i32>" => {
                        let virtual_table = format!("json_each(`{model_name}`.`{col_name}`)");
                        virtual_tables.push(virtual_table);
                    }
                    "Map" => {
                        let virtual_table = format!("json_tree(`{model_name}`.`{col_name}`)");
                        virtual_tables.push(virtual_table);
                    }
                    _ => (),
                }
            }
        }
        if virtual_tables.is_empty() {
            format!(r#"`{table_name}` AS `{model_name}`"#)
        } else {
            format!(
                r#"`{table_name}` AS `{model_name}`, {}"#,
                virtual_tables.join(", ")
            )
        }
    }

    #[inline]
    fn table_name_escaped<M: Schema>() -> String {
        let table_name = M::table_name();
        format!(r#"`{table_name}`"#)
    }

    fn parse_text_search(filter: &Map) -> Option<String> {
        let fields = filter.parse_str_array("$fields")?;
        filter.parse_string("$search").map(|search| {
            let fields = fields.join(",");
            let search = Query::escape_string(search.as_ref());
            format!("{fields} MATCH {search}")
        })
    }
}
