use super::{query::QueryExt, DatabaseDriver, DatabaseRow, Schema};
use crate::{
    datetime::{Date, DateTime, Time},
    error::Error,
    extension::{JsonObjectExt, JsonValueExt},
    model::{Column, DecodeRow, EncodeColumn, Query},
    AvroValue, JsonValue, Map, Record, SharedString, Uuid,
};
use chrono::NaiveDateTime;
use std::borrow::Cow;

#[cfg(feature = "orm-sqlx")]
use sqlx::{types::Decimal, Column as _, Row, TypeInfo, ValueRef};

impl<'c> EncodeColumn<DatabaseDriver> for Column<'c> {
    fn column_type(&self) -> &str {
        if let Some(column_type) = self.extra().get_str("column_type") {
            return column_type;
        }
        match self.type_name() {
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
            "Date" | "NaiveDate" => "DATE",
            "Time" | "NaiveTime" => "TIME",
            "DateTime" => "TIMESTAMP(6)",
            "NaiveDateTime" => "DATETIME(6)",
            "Uuid" | "Option<Uuid>" => {
                if cfg!(feature = "orm-mariadb") {
                    "UUID"
                } else {
                    "CHAR(36)"
                }
            }
            "Vec<u8>" => "BLOB",
            "Vec<String>" | "Vec<Uuid>" | "Vec<u64>" | "Vec<i64>" | "Vec<u32>" | "Vec<i32>"
            | "Map" => "JSON",
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
            #[cfg(feature = "orm-mariadb")]
            "Uuid" | "Option<Uuid>" => format!("'{value}'").into(),
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
            if type_name == "Map" {
                let value = self.encode_value(Some(value));
                return format!(r#"json_contains({field}, {value})"#);
            } else {
                let mut conditions = Vec::with_capacity(filter.len());
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
                        "$ilike" => "ILIKE",
                        "$rlike" => "RLIKE",
                        "$is" => "IS",
                        "$size" => "json_length",
                        _ => {
                            if cfg!(debug_assertions) && name.starts_with('$') {
                                tracing::warn!("unsupported operator `{name}` for MySQL");
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
                    } else if operator == "json_length" {
                        if let Some(Ok(length)) = value.parse_usize() {
                            let condition = format!(r#"json_length({field}) = {length}"#);
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
                                    let value = Query::escape_string(s);
                                    format!(r#"{field} RLIKE {value}"#)
                                })
                                .collect::<Vec<_>>();
                            format!("({})", exprs.join(" OR "))
                        } else {
                            let value = Query::escape_string(value);
                            format!(r#"{field} RLIKE {value}"#)
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
                        4 => format!(r#"date_format({field}, '%Y') = {value}"#),
                        7 => format!(r#"date_format({field}, '%Y-%m') = {value}"#),
                        10 => format!(r#"date_format({field}, '%Y-%m-%d') = {value}"#),
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
                        4 => format!(r#"date_format({field}, '%Y') = {value}"#),
                        7 => format!(r#"date_format({field}, '%Y-%m') = {value}"#),
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
                        2 => format!(r#"date_format({field}, '%H') = {value}"#),
                        5 => format!(r#"date_format({field}, '%H:%i') = {value}"#),
                        8 => format!(r#"date_format({field}, '%H:%i:%s') = {value}"#),
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
                        format!(r#"json_length({field}) > 0"#)
                    } else if value.contains(';') {
                        let exprs = value
                            .split(',')
                            .map(|v| {
                                let s = v.replace(';', ",");
                                let value = self.format_value(&s);
                                format!(r#"json_contains({field}, {value})"#)
                            })
                            .collect::<Vec<_>>();
                        format!("({})", exprs.join(" OR "))
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
                format!(r#"json_contains({field}, {value})"#)
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
                    "TINYINT" => decode_raw::<i8>(field, raw_value)?.into(),
                    "TINYINT UNSIGNED" => decode_raw::<u8>(field, raw_value)?.into(),
                    "SMALLINT" => decode_raw::<i16>(field, raw_value)?.into(),
                    "SMALLINT UNSIGNED" => decode_raw::<u16>(field, raw_value)?.into(),
                    "INT" => decode_raw::<i32>(field, raw_value)?.into(),
                    "INT UNSIGNED" => decode_raw::<u32>(field, raw_value)?.into(),
                    "BIGINT" => decode_raw::<i64>(field, raw_value)?.into(),
                    "BIGINT UNSIGNED" => decode_raw::<u64>(field, raw_value)?.into(),
                    "FLOAT" => decode_raw::<f32>(field, raw_value)?.into(),
                    "DOUBLE" => decode_raw::<f64>(field, raw_value)?.into(),
                    "NUMERIC" => {
                        let value = decode_raw::<Decimal>(field, raw_value)?;
                        serde_json::to_value(value)?
                    }
                    "TIMESTAMP" => decode_raw::<DateTime>(field, raw_value)?.into(),
                    "DATETIME" => decode_raw::<NaiveDateTime>(field, raw_value)?
                        .to_string()
                        .into(),
                    "DATE" => decode_raw::<Date>(field, raw_value)?.into(),
                    "TIME" => decode_raw::<Time>(field, raw_value)?.into(),
                    "BYTE" | "BINARY" | "VARBINARY" | "BLOB" => {
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
                    "JSON" => decode_raw::<JsonValue>(field, raw_value)?,
                    #[cfg(feature = "orm-mariadb")]
                    "TEXT" | "LONGTEXT" => {
                        // In MariaDB, JSON is just an alias for LONGTEXT.
                        let value = decode_raw::<String>(field, raw_value)?;
                        if value.starts_with('[') && value.ends_with(']')
                            || value.starts_with('{') && value.ends_with('}')
                        {
                            serde_json::from_str(&value)?
                        } else {
                            value.into()
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
                    "TINYINT" => i32::from(decode_raw::<i8>(field, raw_value)?).into(),
                    "TINYINT UNSIGNED" => i32::from(decode_raw::<u8>(field, raw_value)?).into(),
                    "SMALLINT" => i32::from(decode_raw::<i16>(field, raw_value)?).into(),
                    "SMALLINT UNSIGNED" => i32::from(decode_raw::<u16>(field, raw_value)?).into(),
                    "INT" => decode_raw::<i32>(field, raw_value)?.into(),
                    "INT UNSIGNED" => i32::try_from(decode_raw::<u32>(field, raw_value)?)?.into(),
                    "BIGINT" => decode_raw::<i64>(field, raw_value)?.into(),
                    "BIGINT UNSIGNED" => {
                        i64::try_from(decode_raw::<u64>(field, raw_value)?)?.into()
                    }
                    "FLOAT" => decode_raw::<f32>(field, raw_value)?.into(),
                    "DOUBLE" => decode_raw::<f64>(field, raw_value)?.into(),
                    "NUMERIC" => decode_raw::<Decimal>(field, raw_value)?.to_string().into(),
                    "TIMESTAMP" => decode_raw::<DateTime>(field, raw_value)?.into(),
                    "DATETIME" => decode_raw::<NaiveDateTime>(field, raw_value)?
                        .to_string()
                        .into(),
                    "DATE" => decode_raw::<Date>(field, raw_value)?.into(),
                    "TIME" => decode_raw::<Time>(field, raw_value)?.into(),
                    "BYTE" | "BINARY" | "VARBINARY" | "BLOB" => {
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
                    "JSON" => decode_raw::<JsonValue>(field, raw_value)?.into(),
                    #[cfg(feature = "orm-mariadb")]
                    "TEXT" | "LONGTEXT" => {
                        let value = decode_raw::<String>(field, raw_value)?;
                        if value.starts_with('[') && value.ends_with(']')
                            || value.starts_with('{') && value.ends_with('}')
                        {
                            serde_json::from_str::<JsonValue>(&value)?.into()
                        } else {
                            value.into()
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
    type QueryResult = sqlx::mysql::MySqlQueryResult;

    #[inline]
    fn parse_query_result(query_result: Self::QueryResult) -> (Option<i64>, u64) {
        let last_insert_id = query_result.last_insert_id();
        let rows_affected = query_result.rows_affected();
        (last_insert_id.try_into().ok(), rows_affected)
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

    #[inline]
    fn format_table_name<M: Schema>(&self) -> String {
        let table_name = M::table_name();
        let model_name = M::model_name();
        format!(r#"`{table_name}` AS `{model_name}`"#)
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
            format!("match({fields}) against({search})")
        })
    }
}
