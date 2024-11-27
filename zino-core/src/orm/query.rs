use super::{Entity, Schema};
use crate::{
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
    /// The projection fields.
    fields: Vec<String>,
    /// The logical `AND` conditions.
    logical_and: Vec<Map>,
    /// The logical `OR` conditions.
    logical_or: Vec<Map>,
    /// Sort order.
    sort_order: Vec<(String, bool)>,
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
            fields: Vec::new(),
            logical_and: Vec::new(),
            logical_or: Vec::new(),
            sort_order: Vec::new(),
            offset: 0,
            limit: 0,
        }
    }

    /// Adds a field corresponding to the column.
    #[inline]
    pub fn field(mut self, col: E::Column) -> Self {
        self.fields.push(E::format_column(&col));
        self.columns.push(col);
        self
    }

    /// Adds the fields corresponding to the columns.
    #[inline]
    pub fn fields(mut self, cols: Vec<E::Column>) -> Self {
        self.fields = cols.iter().map(E::format_column).collect();
        self.columns = cols;
        self
    }

    /// Adds a field with an alias for the column.
    #[inline]
    pub fn alias(mut self, col: E::Column, alias: &str) -> Self {
        let field = format!("{alias}:{}", E::format_column(&col));
        self.columns.push(col);
        self.fields.push(field);
        self
    }

    /// Adds a field with an optional alias for the aggregate function `COUNT` of a column.
    pub fn count(mut self, col: E::Column, alias: Option<&str>) -> Self {
        let col_name = E::format_column(&col);
        let field = if let Some(alias) = alias {
            format!("{alias}:count({col_name})")
        } else {
            format!("{}_count:count({})", col.as_ref(), col_name)
        };
        self.columns.push(col);
        self.fields.push(field);
        self
    }

    /// Adds a field with an optional alias for the aggregate function `COUNT DISTINCT` of a column.
    pub fn distinct(mut self, col: E::Column, alias: Option<&str>) -> Self {
        let col_name = E::format_column(&col);
        let field = if let Some(alias) = alias {
            format!("{alias}:count(distinct {col_name})")
        } else {
            format!("{}_distinct:count(distinct {})", col.as_ref(), col_name)
        };
        self.columns.push(col);
        self.fields.push(field);
        self
    }

    /// Adds a field with an optional alias for the aggregate function `SUM` of a column.
    pub fn sum(mut self, col: E::Column, alias: Option<&str>) -> Self {
        let col_name = E::format_column(&col);
        let field = if let Some(alias) = alias {
            format!("{alias}:sum({col_name})")
        } else {
            format!("{}_sum:sum({})", col.as_ref(), col_name)
        };
        self.columns.push(col);
        self.fields.push(field);
        self
    }

    /// Adds a field with an optional alias for the aggregate function `AVG` of a column.
    pub fn avg(mut self, col: E::Column, alias: Option<&str>) -> Self {
        let col_name = E::format_column(&col);
        let field = if let Some(alias) = alias {
            format!("{alias}:avg({col_name})")
        } else {
            format!("{}_avg:avg({})", col.as_ref(), col_name)
        };
        self.columns.push(col);
        self.fields.push(field);
        self
    }

    /// Adds a field with an optional alias for the aggregate function `MIN` of a column.
    pub fn min(mut self, col: E::Column, alias: Option<&str>) -> Self {
        let col_name = E::format_column(&col);
        let field = if let Some(alias) = alias {
            format!("{alias}:min({col_name})")
        } else {
            format!("{}_min:min({})", col.as_ref(), col_name)
        };
        self.columns.push(col);
        self.fields.push(field);
        self
    }

    /// Adds a field with an optional alias for the aggregate function `MAX` of a column.
    #[inline]
    pub fn max(mut self, col: E::Column, alias: Option<&str>) -> Self {
        let col_name = E::format_column(&col);
        let field = if let Some(alias) = alias {
            format!("{alias}:max({col_name})")
        } else {
            format!("{}_max:max({})", col.as_ref(), col_name)
        };
        self.columns.push(col);
        self.fields.push(field);
        self
    }

    /// Adds a logical `AND` condition by merging the other query builder.
    pub fn and<M: Entity>(mut self, query: QueryBuilder<M>) -> Self {
        let mut logical_and = query.logical_and;
        let logical_or = query.logical_or;
        if !logical_or.is_empty() {
            logical_and.push(Map::from_entry("$or", logical_or));
        }
        self.logical_and.push(Map::from_entry("$and", logical_and));
        self
    }

    /// Adds a logical `AND NOT` condition by merging the other query builder.
    pub fn and_not<M: Entity>(mut self, query: QueryBuilder<M>) -> Self {
        let mut logical_and = query.logical_and;
        let logical_or = query.logical_or;
        if !logical_or.is_empty() {
            logical_and.push(Map::from_entry("$or", logical_or));
        }
        self.logical_and.push(Map::from_entry("$not", logical_and));
        self
    }

    /// Adds a logical `AND` condition for equal parts.
    #[inline]
    pub fn and_eq(mut self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        let condition = Map::from_entry(E::format_column(&col), value);
        self.logical_and.push(condition);
        self
    }

    /// Adds a logical `AND` condition for non-equal parts.
    #[inline]
    pub fn and_ne(self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        self.push_logical_and(col, "$ne", value.into())
    }

    /// Adds a logical `AND` condition for the field less than a value.
    #[inline]
    pub fn and_lt(self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        self.push_logical_and(col, "$lt", value.into())
    }

    /// Adds a logical `AND` condition for the field not greater than a value.
    #[inline]
    pub fn and_le(self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        self.push_logical_and(col, "$le", value.into())
    }

    /// Adds a logical `AND` condition for the field greater than a value.
    #[inline]
    pub fn and_gt(self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        self.push_logical_and(col, "$gt", value.into())
    }

    /// Adds a logical `AND` condition for the field not less than a value.
    #[inline]
    pub fn and_ge(self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        self.push_logical_and(col, "$ge", value.into())
    }

    /// Adds a logical `AND` condition for the field `IN` a list of values.
    #[inline]
    pub fn and_in<T, V>(self, col: E::Column, values: V) -> Self
    where
        T: Into<JsonValue>,
        V: Into<Vec<T>>,
    {
        self.push_logical_and(col, "$in", values.into().into())
    }

    /// Adds a logical `AND` condition for the field `NOT IN` a list of values.
    #[inline]
    pub fn and_not_in<T, V>(self, col: E::Column, values: V) -> Self
    where
        T: Into<JsonValue>,
        V: Into<Vec<T>>,
    {
        self.push_logical_and(col, "$nin", values.into().into())
    }

    /// Adds a logical `AND` condition for the field `BETWEEN` two values.
    #[inline]
    pub fn and_between<T: Into<JsonValue>>(self, col: E::Column, min: T, max: T) -> Self {
        self.push_logical_and(col, "$betw", vec![min, max].into())
    }

    /// Adds a logical `AND` condition for the field `LIKE` a string value.
    #[inline]
    pub fn and_like(self, col: E::Column, value: String) -> Self {
        self.push_logical_and(col, "$like", value.into())
    }

    /// Adds a logical `AND` condition for the field `ILIKE` a string value.
    #[inline]
    pub fn and_ilike(self, col: E::Column, value: String) -> Self {
        self.push_logical_and(col, "$ilike", value.into())
    }

    /// Adds a logical `AND` condition for the field `RLIKE` a string value.
    #[inline]
    pub fn and_rlike(self, col: E::Column, value: String) -> Self {
        self.push_logical_and(col, "$rlike", value.into())
    }

    /// Adds a logical `AND` condition for the field which contains a string value.
    #[inline]
    pub fn and_contains(self, col: E::Column, value: &str) -> Self {
        self.push_logical_and(col, "$like", format!("%{value}%").into())
    }

    /// Adds a logical `AND` condition for the field which starts with a string value.
    #[inline]
    pub fn and_starts_with(self, col: E::Column, value: &str) -> Self {
        self.push_logical_and(col, "$like", format!("{value}%").into())
    }

    /// Adds a logical `AND` condition for the field which ends with a string value.
    #[inline]
    pub fn and_ends_with(self, col: E::Column, value: &str) -> Self {
        self.push_logical_and(col, "$like", format!("%{value}").into())
    }

    /// Adds a logical `AND` condition for the field which is null.
    #[inline]
    pub fn and_null(self, col: E::Column) -> Self {
        self.push_logical_and(col, "$is", JsonValue::Null)
    }

    /// Adds a logical `AND` condition for the field which is not null.
    #[inline]
    pub fn and_not_null(self, col: E::Column) -> Self {
        self.push_logical_and(col, "$is", "not_null".into())
    }

    /// Adds a logical `AND` condition for the two ranges which overlaps with each other.
    #[inline]
    pub fn and_overlaps<T: Into<JsonValue>>(
        mut self,
        cols: (E::Column, E::Column),
        values: (T, T),
    ) -> Self {
        let mut condition = Map::new();
        condition.upsert(E::format_column(&cols.0), Map::from_entry("$le", values.1));
        condition.upsert(E::format_column(&cols.1), Map::from_entry("$ge", values.0));
        self.logical_and.push(condition);
        self
    }

    /// Adds a logical `OR` condition by merging the other query builder.
    pub fn or<M: Entity>(mut self, query: QueryBuilder<M>) -> Self {
        let mut logical_and = query.logical_and;
        let logical_or = query.logical_or;
        if !logical_or.is_empty() {
            logical_and.push(Map::from_entry("$or", logical_or));
        }
        self.logical_or.push(Map::from_entry("$and", logical_and));
        self
    }

    /// Adds a logical `OR NOT` condition by merging the other query builder.
    pub fn or_not<M: Entity>(mut self, query: QueryBuilder<M>) -> Self {
        let mut logical_and = query.logical_and;
        let logical_or = query.logical_or;
        if !logical_or.is_empty() {
            logical_and.push(Map::from_entry("$or", logical_or));
        }
        self.logical_or.push(Map::from_entry("$not", logical_and));
        self
    }

    /// Adds a logical `OR` condition for equal parts.
    #[inline]
    pub fn or_eq(mut self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        let condition = Map::from_entry(E::format_column(&col), value);
        self.logical_or.push(condition);
        self
    }

    /// Adds a logical `OR` condition for non-equal parts.
    #[inline]
    pub fn or_ne(self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        self.push_logical_or(col, "$ne", value.into())
    }

    /// Adds a logical `OR` condition for the field less than a value.
    #[inline]
    pub fn or_lt(self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        self.push_logical_or(col, "$lt", value.into())
    }

    /// Adds a logical `OR` condition for the field not greater than a value.
    #[inline]
    pub fn or_le(self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        self.push_logical_or(col, "$le", value.into())
    }

    /// Adds a logical `OR` condition for the field greater than a value.
    #[inline]
    pub fn or_gt(self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        self.push_logical_or(col, "$gt", value.into())
    }

    /// Adds a logical `OR` condition for the field not less than a value.
    #[inline]
    pub fn or_ge(self, col: E::Column, value: impl Into<JsonValue>) -> Self {
        self.push_logical_or(col, "$ge", value.into())
    }

    /// Adds a logical `OR` condition for the field `IN` a list of values.
    #[inline]
    pub fn or_in<T, V>(self, col: E::Column, values: V) -> Self
    where
        T: Into<JsonValue>,
        V: Into<Vec<T>>,
    {
        self.push_logical_or(col, "$in", values.into().into())
    }

    /// Adds a logical `OR` condition for the field `NOT IN` a list of values.
    #[inline]
    pub fn or_not_in<T, V>(self, col: E::Column, values: V) -> Self
    where
        T: Into<JsonValue>,
        V: Into<Vec<T>>,
    {
        self.push_logical_or(col, "$nin", values.into().into())
    }

    /// Adds a logical `OR` condition for the field `BETWEEN` two values.
    #[inline]
    pub fn or_between<T: Into<JsonValue>>(self, col: E::Column, min: T, max: T) -> Self {
        self.push_logical_or(col, "$betw", vec![min, max].into())
    }

    /// Adds a logical `OR` condition for the field `LIKE` a string value.
    #[inline]
    pub fn or_like(self, col: E::Column, value: String) -> Self {
        self.push_logical_or(col, "$like", value.into())
    }

    /// Adds a logical `OR` condition for the field `ILIKE` a string value.
    #[inline]
    pub fn or_ilike(self, col: E::Column, value: String) -> Self {
        self.push_logical_or(col, "$ilike", value.into())
    }

    /// Adds a logical `OR` condition for the field `RLIKE` a string value.
    #[inline]
    pub fn or_rlike(self, col: E::Column, value: String) -> Self {
        self.push_logical_or(col, "$rlike", value.into())
    }

    /// Adds a logical `OR` condition for the field which contains a string value.
    #[inline]
    pub fn or_contains(self, col: E::Column, value: &str) -> Self {
        self.push_logical_or(col, "$like", format!("%{value}%").into())
    }

    /// Adds a logical `OR` condition for the field which starts with a string value.
    #[inline]
    pub fn or_starts_with(self, col: E::Column, value: &str) -> Self {
        self.push_logical_or(col, "$like", format!("{value}%").into())
    }

    /// Adds a logical `OR` condition for the field which ends with a string value.
    #[inline]
    pub fn or_ends_with(self, col: E::Column, value: &str) -> Self {
        self.push_logical_or(col, "$like", format!("%{value}").into())
    }

    /// Adds a logical `OR` condition for the field which is null.
    #[inline]
    pub fn or_null(self, col: E::Column) -> Self {
        self.push_logical_or(col, "$is", JsonValue::Null)
    }

    /// Adds a logical `OR` condition for the field which is not null.
    #[inline]
    pub fn or_not_null(self, col: E::Column) -> Self {
        self.push_logical_or(col, "$is", "not_null".into())
    }

    /// Adds a logical `OR` condition for the two ranges which overlaps with each other.
    #[inline]
    pub fn or_overlaps<T: Into<JsonValue>>(
        mut self,
        cols: (E::Column, E::Column),
        values: (T, T),
    ) -> Self {
        let mut condition = Map::new();
        condition.upsert(E::format_column(&cols.0), Map::from_entry("$le", values.1));
        condition.upsert(E::format_column(&cols.1), Map::from_entry("$ge", values.0));
        self.logical_or.push(condition);
        self
    }

    /// Sets the sort order.
    #[inline]
    pub fn order_by(mut self, col: E::Column, descending: bool) -> Self {
        let field = col.as_ref().into();
        self.sort_order.push((field, descending));
        self
    }

    /// Sets the sort with an ascending order.
    #[inline]
    pub fn order_asc(mut self, col: E::Column) -> Self {
        let field = col.as_ref().into();
        self.sort_order.push((field, false));
        self
    }

    /// Sets the sort with an descending order.
    #[inline]
    pub fn order_desc(mut self, col: E::Column) -> Self {
        let field = col.as_ref().into();
        self.sort_order.push((field, true));
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
    pub fn build(self) -> Query {
        let mut filters = Map::new();
        let logical_and = self.logical_and;
        let logical_or = self.logical_or;
        if !logical_and.is_empty() {
            filters.upsert("$and", logical_and);
        }
        if !logical_or.is_empty() {
            filters.upsert("$or", logical_or);
        }

        let mut query = Query::new(filters);
        query.set_fields(self.fields);
        query.set_offset(self.offset);
        query.set_limit(self.limit);
        for (field, descending) in self.sort_order.into_iter() {
            query.order_by(field, descending);
        }
        query
    }

    /// Pushes a logical `AND` condition for the column and expressions.
    #[inline]
    fn push_logical_and(mut self, col: E::Column, operator: &str, value: JsonValue) -> Self {
        let condition = Map::from_entry(operator, value);
        self.logical_and
            .push(Map::from_entry(E::format_column(&col), condition));
        self
    }

    /// Pushes a logical `OR` condition for the column and expressions.
    #[inline]
    fn push_logical_or(mut self, col: E::Column, operator: &str, value: JsonValue) -> Self {
        let condition = Map::from_entry(operator, value);
        self.logical_or
            .push(Map::from_entry(E::format_column(&col), condition));
        self
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
