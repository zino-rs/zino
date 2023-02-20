use super::{
    column::{Column, ColumnExt},
    Schema,
};
use crate::{model::Query, request::Validation, Map};
use serde_json::Value;

/// Extension trait for [`Query`](crate::model::Query).
pub(super) trait QueryExt {
    /// Formats projection fields.
    fn format_fields(&self) -> String;

    /// Formats the query filter to generate SQL `WHERE` expression.
    fn format_filter<M: Schema>(&self) -> String;

    /// Formats the query sort to generate SQL `ORDER BY` expression.
    fn format_sort(&self) -> String;

    /// Formats the query pagination to generate SQL `LIMIT` expression.
    fn format_pagination(&self) -> String;

    // Formats the selection with a logic operator.
    fn format_selection<M: Schema>(selection: &Map, operator: &str) -> String;

    /// Parses text search filter.
    fn parse_text_search(filter: &Map) -> Option<String>;
}

impl QueryExt for Query {
    #[inline]
    fn format_fields(&self) -> String {
        let fields = self.fields();
        if fields.is_empty() {
            "*".to_owned()
        } else {
            fields.join(", ")
        }
    }

    fn format_filter<M: Schema>(&self) -> String {
        let filter = self.filter();
        if filter.is_empty() {
            return String::new();
        }

        let (sort_by, ascending) = self.sort_order();
        let mut expression = " ".to_owned();
        let mut conditions = Vec::new();
        for (key, value) in filter {
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
                            let value = col.encode_value(Some(value));
                            format!("{key} {operator} {value}")
                        } else {
                            col.format_filter(key, value)
                        };
                        conditions.push(condition);
                    }
                }
            }
        }
        if !conditions.is_empty() {
            expression += &format!("WHERE {}", conditions.join(" AND "));
        };
        if let Some(Value::String(group_by)) = filter.get("group_by") {
            expression += &format!("GROUP BY {group_by}");
            if let Some(Value::Object(selection)) = filter.get("having") {
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
        if self.filter().contains_key(sort_by) {
            format!("LIMIT {}", self.limit())
        } else {
            format!("LIMIT {} OFFSET {}", self.limit(), self.offset())
        }
    }

    fn format_selection<M: Schema>(selection: &Map, operator: &str) -> String {
        let mut conditions = Vec::new();
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
                        let condition = col.format_filter(key, value);
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
        let columns: Option<Vec<String>> = Validation::parse_array(filter.get("$columns"));
        if let Some(columns) = columns {
            if let Some(search) = Validation::parse_string(filter.get("$search")) {
                let column = columns.join(" || ' ' || ");
                let language = Validation::parse_string(filter.get("$language"))
                    .unwrap_or_else(|| "english".to_owned());
                let search = Column::format_string(&search);
                let condition = format!(
                    "to_tsvector('{language}', {column}) @@ websearch_to_tsquery('{language}', '{search}')",
                );
                return Some(condition);
            }
        }
        None
    }
}
