use super::{query::QueryExt, DatabaseDriver, DatabaseRow, Schema};
use crate::{
    datetime::DateTime,
    error::Error,
    extension::{JsonObjectExt, JsonValueExt},
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
            "u64" | "i64" | "usize" | "isize" | "Option<u64>" | "Option<i64>" => {
                if self.auto_increment() {
                    "BIGSERIAL"
                } else {
                    "BIGINT"
                }
            }
            "u32" | "i32" | "Option<u32>" | "Option<i32>" => {
                if self.auto_increment() {
                    "SERIAL"
                } else {
                    "INT"
                }
            }
            "u16" | "i16" | "u8" | "i8" => {
                if self.auto_increment() {
                    "SMALLSERIAL"
                } else {
                    "SMALLINT"
                }
            }
            "f64" => "DOUBLE PRECISION",
            "f32" => "REAL",
            "Decimal" => "NUMERIC",
            "String" | "Option<String>" => "TEXT",
            "DateTime" => "TIMESTAMPTZ",
            "NaiveDateTime" => "TIMESTAMP",
            "NaiveDate" | "Date" => "DATE",
            "NaiveTime" | "Time" => "TIME",
            "Uuid" | "Option<Uuid>" => "UUID",
            "Vec<u8>" => "BYTEA",
            "Vec<String>" => "TEXT[]",
            "Vec<Uuid>" => "UUID[]",
            "Map" => "JSONB",
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
                    format!("ARRAY[{}]::{}", values.join(","), self.column_type()).into()
                }
                JsonValue::Object(_) => {
                    format!("{}::{}", Query::escape_string(value), self.column_type()).into()
                }
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
            "String" | "Option<String>" => Query::escape_string(value).into(),
            "DateTime" | "NaiveDateTime" => match value {
                "epoch" => "'epoch'".into(),
                "now" => "now()".into(),
                "today" => "date_trunc('day', now())".into(),
                "tomorrow" => "date_trunc('day', now()) + '1 day'::INTERVAL".into(),
                "yesterday" => "date_trunc('day', now()) - '1 day'::INTERVAL".into(),
                _ => Query::escape_string(value).into(),
            },
            "Date" | "NaiveDate" => match value {
                "epoch" => "'epoch'".into(),
                "today" => "curdate()".into(),
                "tomorrow" => "curdate() + INTERVAL 1 DAY".into(),
                "yesterday" => "curdate() - INTERVAL 1 DAY".into(),
                _ => Query::escape_string(value).into(),
            },
            "Time" | "NaiveTime" => match value {
                "now" => "curtime()".into(),
                "midnight" => "'allballs'".into(),
                _ => Query::escape_string(value).into(),
            },
            "Uuid" | "Option<Uuid>" => format!("'{value}'::uuid").into(),
            "Vec<u8>" => format!(r"'\x{value}'").into(),
            "Vec<Uuid>" | "Vec<String>" | "Vec<u64>" | "Vec<i64>" | "Vec<u32>" | "Vec<i32>" => {
                let column_type = self.column_type();
                if value.contains(',') {
                    let values = value
                        .split(',')
                        .map(Query::escape_string)
                        .collect::<Vec<_>>();
                    format!("ARRAY[{}]::{}", values.join(","), column_type).into()
                } else {
                    let value = Query::escape_string(value);
                    format!("ARRAY[{value}]::{column_type}").into()
                }
            }
            "Map" => {
                let value = Query::escape_string(value);
                format!("{value}::jsonb").into()
            }
            _ => "NULL".into(),
        }
    }

    fn format_filter(&self, field: &str, value: &JsonValue) -> String {
        let type_name = self.type_name();
        let field = Query::format_field(field);
        if let Some(filter) = value.as_object() {
            if type_name == "Map" {
                let value = self.encode_value(Some(value));
                return format!(r#"{field} @> {value}"#);
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
                        "$between" => "BETWEEN",
                        "$like" => "LIKE",
                        "$ilike" => "ILIKE",
                        "$rlike" => "~*",
                        "$is" => "IS",
                        "$all" => "@>",
                        "$size" => "array_length",
                        _ => name,
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
                                    .join(",");
                                let condition = format!(r#"{field} {operator} ({value})"#);
                                conditions.push(condition);
                            }
                        }
                    } else if operator == "BETWEEN" {
                        if let Some(values) = value.as_array() &&
                            let [min_value, max_value, ..] = values.as_slice()
                        {
                            let condition = format!(r#"{field} BETWEEN {min_value} AND {max_value}"#);
                            conditions.push(condition);
                        }
                    } else if operator == "array_length" {
                        let value = self.encode_value(Some(value));
                        let condition = format!(r#"array_length({field}, 1) = {value}"#);
                        conditions.push(condition);
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
        } else if let Some(range) = value.as_array() && range.len() == 2 {
            let min_value = self.encode_value(range.first());
            let max_value = self.encode_value(range.last());
            return format!(r#"{field} >= {min_value} AND {field} < {max_value}"#);
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
            "u64" | "i64" | "u32" | "i32" | "Option<u64>" | "Option<i64>" | "Option<u32>"
            | "Option<i32>" => {
                if let Some(value) = value.as_str() {
                    if value == "null" {
                        format!(r#"{field} IS NULL"#)
                    } else if value == "not_null" {
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
                    } else if value == "not_null" {
                        format!(r#"({field} = '') IS FALSE"#)
                    } else if self.index_type() == Some("text") {
                        if value.contains(',') {
                            value
                                .split(',')
                                .map(|s| {
                                    let value = Query::escape_string(s);
                                    format!(r#"{field} ~* {value}"#)
                                })
                                .collect::<Vec<_>>()
                                .join(" OR ")
                        } else {
                            let value = Query::escape_string(value);
                            format!(r#"{field} ~* {value}"#)
                        }
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
                            let value = Query::escape_string(value);
                            format!(r#"{field} = {value}"#)
                        }
                    }
                } else {
                    let value = self.encode_value(Some(value));
                    format!(r#"{field} = {value}"#)
                }
            }
            "DateTime" | "Date" | "Time" | "NaiveDateTime" | "NaiveDate" | "NaiveTime" => {
                if let Some(value) = value.as_str() {
                    if let Some((min_value, max_value)) = value.split_once(',') {
                        let min_value = self.format_value(min_value);
                        let max_value = self.format_value(max_value);
                        format!(r#"{field} >= {min_value} AND {field} < {max_value}"#)
                    } else {
                        let value = self.format_value(value);
                        format!(r#"{field} = {value}"#)
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
            "Vec<Uuid>" | "Vec<String>" | "Vec<u64>" | "Vec<i64>" | "Vec<u32>" | "Vec<i32>" => {
                if let Some(value) = value.as_str() {
                    if value.contains(';') {
                        value
                            .split(',')
                            .map(|v| {
                                let s = v.replace(';', ",");
                                let value = self.format_value(&s);
                                format!(r#"{field} @> {value}"#)
                            })
                            .collect::<Vec<_>>()
                            .join(" OR ")
                    } else {
                        let value = self.format_value(value);
                        format!(r#"{field} && {value}"#)
                    }
                } else {
                    let value = self.encode_value(Some(value));
                    format!(r#"{field} && {value}"#)
                }
            }
            "Map" => {
                if let Some(value) = value.as_str() {
                    // JSON path operator is supported in Postgres 12+
                    let value = Query::escape_string(value);
                    format!(r#"{field} @? {value}"#)
                } else {
                    let value = self.encode_value(Some(value));
                    format!(r#"{field} @> {value}"#)
                }
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
                    "BOOL" => decode_column::<bool>(field, raw_value)?.into(),
                    "INT2" => decode_column::<i16>(field, raw_value)?.into(),
                    "INT4" => decode_column::<i32>(field, raw_value)?.into(),
                    "INT8" => decode_column::<i64>(field, raw_value)?.into(),
                    "FLOAT4" => decode_column::<f32>(field, raw_value)?.into(),
                    "FLOAT8" => decode_column::<f64>(field, raw_value)?.into(),
                    "NUMERIC" => {
                        let value = decode_column::<Decimal>(field, raw_value)?;
                        serde_json::to_value(value)?
                    }
                    "TEXT" | "VARCHAR" | "CHAR" => {
                        decode_column::<String>(field, raw_value)?.into()
                    }
                    "TIMESTAMPTZ" => decode_column::<DateTime>(field, raw_value)?.into(),
                    "TIMESTAMP" => decode_column::<NaiveDateTime>(field, raw_value)?
                        .to_string()
                        .into(),
                    "DATE" => decode_column::<NaiveDate>(field, raw_value)?
                        .to_string()
                        .into(),
                    "TIME" => decode_column::<NaiveTime>(field, raw_value)?
                        .to_string()
                        .into(),
                    "UUID" => decode_column::<Uuid>(field, raw_value)?.to_string().into(),
                    "BYTEA" => decode_column::<Vec<u8>>(field, raw_value)?.into(),
                    "INT4[]" => decode_column::<Vec<i32>>(field, raw_value)?.into(),
                    "INT8[]" => decode_column::<Vec<i64>>(field, raw_value)?.into(),
                    "TEXT[]" => decode_column::<Vec<String>>(field, raw_value)?.into(),
                    "UUID[]" => {
                        let values = decode_column::<Vec<Uuid>>(field, raw_value)?;
                        values
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .into()
                    }
                    "JSONB" | "JSON" => decode_column::<JsonValue>(field, raw_value)?,
                    _ => JsonValue::Null,
                }
            };
            if !value.is_ignorable() {
                map.insert(field.to_owned(), value);
            }
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
            let raw_value = row.try_get_raw(index)?;
            let value = if raw_value.is_null() {
                AvroValue::Null
            } else {
                use super::decode::decode_column;
                match col.type_info().name() {
                    "BOOL" => decode_column::<bool>(field, raw_value)?.into(),
                    "INT4" => decode_column::<i32>(field, raw_value)?.into(),
                    "INT8" => decode_column::<i64>(field, raw_value)?.into(),
                    "FLOAT4" => decode_column::<f32>(field, raw_value)?.into(),
                    "FLOAT8" => decode_column::<f64>(field, raw_value)?.into(),
                    "NUMERIC" => decode_column::<Decimal>(field, raw_value)?
                        .to_string()
                        .into(),
                    "TEXT" | "VARCHAR" | "CHAR" => {
                        decode_column::<String>(field, raw_value)?.into()
                    }
                    "TIMESTAMPTZ" => decode_column::<DateTime>(field, raw_value)?.into(),
                    "TIMESTAMP" => decode_column::<NaiveDateTime>(field, raw_value)?
                        .to_string()
                        .into(),
                    "DATE" => decode_column::<NaiveDate>(field, raw_value)?
                        .to_string()
                        .into(),
                    "TIME" => decode_column::<NaiveTime>(field, raw_value)?
                        .to_string()
                        .into(),
                    "UUID" => decode_column::<Uuid>(field, raw_value)?.into(),
                    "BYTEA" => decode_column::<Vec<u8>>(field, raw_value)?.into(),
                    "INT4[]" => {
                        let values = decode_column::<Vec<i32>>(field, raw_value)?;
                        let vec = values.into_iter().map(AvroValue::Int).collect::<Vec<_>>();
                        AvroValue::Array(vec)
                    }
                    "INT8[]" => {
                        let values = decode_column::<Vec<i64>>(field, raw_value)?;
                        let vec = values.into_iter().map(AvroValue::Long).collect::<Vec<_>>();
                        AvroValue::Array(vec)
                    }
                    "TEXT[]" => {
                        let values = decode_column::<Vec<String>>(field, raw_value)?;
                        let vec = values
                            .into_iter()
                            .map(AvroValue::String)
                            .collect::<Vec<_>>();
                        AvroValue::Array(vec)
                    }
                    "UUID[]" => {
                        let values = decode_column::<Vec<Uuid>>(field, raw_value)?;
                        let vec = values.into_iter().map(AvroValue::Uuid).collect::<Vec<_>>();
                        AvroValue::Array(vec)
                    }
                    "JSONB" | "JSON" => decode_column::<JsonValue>(field, raw_value)?.into(),
                    _ => AvroValue::Null,
                }
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
    fn placeholder(n: usize) -> SharedString {
        if n == 1 {
            "$1".into()
        } else {
            format!("${n}").into()
        }
    }

    #[inline]
    fn prepare_query<'a>(
        query: &'a str,
        params: Option<&'a Map>,
    ) -> (Cow<'a, str>, Vec<&'a JsonValue>) {
        crate::helper::prepare_sql_query(query, params, '$')
    }

    fn format_field(field: &str) -> Cow<'_, str> {
        if field.contains('.') {
            field
                .split('.')
                .map(|s| format!(r#""{s}""#))
                .collect::<Vec<_>>()
                .join(".")
                .into()
        } else {
            format!(r#""{field}""#).into()
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
                            .map(|s| format!(r#""{s}""#))
                            .collect::<Vec<_>>()
                            .join(".")
                    } else {
                        format!(r#""{model_name}"."{field}""#)
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
        format!(r#""{table_name}" "{model_name}""#)
    }

    fn parse_text_search(filter: &Map) -> Option<String> {
        let fields = filter.parse_str_array("$fields")?;
        filter.parse_string("$search").map(|search| {
            let text = fields.join(" || ' ' || ");
            let lang = filter
                .parse_string("$language")
                .unwrap_or_else(|| "english".into());
            format!("to_tsvector('{lang}', {text}) @@ websearch_to_tsquery('{lang}', '{search}')")
        })
    }
}
