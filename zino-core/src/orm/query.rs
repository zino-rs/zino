use super::Schema;
use crate::{
    extension::{JsonObjectExt, JsonValueExt},
    model::EncodeColumn,
    JsonValue, Map, SharedString,
};
use std::{borrow::Cow, fmt::Display};

/// Extension trait for [`Query`](crate::model::Query).
pub(super) trait QueryExt<DB> {
    /// Query result type.
    type QueryResult;

    /// Parses the query result to get `last_insert_id` and `rows_affected`.
    fn parse_query_result(query_result: Self::QueryResult) -> (Option<i64>, u64);

    /// Returns a reference to the projection fields.
    fn query_fields(&self) -> &[String];

    /// Returns a reference to the filters.
    fn query_filters(&self) -> &Map;

    /// Returns the sort order.
    fn query_order(&self) -> &[(SharedString, bool)];

    /// Returns the query offset.
    fn query_offset(&self) -> usize;

    /// Returns the query limit.
    fn query_limit(&self) -> usize;

    /// Returns a placeholder for the n-th parameter.
    fn placeholder(n: usize) -> SharedString;

    /// Prepares the SQL query for binding parameters.
    fn prepare_query<'a>(
        query: &'a str,
        params: Option<&'a Map>,
    ) -> (Cow<'a, str>, Vec<&'a JsonValue>);

    /// Formats a field for the query.
    fn format_field(field: &str) -> Cow<'_, str>;

    /// Formats table fields.
    fn format_table_fields<M: Schema>(&self) -> Cow<'_, str>;

    /// Formats the table name.
    fn format_table_name<M: Schema>(&self) -> String;

    /// Parses text search filter.
    fn parse_text_search(filter: &Map) -> Option<String>;

    /// Escapes a string.
    #[inline]
    fn escape_string(value: impl Display) -> String {
        format!("'{}'", value.to_string().replace('\'', "''"))
    }

    /// Formats projection fields.
    fn format_projection(&self) -> Cow<'_, str> {
        let fields = self.query_fields();
        if fields.is_empty() {
            "*".into()
        } else {
            fields
                .iter()
                .map(|field| {
                    if let Some((alias, expr)) = field.split_once(':') {
                        let alias = Self::format_field(alias.trim());
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

        let mut expression = String::new();
        let mut conditions = Vec::with_capacity(filters.len());
        for (key, value) in filters {
            match key.as_str() {
                "$and" => {
                    if let Some(filters) = value.as_array() {
                        let condition = Self::format_logical_filters::<M>(filters, " AND ");
                        conditions.push(condition);
                    }
                }
                "$not" => {
                    if let Some(filters) = value.as_array() {
                        let condition = Self::format_logical_filters::<M>(filters, " AND ");
                        conditions.push(format!("(NOT {condition})"));
                    }
                }
                "$or" => {
                    if let Some(filters) = value.as_array() {
                        let condition = Self::format_logical_filters::<M>(filters, " OR ");
                        conditions.push(condition);
                    }
                }
                "$rand" => {
                    if let Some(Ok(value)) = value.parse_f64() {
                        let condition = if cfg!(any(
                            feature = "orm-mariadb",
                            feature = "orm-mysql",
                            feature = "orm-tidb"
                        )) {
                            format!("rand() < {value}")
                        } else if cfg!(feature = "orm-postgres") {
                            format!("random() < {value}")
                        } else {
                            let value = (value * i64::MAX as f64) as i64;
                            format!("abs(random()) < {value}")
                        };
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
                    } else if key.contains('.') {
                        let condition = Self::format_filter(key, value);
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
            let groups = groups
                .into_iter()
                .map(Self::format_field)
                .collect::<Vec<_>>()
                .join(", ");
            expression += &format!(" GROUP BY {groups}");
            if let Some(filters) = filters.get_array("$having") {
                let condition = Self::format_logical_filters::<M>(filters, " AND ");
                expression += &format!(" HAVING {condition}");
            }
        }
        expression
    }

    // Formats the filters with a logic operator.
    fn format_logical_filters<M: Schema>(filters: &[JsonValue], operator: &str) -> String {
        let mut conditions = Vec::with_capacity(filters.len());
        for filter in filters {
            if let JsonValue::Object(filter) = filter {
                for (key, value) in filter {
                    match key.as_str() {
                        "$and" => {
                            if let Some(filters) = value.as_array() {
                                let condition = Self::format_logical_filters::<M>(filters, " AND ");
                                conditions.push(condition);
                            }
                        }
                        "$not" => {
                            if let Some(filters) = value.as_array() {
                                let condition = Self::format_logical_filters::<M>(filters, " AND ");
                                conditions.push(format!("(NOT {condition})"));
                            }
                        }
                        "$nor" => {
                            if let Some(filters) = value.as_array() {
                                let condition = Self::format_logical_filters::<M>(filters, " OR ");
                                conditions.push(format!("(NOT {condition})"));
                            }
                        }
                        "$or" => {
                            if let Some(filters) = value.as_array() {
                                let condition = Self::format_logical_filters::<M>(filters, " OR ");
                                conditions.push(condition);
                            }
                        }
                        _ => {
                            if let Some(col) = M::get_column(key) {
                                let condition = col.format_filter(key, value);
                                if !condition.is_empty() {
                                    conditions.push(condition);
                                }
                            } else if key.contains('.') {
                                let condition = Self::format_filter(key, value);
                                if !condition.is_empty() {
                                    conditions.push(condition);
                                }
                            }
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

    /// Formats a query filter.
    fn format_filter(key: &str, value: &JsonValue) -> String {
        if let Some(filter) = value.as_object() {
            let mut conditions = Vec::with_capacity(filter.len());
            for (name, value) in filter {
                if let Some(value) = value.parse_string() {
                    let operator = match name.as_str() {
                        "$eq" => "=",
                        "$ne" => "<>",
                        "$lt" => "<",
                        "$le" => "<=",
                        "$gt" => ">",
                        "$ge" => ">=",
                        _ => "=",
                    };
                    let field = Self::format_field(key);
                    let value = Self::escape_string(value);
                    let condition = format!(r#"{field} {operator} {value}"#);
                    conditions.push(condition);
                }
            }
            if conditions.is_empty() {
                String::new()
            } else {
                format!("({})", conditions.join(" AND "))
            }
        } else if let Some(value) = value.parse_string() {
            let key = Self::format_field(key);
            let value = Self::escape_string(value);
            format!(r#"{key} = {value}"#)
        } else {
            String::new()
        }
    }

    /// Formats the query sort to generate SQL `ORDER BY` expression.
    fn format_sort(&self) -> String {
        let sort_order = self.query_order();
        if sort_order.is_empty() {
            String::new()
        } else {
            let sort_order = sort_order
                .iter()
                .map(|(sort, descending)| {
                    if *descending {
                        format!("{sort} DESC")
                    } else {
                        format!("{sort} ASC")
                    }
                })
                .collect::<Vec<_>>();
            format!("ORDER BY {}", sort_order.join(", "))
        }
    }

    /// Formats the query pagination to generate SQL `LIMIT` expression.
    fn format_pagination(&self) -> String {
        let limit = self.query_limit();
        if limit == 0 || limit == usize::MAX {
            return String::new();
        }

        let offset = self.query_offset();
        format!("LIMIT {limit} OFFSET {offset}")
    }
}
