use super::{DatabaseDriver, Schema};
use crate::{model::EncodeColumn, request::Validation, Map, SharedString};
use serde_json::Value;
use std::borrow::Cow;

/// Extension trait for [`Query`](crate::model::Query).
pub(super) trait QueryExt<DB> {
    /// Returns a placeholder for the n-th parameter.
    fn placeholder(n: usize) -> SharedString;

    /// Returns a reference to the projection fields.
    fn query_fields(&self) -> &[String];

    /// Returns a reference to the filters.
    fn query_filters(&self) -> &Map;

    /// Returns the sort order.
    fn query_order(&self) -> (&str, bool);

    /// Formats the query pagination to generate SQL `LIMIT` expression.
    fn format_pagination(&self) -> String;

    /// Formats a field for the query.
    fn format_field(field: &str) -> Cow<'_, str>;

    /// Parses text search filter.
    fn parse_text_search(filter: &Map) -> Option<String>;

    /// Escapes a string.
    #[inline]
    fn escape_string(value: &str) -> String {
        format!("'{}'", value.replace('\'', "''"))
    }

    /// Formats projection fields.
    fn format_fields(&self) -> Cow<'_, str> {
        let fields = self.query_fields();
        if fields.is_empty() {
            "*".into()
        } else {
            fields
                .iter()
                .map(|field| {
                    if let Some((expr, alias)) = field.rsplit_once(":>") {
                        format!(r#"{expr} AS "{alias}""#).into()
                    } else {
                        Self::format_field(field)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
                .into()
        }
    }

    /// Formats the query filters to generate SQL `WHERE` expression.
    fn format_filters<M: Schema>(&self) -> String {
        let filters = self.query_filters();
        if filters.is_empty() {
            return String::new();
        }

        let (sort_by, ascending) = self.query_order();
        let mut expression = " ".to_owned();
        let mut conditions = Vec::with_capacity(filters.len());
        for (key, value) in filters {
            match key.as_str() {
                "$and" => {
                    if let Some(selection) = value.as_object() {
                        let condition = Self::format_selection::<M>(selection, " AND ");
                        conditions.push(condition);
                    }
                }
                "$not" => {
                    if let Some(selection) = value.as_object() {
                        let condition = Self::format_selection::<M>(selection, " AND ");
                        conditions.push(format!("NOT {condition}"));
                    }
                }
                "$nor" => {
                    if let Some(selection) = value.as_object() {
                        let condition = Self::format_selection::<M>(selection, " OR ");
                        conditions.push(format!("NOT {condition}"));
                    }
                }
                "$or" => {
                    if let Some(selection) = value.as_object() {
                        let condition = Self::format_selection::<M>(selection, " OR ");
                        conditions.push(condition);
                    }
                }
                "$rand" => {
                    if let Some(Ok(value)) = Validation::parse_f64(value) {
                        let condition = format!("random() < {value}");
                        conditions.push(condition);
                    }
                }
                "$text" => {
                    if let Some(value) = value.as_object() {
                        if let Some(condition) = Self::parse_text_search(value) {
                            conditions.push(condition);
                        }
                    }
                }
                _ => {
                    if let Some(col) = M::get_column(key) {
                        let condition = if key == sort_by {
                            // Use the filter condition to optimize pagination offset.
                            let key = Self::format_field(key);
                            let operator = if ascending { ">" } else { "<" };
                            let value = DatabaseDriver::encode_value(col, Some(value));
                            format!(r#"{key} {operator} {value}"#)
                        } else {
                            DatabaseDriver::format_filter(col, key, value)
                        };
                        conditions.push(condition);
                    }
                }
            }
        }
        if !conditions.is_empty() {
            expression += &format!("WHERE {}", conditions.join(" AND "));
        };
        if let Some(group) = filters.get("$group") {
            let groups = Validation::parse_str_array(group)
                .unwrap_or_default()
                .join(", ");
            expression += &format!("GROUP BY {groups}");
            if let Some(Value::Object(selection)) = filters.get("$match") {
                let condition = Self::format_selection::<M>(selection, " AND ");
                expression += &format!("HAVING {condition}");
            }
        }
        expression
    }

    /// Formats the query sort to generate SQL `ORDER BY` expression.
    fn format_sort(&self) -> String {
        let (sort_by, ascending) = self.query_order();
        if sort_by.is_empty() {
            String::new()
        } else {
            let sort_order = if ascending { "ASC" } else { "DESC" };
            format!("ORDER BY {sort_by} {sort_order} NULLS LAST")
        }
    }

    // Formats the selection with a logic operator.
    fn format_selection<M: Schema>(selection: &Map, operator: &str) -> String {
        let mut conditions = Vec::with_capacity(selection.len());
        for (key, value) in selection {
            match key.as_str() {
                "$and" => {
                    if let Some(selection) = value.as_object() {
                        let condition = Self::format_selection::<M>(selection, " AND ");
                        conditions.push(condition);
                    }
                }
                "$not" => {
                    if let Some(selection) = value.as_object() {
                        let condition = Self::format_selection::<M>(selection, " AND ");
                        conditions.push(format!("(NOT {condition})"));
                    }
                }
                "$nor" => {
                    if let Some(selection) = value.as_object() {
                        let condition = Self::format_selection::<M>(selection, " OR ");
                        conditions.push(format!("(NOT {condition})"));
                    }
                }
                "$or" => {
                    if let Some(selection) = value.as_object() {
                        let condition = Self::format_selection::<M>(selection, " OR ");
                        conditions.push(condition);
                    }
                }
                "$text" => {
                    if let Some(value) = value.as_object() {
                        if let Some(condition) = Self::parse_text_search(value) {
                            conditions.push(condition);
                        }
                    }
                }
                _ => {
                    if let Some(col) = M::get_column(key) {
                        let condition = DatabaseDriver::format_filter(col, key, value);
                        conditions.push(condition);
                    }
                }
            }
        }
        if conditions.is_empty() {
            String::new()
        } else {
            format!("({})", conditions.join(operator))
        }
    }
}
