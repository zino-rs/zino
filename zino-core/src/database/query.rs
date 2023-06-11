use super::Schema;
use crate::{
    extension::{JsonObjectExt, JsonValueExt},
    model::EncodeColumn,
    JsonValue, Map, SharedString,
};
use std::{borrow::Cow, fmt::Display};

/// Extension trait for [`Query`](crate::model::Query).
pub(super) trait QueryExt<DB> {
    /// Returns a reference to the projection fields.
    fn query_fields(&self) -> &[String];

    /// Returns a reference to the filters.
    fn query_filters(&self) -> &Map;

    /// Returns the sort order.
    fn query_order(&self) -> (&str, bool);

    /// Returns a placeholder for the n-th parameter.
    fn placeholder(n: usize) -> SharedString;

    /// Prepares the SQL query for binding parameters.
    fn prepare_query<'a>(
        query: &'a str,
        params: Option<&'a Map>,
    ) -> (Cow<'a, str>, Vec<&'a JsonValue>);

    /// Formats the query pagination to generate SQL `LIMIT` expression.
    fn format_pagination(&self) -> String;

    /// Formats a field for the query.
    fn format_field(field: &str) -> Cow<'_, str>;

    /// Parses text search filter.
    fn parse_text_search(filter: &Map) -> Option<String>;

    /// Escapes a string.
    #[inline]
    fn escape_string(value: impl Display) -> String {
        format!("'{}'", value.to_string().replace('\'', "''"))
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
                    if let Some((alias, expr)) = field.rsplit_once(':') {
                        let alias = Self::format_field(alias);
                        format!(r#"{expr} AS {alias}"#).into()
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
        let mut expression = String::new();
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
                    if let Some(Ok(value)) = value.parse_f64() {
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
                            let value = col.encode_value(Some(value));
                            format!(r#"{key} {operator} {value}"#)
                        } else {
                            col.format_filter(key, value)
                        };
                        if !condition.is_empty() {
                            conditions.push(condition);
                        }
                    }
                }
            }
        }
        if !conditions.is_empty() {
            expression += &format!("WHERE {}", conditions.join(" AND "));
        };
        if let Some(groups) = filters.parse_str_array("$group") {
            let groups = groups.join(", ");
            expression += &format!(" GROUP BY {groups}");
            if let Some(JsonValue::Object(selection)) = filters.get("$match") {
                let condition = Self::format_selection::<M>(selection, " AND ");
                expression += &format!(" HAVING {condition}");
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
            format!("ORDER BY {sort_by} {sort_order}")
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
                        let condition = col.format_filter(key, value);
                        if !condition.is_empty() {
                            conditions.push(condition);
                        }
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
