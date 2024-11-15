use super::{Entity, Schema};
use crate::{
    error::Error,
    extension::{JsonObjectExt, JsonValueExt},
    model::{EncodeColumn, Query},
    JsonValue, Map, SharedString,
};
use std::{borrow::Cow, fmt::Display};

/// A query builder for the model entity.
#[derive(Debug, Clone)]
pub struct QueryBuilder<E: Entity> {
    /// The selection columns.
    columns: Vec<E::Column>,
    /// The logical `AND` conditions.
    logical_and: Map,
    /// The logical `OR` conditions.
    logical_or: Vec<Map>,
    // Offset.
    offset: usize,
    // Limit.
    limit: usize,
}

impl<E: Entity> QueryBuilder<E> {
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            logical_and: Map::new(),
            logical_or: Vec::new(),
            offset: 0,
            limit: 0,
        }
    }

    /// Adds a field corresponding to the column.
    #[inline]
    pub fn field(mut self, col: E::Column) -> Self {
        self.columns.push(col);
        self
    }

    /// Adds the fields corresponding to the columns.
    #[inline]
    pub fn fields(mut self, cols: Vec<E::Column>) -> Self {
        self.columns = cols;
        self
    }

    /// Adds a logical `AND` condition by merging the other query builder.
    pub fn and<M: Entity>(mut self, query: QueryBuilder<M>) -> Self {
        let mut logical_and = query.logical_and;
        let logical_or = query.logical_or;
        if !logical_or.is_empty() {
            logical_and.upsert("$or", logical_or);
        }
        if let Some(conditions) = self
            .logical_and
            .get_mut("$and")
            .and_then(|v| v.as_array_mut())
        {
            conditions.push(logical_and.into());
        } else {
            self.logical_and.upsert("$and", vec![logical_and]);
        }
        self
    }

    /// Adds a logical `AND NOT` condition by merging the other query builder.
    pub fn and_not<M: Entity>(mut self, query: QueryBuilder<M>) -> Self {
        let mut logical_and = query.logical_and;
        let logical_or = query.logical_or;
        if !logical_or.is_empty() {
            logical_and.upsert("$or", logical_or);
        }

        let condition = Map::from_entry("$not", logical_and);
        if let Some(conditions) = self
            .logical_and
            .get_mut("$and")
            .and_then(|v| v.as_array_mut())
        {
            conditions.push(condition.into());
        } else {
            self.logical_and.upsert("$and", vec![condition]);
        }
        self
    }

    /// Adds a logical `AND` condition for equal parts.
    #[inline]
    pub fn and_eq(mut self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        self.logical_and.upsert(col.as_ref(), value);
        self
    }

    /// Adds a logical `AND` condition for non-equal parts.
    #[inline]
    pub fn and_ne(mut self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        let condition = Map::from_entry("$ne", value);
        self.logical_and.upsert(col.as_ref(), condition);
        self
    }

    /// Adds a logical `AND` condition for the field less than a value.
    #[inline]
    pub fn and_lt(mut self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        let condition = Map::from_entry("$lt", value);
        self.logical_and.upsert(col.as_ref(), condition);
        self
    }

    /// Adds a logical `AND` condition for the field not greater than a value.
    #[inline]
    pub fn and_le(mut self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        let condition = Map::from_entry("$le", value);
        self.logical_and.upsert(col.as_ref(), condition);
        self
    }

    /// Adds a logical `AND` condition for the field greater than a value.
    #[inline]
    pub fn and_gt(mut self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        let condition = Map::from_entry("$gt", value);
        self.logical_and.upsert(col.as_ref(), condition);
        self
    }

    /// Adds a logical `AND` condition for the field not less than a value.
    #[inline]
    pub fn and_ge(mut self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        let condition = Map::from_entry("$ge", value);
        self.logical_and.upsert(col.as_ref(), condition);
        self
    }

    /// Adds a logical `AND` condition for the field `IN` a list of values.
    #[inline]
    pub fn and_in<T: Into<JsonValue>>(mut self, col: E::Column, values: Vec<T>) -> Self {
        let values = values.into_iter().map(|v| v.into()).collect::<Vec<_>>();
        let condition = Map::from_entry("$in", values);
        self.logical_and.upsert(col.as_ref(), condition);
        self
    }

    /// Adds a logical `AND` condition for the field `NOT IN` a list of values.
    #[inline]
    pub fn and_not_in<T: Into<JsonValue>>(mut self, col: E::Column, values: Vec<T>) -> Self {
        let values = values.into_iter().map(|v| v.into()).collect::<Vec<_>>();
        let condition = Map::from_entry("$nin", values);
        self.logical_and.upsert(col.as_ref(), condition);
        self
    }

    /// Adds a logical `AND` condition for the field `BETWEEN` two values.
    #[inline]
    pub fn and_between<T: Into<JsonValue>>(mut self, col: E::Column, min: T, max: T) -> Self {
        let condition = Map::from_entry("$betw", vec![min.into(), max.into()]);
        self.logical_and.upsert(col.as_ref(), condition);
        self
    }

    /// Adds a logical `AND` condition for the field `LIKE` a string value.
    #[inline]
    pub fn and_like<T: Into<JsonValue>>(mut self, col: E::Column, value: String) -> Self {
        let condition = Map::from_entry("$like", value);
        self.logical_and.upsert(col.as_ref(), condition);
        self
    }

    /// Adds a logical `AND` condition for the field `ILIKE` a string value.
    #[inline]
    pub fn and_ilike<T: Into<JsonValue>>(mut self, col: E::Column, value: String) -> Self {
        let condition = Map::from_entry("$ilike", value);
        self.logical_and.upsert(col.as_ref(), condition);
        self
    }

    /// Adds a logical `AND` condition for the field `RLIKE` a string value.
    #[inline]
    pub fn and_rlike<T: Into<JsonValue>>(mut self, col: E::Column, value: String) -> Self {
        let condition = Map::from_entry("$rlike", value);
        self.logical_and.upsert(col.as_ref(), condition);
        self
    }

    /// Adds a logical `OR` condition by merging the other query builder.
    pub fn or<M: Entity>(mut self, query: QueryBuilder<M>) -> Self {
        let mut logical_and = query.logical_and;
        let logical_or = query.logical_or;
        if !logical_or.is_empty() {
            logical_and.upsert("$or", logical_or);
        }
        self.logical_or.push(logical_and);
        self
    }

    /// Adds a logical `OR NOT` condition by merging the other query builder.
    pub fn or_not<M: Entity>(mut self, query: QueryBuilder<M>) -> Self {
        let mut logical_and = query.logical_and;
        let logical_or = query.logical_or;
        if !logical_or.is_empty() {
            logical_and.upsert("$or", logical_or);
        }

        let condition = Map::from_entry("$not", logical_and);
        self.logical_or.push(condition);
        self
    }

    /// Adds a logical `OR` condition for equal parts.
    #[inline]
    pub fn or_eq(mut self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        let condition = Map::from_entry(col.as_ref(), value);
        self.logical_or.push(condition);
        self
    }

    /// Adds a logical `OR` condition for non-equal parts.
    #[inline]
    pub fn or_ne(mut self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        let condition = Map::from_entry("$ne", value);
        self.logical_or
            .push(Map::from_entry(col.as_ref(), condition));
        self
    }

    /// Adds a logical `OR` condition for the field less than a value.
    #[inline]
    pub fn or_lt(mut self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        let condition = Map::from_entry("$lt", value);
        self.logical_or
            .push(Map::from_entry(col.as_ref(), condition));
        self
    }

    /// Adds a logical `OR` condition for the field not greater than a value.
    #[inline]
    pub fn or_le(mut self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        let condition = Map::from_entry("$le", value);
        self.logical_or
            .push(Map::from_entry(col.as_ref(), condition));
        self
    }

    /// Adds a logical `OR` condition for the field greater than a value.
    #[inline]
    pub fn or_gt(mut self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        let condition = Map::from_entry("$gt", value);
        self.logical_or
            .push(Map::from_entry(col.as_ref(), condition));
        self
    }

    /// Adds a logical `OR` condition for the field not less than a value.
    #[inline]
    pub fn or_ge(mut self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        let condition = Map::from_entry("$ge", value);
        self.logical_or
            .push(Map::from_entry(col.as_ref(), condition));
        self
    }

    /// Adds a logical `OR` condition for the field `IN` a list of values.
    #[inline]
    pub fn or_in<T: Into<JsonValue>>(mut self, col: E::Column, values: Vec<T>) -> Self {
        let values = values.into_iter().map(|v| v.into()).collect::<Vec<_>>();
        let condition = Map::from_entry("$in", values);
        self.logical_or
            .push(Map::from_entry(col.as_ref(), condition));
        self
    }

    /// Adds a logical `OR` condition for the field `NOT IN` a list of values.
    #[inline]
    pub fn or_not_in<T: Into<JsonValue>>(mut self, col: E::Column, values: Vec<T>) -> Self {
        let values = values.into_iter().map(|v| v.into()).collect::<Vec<_>>();
        let condition = Map::from_entry("$nin", values);
        self.logical_or
            .push(Map::from_entry(col.as_ref(), condition));
        self
    }

    /// Adds a logical `OR` condition for the field `BETWEEN` two values.
    #[inline]
    pub fn or_between<T: Into<JsonValue>>(mut self, col: E::Column, min: T, max: T) -> Self {
        let condition = Map::from_entry("$betw", vec![min.into(), max.into()]);
        self.logical_or
            .push(Map::from_entry(col.as_ref(), condition));
        self
    }

    /// Adds a logical `OR` condition for the field `LIKE` a string value.
    #[inline]
    pub fn or_like<T: Into<JsonValue>>(mut self, col: E::Column, value: String) -> Self {
        let condition = Map::from_entry("$like", value);
        self.logical_or
            .push(Map::from_entry(col.as_ref(), condition));
        self
    }

    /// Adds a logical `OR` condition for the field `ILIKE` a string value.
    #[inline]
    pub fn or_ilike<T: Into<JsonValue>>(mut self, col: E::Column, value: String) -> Self {
        let condition = Map::from_entry("$ilike", value);
        self.logical_or
            .push(Map::from_entry(col.as_ref(), condition));
        self
    }

    /// Adds a logical `OR` condition for the field `RLIKE` a string value.
    #[inline]
    pub fn or_rlike<T: Into<JsonValue>>(mut self, col: E::Column, value: String) -> Self {
        let condition = Map::from_entry("$rlike", value);
        self.logical_or
            .push(Map::from_entry(col.as_ref(), condition));
        self
    }

    /// Sets the offset.
    #[inline]
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// Sets the limit.
    #[inline]
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Builds the model query.
    pub fn build(self) -> Result<Query, Error> {
        let mut filters = self.logical_and;
        filters.upsert("$or", self.logical_or);

        let mut query = Query::new(filters);
        let fields = self
            .columns
            .iter()
            .map(|col| col.as_ref())
            .collect::<Vec<_>>();
        query.allow_fields(&fields);
        query.set_offset(self.offset);
        query.set_limit(self.limit);
        Ok(query)
    }
}

