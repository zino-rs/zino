use super::Schema;
use crate::{
    model::{EncodeColumn, Query},
    request::Validation,
    Map,
};
use serde_json::Value;
use sqlx::Postgres;

/// Extension trait for [`Query`](crate::model::Query).
pub(super) trait QueryExt<DB> {
    /// Formats projection fields.
    fn format_fields(&self) -> String;

    /// Formats the query filters to generate SQL `WHERE` expression.
    fn format_filters<M: Schema>(&self) -> String;

    /// Formats the query sort to generate SQL `ORDER BY` expression.
    fn format_sort(&self) -> String;

    /// Formats the query pagination to generate SQL `LIMIT` expression.
    fn format_pagination(&self) -> String;

    // Formats the selection with a logic operator.
    fn format_selection<M: Schema>(selection: &Map, operator: &str) -> String;

    /// Parses text search filter.
    fn parse_text_search(filter: &Map) -> Option<String>;
}

impl QueryExt<Postgres> for Query {
    #[inline]
    fn format_fields(&self) -> String {
        let fields = self.fields();
        if fields.is_empty() {
            "*".to_owned()
        } else {
            fields.join(", ")
        }
    }

    fn format_filters<M: Schema>(&self) -> String {
        let filters = self.filters();
        if filters.is_empty() {
            return String::new();
        }

        let (sort_by, ascending) = self.sort_order();
        let mut expression = " ".to_owned();
        let mut conditions = Vec::with_capacity(filters.len());
        for (key, value) in filters {
            match key.as_str() {
                "sample" => {
                    if let Some(Ok(value)) = Validation::parse_f64(value) {
                        let condition = format!("random() < {value}");
                        conditions.push(condition);
                    }
                }
                "$and" => {
                    if let Some(selection) = value.as_object() {
                        let condition = Self::format_selection::<M>(selection, " AND ");
                        conditions.push(condition);
                    }
                }
                "$or" => {
                    if let Some(selection) = value.as_object() {
                        let condition = Self::format_selection::<M>(selection, " OR ");
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
                "$text" => {
                    if let Some(value) = value.as_object() {
                        if let Some(condition) = Self::parse_text_search(value) {
                            conditions.push(condition);
                        }
                    }
                }
                "$join" => {
                    if let Some(value) = value.as_str() {
                        expression += value;
                    }
                }
                _ => {
                    if let Some(col) = M::get_column(key) {
                        let condition = if key == sort_by {
                            // Use the filter condition to optimize pagination offset.
                            let operator = if ascending { ">" } else { "<" };
                            let value = Postgres::encode_value(col, Some(value));
                            format!("{key} {operator} {value}")
                        } else {
                            Postgres::format_filter(col, key, value)
                        };
                        conditions.push(condition);
                    }
                }
            }
        }
        if !conditions.is_empty() {
            expression += &format!("WHERE {}", conditions.join(" AND "));
        };
        if let Some(Value::String(group_by)) = filters.get("group_by") {
            expression += &format!("GROUP BY {group_by}");
            if let Some(Value::Object(selection)) = filters.get("having") {
                let condition = Self::format_selection::<M>(selection, " AND ");
                expression += &format!("HAVING {condition}");
            }
        }
        expression
    }

    fn format_sort(&self) -> String {
        let (sort_by, ascending) = self.sort_order();
        if sort_by.is_empty() {
            String::new()
        } else {
            let sort_order = if ascending { "ASC" } else { "DESC" };
            if sort_by.contains('.') {
                let sort_by = sort_by.replace('.', "->'") + "'";
                format!("ORDER BY {sort_by} {sort_order} NULLS LAST")
            } else {
                format!("ORDER BY {sort_by} {sort_order} NULLS LAST")
            }
        }
    }

    fn format_pagination(&self) -> String {
        let (sort_by, _) = self.sort_order();
        if self.filters().contains_key(sort_by) {
            format!("LIMIT {}", self.limit())
        } else {
            format!("LIMIT {} OFFSET {}", self.limit(), self.offset())
        }
    }

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
                "$or" => {
                    if let Some(selection) = value.as_object() {
                        let condition = Self::format_selection::<M>(selection, " OR ");
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
                "$text" => {
                    if let Some(value) = value.as_object() {
                        if let Some(condition) = Self::parse_text_search(value) {
                            conditions.push(condition);
                        }
                    }
                }
                _ => {
                    if let Some(col) = M::get_column(key) {
                        let condition = Postgres::format_filter(col, key, value);
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

    fn parse_text_search(filter: &Map) -> Option<String> {
        let columns: Vec<String> = Validation::parse_array(filter.get("$columns"))?;
        Validation::parse_string(filter.get("$search")).map(|search| {
            let col = columns.join(" || ' ' || ");
            let lang = Validation::parse_string(filter.get("$language"))
                .unwrap_or_else(|| "english".to_owned());
            format!("to_tsvector('{lang}', {col}) @@ websearch_to_tsquery('{lang}', '{search}')")
        })
    }
}