impl<E: Entity> Default for QueryBuilder<E> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

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

    /// Returns the escaped table name.
    fn table_name_escaped<M: Schema>() -> String;

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
        let mut logical_and_conditions = Vec::with_capacity(filters.len());
        for (key, value) in filters {
            match key.as_str() {
                "$and" => {
                    if let Some(filters) = value.as_array() {
                        let condition = Self::format_logical_filters::<M>(filters, " AND ");
                        logical_and_conditions.push(condition);
                    }
                }
                "$not" => {
                    if let Some(filters) = value.as_array() {
                        let condition = Self::format_logical_filters::<M>(filters, " AND ");
                        logical_and_conditions.push(format!("(NOT {condition})"));
                    }
                }
                "$or" => {
                    if let Some(filters) = value.as_array() {
                        let condition = Self::format_logical_filters::<M>(filters, " OR ");
                        logical_and_conditions.push(condition);
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
                        logical_and_conditions.push(condition);
                    }
                }
                "$text" => {
                    if let Some(condition) = value.as_object().and_then(Self::parse_text_search) {
                        logical_and_conditions.push(condition);
                    }
                }
                "$ovlp" => {
                    if let Some(values) = value.parse_str_array() {
                        if let [start_field, end_field, start_value, end_value] = values.as_slice()
                        {
                            let start_field = Self::format_field(start_field);
                            let end_field = Self::format_field(end_field);
                            let start_value = Self::escape_string(start_value);
                            let end_value = Self::escape_string(end_value);
                            let condition = if cfg!(any(
                                feature = "orm-mariadb",
                                feature = "orm-mysql",
                                feature = "orm-tidb"
                            )) {
                                format!(
                                    r#"LEAST({end_field}, {end_value}) > GREATEST({start_field}, {start_value})"#
                                )
                            } else if cfg!(feature = "orm-postgres") {
                                format!(
                                    r#"({start_field}, {end_field}) OVERLAPS ({start_value}, {end_value})"#
                                )
                            } else {
                                format!(
                                    r#"MIN({end_field}, {end_value}) > MAX({start_field}, {start_value})"#
                                )
                            };
                            logical_and_conditions.push(condition);
                        }
                    }
                }
                _ => {
                    if let Some(col) = M::get_column(key) {
                        let condition = col.format_filter(key, value);
                        if !condition.is_empty() {
                            logical_and_conditions.push(condition);
                        }
                    } else if key.contains('.') {
                        let condition = Self::format_filter(key, value);
                        if !condition.is_empty() {
                            logical_and_conditions.push(condition);
                        }
                    }
                }
            }
        }
        if !logical_and_conditions.is_empty() {
            expression += &format!("WHERE {}", logical_and_conditions.join(" AND "));
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
                let mut logical_and_conditions = Vec::with_capacity(filter.len());
                for (key, value) in filter {
                    match key.as_str() {
                        "$and" => {
                            if let Some(filters) = value.as_array() {
                                let condition = Self::format_logical_filters::<M>(filters, " AND ");
                                logical_and_conditions.push(condition);
                            }
                        }
                        "$not" => {
                            if let Some(filters) = value.as_array() {
                                let condition = Self::format_logical_filters::<M>(filters, " AND ");
                                logical_and_conditions.push(format!("(NOT {condition})"));
                            }
                        }
                        "$nor" => {
                            if let Some(filters) = value.as_array() {
                                let condition = Self::format_logical_filters::<M>(filters, " OR ");
                                logical_and_conditions.push(format!("(NOT {condition})"));
                            }
                        }
                        "$or" => {
                            if let Some(filters) = value.as_array() {
                                let condition = Self::format_logical_filters::<M>(filters, " OR ");
                                logical_and_conditions.push(condition);
                            }
                        }
                        "$ovlp" => {
                            if let Some(values) = value.parse_str_array() {
                                if let [start_field, end_field, start_value, end_value] =
                                    values.as_slice()
                                {
                                    let start_field = Self::format_field(start_field);
                                    let end_field = Self::format_field(end_field);
                                    let start_value = Self::escape_string(start_value);
                                    let end_value = Self::escape_string(end_value);
                                    let condition = if cfg!(any(
                                        feature = "orm-mariadb",
                                        feature = "orm-mysql",
                                        feature = "orm-tidb"
                                    )) {
                                        format!(
                                            r#"overlaps({start_field}, {end_field}, {start_value}, {end_value})"#
                                        )
                                    } else if cfg!(feature = "orm-postgres") {
                                        format!(
                                            r#"({start_field}, {end_field}) OVERLAPS ({start_value}, {end_value})"#
                                        )
                                    } else {
                                        format!(
                                            r#"({start_field} <= {end_value} AND {end_field} >= {start_value})"#
                                        )
                                    };
                                    logical_and_conditions.push(condition);
                                }
                            }
                        }
                        _ => {
                            if let Some(col) = M::get_column(key) {
                                let condition = col.format_filter(key, value);
                                if !condition.is_empty() {
                                    logical_and_conditions.push(condition);
                                }
                            } else if key.contains('.') {
                                let condition = Self::format_filter(key, value);
                                if !condition.is_empty() {
                                    logical_and_conditions.push(condition);
                                }
                            }
                        }
                    }
                }
                if !logical_and_conditions.is_empty() {
                    let condition = format!("({})", logical_and_conditions.join(" AND "));
                    conditions.push(condition);
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
