//! Query builder for SQL.
//!
//! # Design references
//!
//! The design of our ORM is inspired by [`Mongoose`], [`Prisma`], [`TypeORM`] and [`PostgREST`].
//!
//! ```rust,ignore
//! use zino_core::{model::{Mutation, Query}, json, Map, Record};
//! use zino_orm::{JoinOn, Schema};
//!
//! // Constructs a model `Query` with JSON expressions.
//! let query = Query::new(json!({
//!     "$or": [
//!         {
//!             "roles": "worker",
//!             "visibility": "Public",
//!         },
//!         {
//!             "roles": { "$in": ["admin", "auditor"] },
//!             "visibility": { "$ne": "Public" },
//!         },
//!     ],
//!     "status": { "$nin": ["Deleted", "Locked"] },
//! }));
//!
//! // Constructs a model `Mutation` with JSON expressions.
//! let mut mutation = Mutation::new(json!({
//!     "status": "Active",
//!     "refreshed_at": DateTime::now(),
//!     "$inc": { "refresh_count": 1 },
//! }));
//!
//! // Updates the models using `update_many` provided by the `Schema` trait.
//! let ctx = User::update_many(&query, &mut mutation).await?;
//! ctx.emit_metrics("user_refresh");
//!
//! // Constructs a model `Query` with projection fields.
//! let mut query = Query::new(json!({
//!     "project.start_date": { "$le": "2023-10-07" },
//!     "project.end_date": { "$ge": "2023-10-01" },
//!     "task.status": "Completed",
//! }));
//! query.allow_fields(&[
//!     "task.id",
//!     "task.name",
//!     "task.status",
//!     "task.project_id",
//!     "project.start_date",
//!     "project.end_date",
//! ]);
//! query.order_desc("task.updated_at");
//!
//! // Performs a LEFT OUTER JOIN using `lookup` provided by the `Schema` trait.
//! let join_on = JoinOn::left_join().with("project_id", "id");
//! let entries = Task::lookup::<Project, Map>(&query, &join_on).await?;
//!
//! // Executes the raw SQL with interpolations `${param}` and argument bindings `#{param}`.
//! let sql =
//!     "SELECT u.id, u.name, u.tags, t.id, t.name \
//!         FROM ${user_table} u INNER JOIN ${tag_table} t \
//!             ON t.id = ANY(u.tags) AND t.category = #{category};";
//! let params = json!({
//!     "user_table": User::table_name(),
//!     "tag_table": Tag::table_name(),
//!     "category": "Rustacean",
//! });
//! let records = User::query::<Record>(sql, params.as_object()).await?;
//! ```
//!
//! # Query operators
//!
//! | Name       | MySQL               | PostgreSQL       | SQLite                |
//! |------------|---------------------|------------------|-----------------------|
//! | `$and`     | `AND`               | `AND`            | `AND`                 |
//! | `$or`      | `OR`                | `OR`             | `OR`                  |
//! | `$not`     | `NOT`               | `NOT`            | `NOT`                 |
//! | `$rand`    | `rand()`            | `random()`       | `abs(random())`       |
//! | `$text`    | `match() against()` | `to_tsvector()`  | `MATCH`               |
//! | `$eq`      | `=`                 | `=`              | `=`                   |
//! | `$ne`      | `<>`                | `<>`             | `<>`                  |
//! | `$lt`      | `<`                 | `<`              | `<`                   |
//! | `$le`      | `<=`                | `<=`             | `<=`                  |
//! | `$gt`      | `>`                 | `>`              | `>`                   |
//! | `$ge`      | `>=`                | `>=`             | `>=`                  |
//! | `$in`      | `IN`                | `IN`             | `IN`                  |
//! | `$nin`     | `NOT IN`            | `NOT IN`         | `NOT IN`              |
//! | `$betw`    | `BETWEEN AND`       | `BETWEEN AND`    | `BETWEEN AND`         |
//! | `$like`    | `LIKE`              | `LIKE`           | `LIKE`                |
//! | `$ilike`   | `ILIKE`             | `ILIKE`          | `LOWER() LIKE`        |
//! | `$rlike`   | `RLIKE`             | `~*`             | `REGEXP`              |
//! | `$is`      | `IS`                | `IS`             | `IS`                  |
//! | `$size`    | `json_length()`     | `array_length()` | `json_array_length()` |
//!
//! [`Mongoose`]: https://mongoosejs.com/
//! [`Prisma`]: https://www.prisma.io/
//! [`TypeORM`]: https://typeorm.io/
//! [`PostgREST`]: https://postgrest.org/

use super::{Aggregation, EncodeColumn, Entity, IntoSqlValue, ModelColumn, Schema, Window};
use regex::{Captures, Regex};
use std::{borrow::Cow, fmt::Display, marker::PhantomData};
use zino_core::{
    JsonValue, LazyLock, Map, SharedString,
    extension::{JsonObjectExt, JsonValueExt},
    model::{Query, QueryOrder},
};

/// A query builder for the model entity.
///
/// # Examples
/// ```rust,ignore
/// use crate::model::{User, UserColumn};
/// use zino_core::orm::{QueryBuilder, Schema};
///
/// let query = QueryBuilder::new()
///     .and_not_in(UserColumn::Status, ["Deleted", "Locked"])
///     .or(QueryBuilder::new()
///         .and_eq(UserColumn::Roles, "worker")
///         .and_eq(UserColumn::Visibility, "Public"))
///     .or(QueryBuilder::new()
///         .and_in(UserColumn::Roles, ["admin", "auditor"])
///         .and_ne(UserColumn::Visibility, "Public"))
///     .order_desc(UserColumn::UpdatedAt)
///     .limit(10)
///     .build();
/// let users: Vec<User> = User::find(&query).await?;
/// ```
#[derive(Debug, Clone)]
pub struct QueryBuilder<E: Entity> {
    /// The projection fields.
    fields: Vec<String>,
    /// The `GROUP BY` fields.
    group_by_fields: Vec<String>,
    /// The `HAVING` conditions.
    having_conditions: Vec<Map>,
    /// The filters.
    filters: Map,
    /// The logical `AND` conditions.
    logical_and: Vec<Map>,
    /// The logical `OR` conditions.
    logical_or: Vec<Map>,
    /// Sort order.
    sort_order: Vec<QueryOrder>,
    /// Offset.
    offset: usize,
    /// Limit.
    limit: usize,
    /// Extra flags.
    extra: Map,
    /// The phantom data.
    phantom: PhantomData<E>,
}

impl<E: Entity> QueryBuilder<E> {
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            group_by_fields: Vec::new(),
            having_conditions: Vec::new(),
            filters: Map::new(),
            logical_and: Vec::new(),
            logical_or: Vec::new(),
            sort_order: Vec::new(),
            offset: 0,
            limit: 0,
            extra: Map::new(),
            phantom: PhantomData,
        }
    }

    /// Sets a table for the query, which should only be used to specify a dynamic table.
    #[inline]
    pub fn table_name(mut self, table_name: String) -> Self {
        self.extra.upsert("table_name", table_name);
        self
    }

    /// Adds a field corresponding to the column.
    #[inline]
    pub fn field<C: ModelColumn<E>>(mut self, col: C) -> Self {
        self.fields.push(col.into_column_expr());
        self
    }

    /// Adds the fields corresponding to the columns.
    #[inline]
    pub fn fields<C, V>(mut self, cols: V) -> Self
    where
        C: ModelColumn<E>,
        V: Into<Vec<C>>,
    {
        let mut fields = cols
            .into()
            .into_iter()
            .map(|col| col.into_column_expr())
            .collect();
        self.fields.append(&mut fields);
        self
    }

    /// Adds a field with an alias for the column.
    pub fn alias<C: ModelColumn<E>>(mut self, col: C, alias: &str) -> Self {
        let col_name = col.into_column_expr();
        let field = Query::format_field(&col_name);
        let field_alias = [alias, ":", &field].concat();
        self.fields.push(field_alias);
        self
    }

    /// Adds a field with an optional alias for the aggregate function.
    pub fn aggregate(mut self, aggregation: Aggregation<E>, alias: Option<&str>) -> Self {
        let expr = aggregation.expr();
        let field_alias = if let Some(alias) = alias {
            [alias, ":", &expr].concat()
        } else {
            let mut field_alias = aggregation.default_alias();
            field_alias.push(':');
            field_alias.push_str(&expr);
            field_alias
        };
        self.fields.push(field_alias);
        self
    }

    /// Adds a field with an optional alias for the window function.
    pub fn window(mut self, window: Window<E>, alias: Option<&str>) -> Self {
        let expr = window.expr();
        let field_alias = if let Some(alias) = alias {
            [alias, ":", &expr].concat()
        } else {
            let mut field_alias = window.default_alias();
            field_alias.push(':');
            field_alias.push_str(&expr);
            field_alias
        };
        self.fields.push(field_alias);
        self
    }

    /// Adds a `GROUP BY` column.
    pub fn group_by<C: ModelColumn<E>>(mut self, col: C, alias: Option<&str>) -> Self {
        let expr = col.into_column_expr();
        let field = if let Some(alias) = alias {
            [alias, ":", &expr].concat()
        } else {
            expr.clone()
        };
        if !self.fields.contains(&field) {
            self.fields.push(field);
        }
        self.group_by_fields.push(expr);
        self
    }

    /// Adds a `HAVING` condition using the value as a filter for the column.
    #[inline]
    pub fn having_filter<V: IntoSqlValue>(mut self, aggregation: Aggregation<E>, value: V) -> Self {
        let condition = Map::from_entry(aggregation.expr(), value.into_sql_value());
        self.having_conditions.push(condition);
        self
    }

    /// Adds a `HAVING` condition for equal parts.
    #[inline]
    pub fn having_eq<V: IntoSqlValue>(self, aggregation: Aggregation<E>, value: V) -> Self {
        self.push_having_condition(aggregation, "$eq", value.into_sql_value())
    }

    /// Adds a `HAVING` condition for non-equal parts.
    #[inline]
    pub fn having_ne<V: IntoSqlValue>(self, aggregation: Aggregation<E>, value: V) -> Self {
        self.push_having_condition(aggregation, "$ne", value.into_sql_value())
    }

    /// Adds a `HAVING` condition for the column less than a value.
    #[inline]
    pub fn having_lt<V: IntoSqlValue>(self, aggregation: Aggregation<E>, value: V) -> Self {
        self.push_having_condition(aggregation, "$lt", value.into_sql_value())
    }

    /// Adds a `HAVING` condition for the column not greater than a value.
    #[inline]
    pub fn having_le<V: IntoSqlValue>(self, aggregation: Aggregation<E>, value: V) -> Self {
        self.push_having_condition(aggregation, "$le", value.into_sql_value())
    }

    /// Adds a `HAVING` condition for the column greater than a value.
    #[inline]
    pub fn having_gt<V: IntoSqlValue>(self, aggregation: Aggregation<E>, value: V) -> Self {
        self.push_having_condition(aggregation, "$gt", value.into_sql_value())
    }

    /// Adds a `HAVING` condition for the column not less than a value.
    #[inline]
    pub fn having_ge<V: IntoSqlValue>(self, aggregation: Aggregation<E>, value: V) -> Self {
        self.push_having_condition(aggregation, "$ge", value.into_sql_value())
    }

    /// Adds a logical `AND` condition for the primary key.
    #[inline]
    pub fn primary_key<V: IntoSqlValue>(mut self, value: V) -> Self {
        let field = E::format_column(&E::PRIMARY_KEY);
        self.filters.upsert(field, value.into_sql_value());
        self
    }

    /// Adds a logical `AND` condition which selects random items by `rand() < value`.
    #[inline]
    pub fn rand<V: IntoSqlValue>(mut self, value: V) -> Self {
        self.filters.upsert("$rand", value.into_sql_value());
        self
    }

    /// Adds a logical `AND` condition by merging the other query builder.
    pub fn and<M: Entity>(mut self, mut other: QueryBuilder<M>) -> Self {
        let mut logical_and = other.logical_and;
        let logical_or = other.logical_or;
        if !logical_or.is_empty() {
            logical_and.push(Map::from_entry("$or", logical_or));
        }
        if !logical_and.is_empty() {
            self.logical_and.push(Map::from_entry("$and", logical_and));
        }
        self.fields.append(&mut other.fields);
        self.group_by_fields.append(&mut other.group_by_fields);
        self
    }

    /// Adds a logical `AND NOT` condition by merging the other query builder.
    pub fn and_not<M: Entity>(mut self, mut other: QueryBuilder<M>) -> Self {
        let mut logical_and = other.logical_and;
        let logical_or = other.logical_or;
        if !logical_or.is_empty() {
            logical_and.push(Map::from_entry("$or", logical_or));
        }
        if !logical_and.is_empty() {
            self.logical_and.push(Map::from_entry("$not", logical_and));
        }
        self.fields.append(&mut other.fields);
        self.group_by_fields.append(&mut other.group_by_fields);
        self
    }

    /// Adds a logical `AND` condition using the value as a filter for the column.
    #[inline]
    pub fn and_filter<C, V>(mut self, col: C, value: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        let condition = Map::from_entry(col.into_column_expr(), value.into_sql_value());
        self.logical_and.push(condition);
        self
    }

    /// Adds a logical `AND` condition for equal parts.
    #[inline]
    pub fn and_eq<C, V>(self, col: C, value: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        self.push_logical_and(col, "$eq", value.into_sql_value())
    }

    /// Adds a logical `AND` condition for equal parts if the value is not null.
    #[inline]
    pub fn and_eq_if_not_null<C, V>(self, col: C, value: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        let value = value.into_sql_value();
        if !value.is_null() {
            self.push_logical_and(col, "$eq", value)
        } else {
            self
        }
    }

    /// Adds a logical `AND` condition for equal parts if the value is not none.
    #[inline]
    pub fn and_eq_if_some<C, V>(self, col: C, value: Option<V>) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        if let Some(value) = value {
            self.push_logical_and(col, "$eq", value.into_sql_value())
        } else {
            self
        }
    }

    /// Adds a logical `AND` condition for non-equal parts.
    #[inline]
    pub fn and_ne<C, V>(self, col: C, value: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        self.push_logical_and(col, "$ne", value.into_sql_value())
    }

    /// Adds a logical `AND` condition for non-equal parts if the value is not null.
    #[inline]
    pub fn and_ne_if_not_null<C, V>(self, col: C, value: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        let value = value.into_sql_value();
        if !value.is_null() {
            self.push_logical_and(col, "$ne", value)
        } else {
            self
        }
    }

    /// Adds a logical `AND` condition for non-equal parts if the value is not none.
    #[inline]
    pub fn and_ne_if_some<C, V>(self, col: C, value: Option<V>) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        if let Some(value) = value {
            self.push_logical_and(col, "$ne", value.into_sql_value())
        } else {
            self
        }
    }

    /// Adds a logical `AND` condition for the column less than a value.
    #[inline]
    pub fn and_lt<C, V>(self, col: C, value: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        self.push_logical_and(col, "$lt", value.into_sql_value())
    }

    /// Adds a logical `AND` condition for the column not greater than a value.
    #[inline]
    pub fn and_le<C, V>(self, col: C, value: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        self.push_logical_and(col, "$le", value.into_sql_value())
    }

    /// Adds a logical `AND` condition for the column greater than a value.
    #[inline]
    pub fn and_gt<C, V>(self, col: C, value: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        self.push_logical_and(col, "$gt", value.into_sql_value())
    }

    /// Adds a logical `AND` condition for the column not less than a value.
    #[inline]
    pub fn and_ge<C, V>(self, col: C, value: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        self.push_logical_and(col, "$ge", value.into_sql_value())
    }

    /// Adds a logical `AND` condition for the column `IN` a list of values.
    #[inline]
    pub fn and_in<C, T, V>(self, col: C, values: V) -> Self
    where
        C: ModelColumn<E>,
        T: IntoSqlValue,
        V: Into<Vec<T>>,
    {
        self.push_logical_and(col, "$in", values.into().into_sql_value())
    }

    /// Adds a logical `AND` condition for the column `NOT IN` a list of values.
    #[inline]
    pub fn and_not_in<C, T, V>(self, col: C, values: V) -> Self
    where
        C: ModelColumn<E>,
        T: IntoSqlValue,
        V: Into<Vec<T>>,
    {
        self.push_logical_and(col, "$nin", values.into().into_sql_value())
    }

    /// Adds a logical `AND` condition for the column `IN` a list of values
    /// if the list is nonempty.
    #[inline]
    pub fn and_in_if_nonempty<C, T, V>(self, col: C, values: V) -> Self
    where
        C: ModelColumn<E>,
        T: IntoSqlValue,
        V: Into<Vec<T>>,
    {
        let values = values.into();
        if values.is_empty() {
            self
        } else {
            self.push_logical_and(col, "$in", values.into_sql_value())
        }
    }

    /// Adds a logical `AND` condition for the column `NOT IN` a list of values
    /// if the list is nonempty.
    #[inline]
    pub fn and_not_in_if_nonempty<C, T, V>(self, col: C, values: V) -> Self
    where
        C: ModelColumn<E>,
        T: IntoSqlValue,
        V: Into<Vec<T>>,
    {
        let values = values.into();
        if values.is_empty() {
            self
        } else {
            self.push_logical_and(col, "$nin", values.into_sql_value())
        }
    }

    /// Adds a logical `AND` condition for the columns `IN` a subquery.
    pub fn and_in_subquery<C, V, M>(mut self, cols: V, subquery: QueryBuilder<M>) -> Self
    where
        C: ModelColumn<E>,
        V: Into<Vec<C>>,
        M: Entity + Schema,
    {
        let cols = cols
            .into()
            .into_iter()
            .map(|col| {
                let col_name = col.into_column_expr();
                Query::format_field(&col_name).into_owned()
            })
            .collect::<Vec<_>>()
            .join(", ");
        let field = format!("({cols})");
        let condition = Map::from_entry("$in", subquery.into_sql_value());
        self.logical_and.push(Map::from_entry(field, condition));
        self
    }

    /// Adds a logical `AND` condition for the columns `NOT IN` a subquery.
    pub fn and_not_in_subquery<C, V, M>(mut self, cols: V, subquery: QueryBuilder<M>) -> Self
    where
        C: ModelColumn<E>,
        V: Into<Vec<C>>,
        M: Entity + Schema,
    {
        let cols = cols
            .into()
            .into_iter()
            .map(|col| {
                let col_name = col.into_column_expr();
                Query::format_field(&col_name).into_owned()
            })
            .collect::<Vec<_>>()
            .join(", ");
        let field = format!("({cols})");
        let condition = Map::from_entry("$nin", subquery.into_sql_value());
        self.logical_and.push(Map::from_entry(field, condition));
        self
    }

    /// Adds a logical `AND` condition for the column in a range `[min, max)`.
    pub fn and_in_range<C, V>(mut self, col: C, min: V, max: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        let field = col.into_column_expr();
        let mut condition = Map::new();
        condition.upsert("$ge", min.into_sql_value());
        condition.upsert("$lt", max.into_sql_value());
        self.logical_and.push(Map::from_entry(field, condition));
        self
    }

    /// Adds a logical `AND` condition for the column `BETWEEN` two values.
    #[inline]
    pub fn and_between<C, V>(self, col: C, min: V, max: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        self.push_logical_and(col, "$betw", [min, max].into_sql_value())
    }

    /// Adds a logical `AND` condition for the column `LIKE` a string value.
    #[inline]
    pub fn and_like<C: ModelColumn<E>>(self, col: C, value: String) -> Self {
        self.push_logical_and(col, "$like", value.into_sql_value())
    }

    /// Adds a logical `AND` condition for the column `ILIKE` a string value.
    #[inline]
    pub fn and_ilike<C: ModelColumn<E>>(self, col: C, value: String) -> Self {
        self.push_logical_and(col, "$ilike", value.into_sql_value())
    }

    /// Adds a logical `AND` condition for the column `RLIKE` a string value.
    #[inline]
    pub fn and_rlike<C: ModelColumn<E>>(self, col: C, value: String) -> Self {
        self.push_logical_and(col, "$rlike", value.into_sql_value())
    }

    /// Adds a logical `AND` condition for the column which contains a string value.
    #[inline]
    pub fn and_contains<C: ModelColumn<E>>(self, col: C, value: &str) -> Self {
        let value = ["%", value, "%"].concat();
        self.push_logical_and(col, "$like", value.into_sql_value())
    }

    /// Adds a logical `AND` condition for the column which starts with a string value.
    #[inline]
    pub fn and_starts_with<C: ModelColumn<E>>(self, col: C, value: &str) -> Self {
        let value = [value, "%"].concat();
        self.push_logical_and(col, "$like", value.into_sql_value())
    }

    /// Adds a logical `AND` condition for the column which ends with a string value.
    #[inline]
    pub fn and_ends_with<C: ModelColumn<E>>(self, col: C, value: &str) -> Self {
        let value = ["%", value].concat();
        self.push_logical_and(col, "$like", value.into_sql_value())
    }

    /// Adds a logical `AND` condition for the column which is null.
    #[inline]
    pub fn and_null<C: ModelColumn<E>>(self, col: C) -> Self {
        self.and_filter(col, JsonValue::Null)
    }

    /// Adds a logical `AND` condition for the column which is not null.
    #[inline]
    pub fn and_not_null<C: ModelColumn<E>>(self, col: C) -> Self {
        self.and_filter(col, "not_null")
    }

    /// Adds a logical `AND` condition for the column which is an empty string or a null.
    #[inline]
    pub fn and_empty<C: ModelColumn<E>>(self, col: C) -> Self {
        self.and_filter(col, "empty")
    }

    /// Adds a logical `AND` condition for the column which is not an empty string or a null.
    #[inline]
    pub fn and_nonempty<C: ModelColumn<E>>(self, col: C) -> Self {
        self.and_filter(col, "nonempty")
    }

    /// Adds a logical `AND` condition for the two ranges which overlaps with each other.
    pub fn and_overlaps<L, R, V>(mut self, cols: (L, R), values: (V, V)) -> Self
    where
        L: ModelColumn<E>,
        R: ModelColumn<E>,
        V: IntoSqlValue,
    {
        let mut condition = Map::new();
        condition.upsert(
            cols.0.into_column_expr(),
            Map::from_entry("$le", values.1.into_sql_value()),
        );
        condition.upsert(
            cols.1.into_column_expr(),
            Map::from_entry("$ge", values.0.into_sql_value()),
        );
        self.logical_and.push(condition);
        self
    }

    /// Adds a logical `OR` condition by merging the other query builder.
    pub fn or<M: Entity>(mut self, mut other: QueryBuilder<M>) -> Self {
        let mut logical_and = other.logical_and;
        let logical_or = other.logical_or;
        if !logical_or.is_empty() {
            logical_and.push(Map::from_entry("$or", logical_or));
        }
        if !logical_and.is_empty() {
            self.logical_or.push(Map::from_entry("$and", logical_and));
        }
        self.fields.append(&mut other.fields);
        self.group_by_fields.append(&mut other.group_by_fields);
        self
    }

    /// Adds a logical `OR NOT` condition by merging the other query builder.
    pub fn or_not<M: Entity>(mut self, mut other: QueryBuilder<M>) -> Self {
        let mut logical_and = other.logical_and;
        let logical_or = other.logical_or;
        if !logical_or.is_empty() {
            logical_and.push(Map::from_entry("$or", logical_or));
        }
        if !logical_and.is_empty() {
            self.logical_or.push(Map::from_entry("$not", logical_and));
        }
        self.fields.append(&mut other.fields);
        self.group_by_fields.append(&mut other.group_by_fields);
        self
    }

    /// Adds a logical `OR` condition using the value as a filter for the column.
    #[inline]
    pub fn or_filter<C, V>(mut self, col: C, value: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        let condition = Map::from_entry(col.into_column_expr(), value.into_sql_value());
        self.logical_or.push(condition);
        self
    }

    /// Adds a logical `OR` condition for equal parts.
    #[inline]
    pub fn or_eq<C, V>(self, col: C, value: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        self.push_logical_or(col, "$eq", value.into_sql_value())
    }

    /// Adds a logical `OR` condition for equal parts if the value is not null.
    #[inline]
    pub fn or_eq_if_not_null<C, V>(self, col: C, value: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        let value = value.into_sql_value();
        if !value.is_null() {
            self.push_logical_or(col, "$eq", value)
        } else {
            self
        }
    }

    /// Adds a logical `OR` condition for equal parts if the value is not none.
    #[inline]
    pub fn or_eq_if_some<C, V>(self, col: C, value: Option<V>) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        if let Some(value) = value {
            self.push_logical_or(col, "$eq", value.into_sql_value())
        } else {
            self
        }
    }

    /// Adds a logical `OR` condition for non-equal parts.
    #[inline]
    pub fn or_ne<C, V>(self, col: C, value: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        self.push_logical_or(col, "$ne", value.into_sql_value())
    }

    /// Adds a logical `OR` condition for non-equal parts if the value is not none.
    #[inline]
    pub fn or_ne_if_not_null<C, V>(self, col: C, value: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        let value = value.into_sql_value();
        if !value.is_null() {
            self.push_logical_or(col, "$ne", value)
        } else {
            self
        }
    }

    /// Adds a logical `OR` condition for non-equal parts if the value is not none.
    #[inline]
    pub fn or_ne_if_some<C, V>(self, col: C, value: Option<V>) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        if let Some(value) = value {
            self.push_logical_or(col, "$ne", value.into_sql_value())
        } else {
            self
        }
    }

    /// Adds a logical `OR` condition for the column less than a value.
    #[inline]
    pub fn or_lt<C, V>(self, col: C, value: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        self.push_logical_or(col, "$lt", value.into_sql_value())
    }

    /// Adds a logical `OR` condition for the column not greater than a value.
    #[inline]
    pub fn or_le<C, V>(self, col: C, value: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        self.push_logical_or(col, "$le", value.into_sql_value())
    }

    /// Adds a logical `OR` condition for the column greater than a value.
    #[inline]
    pub fn or_gt<C, V>(self, col: C, value: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        self.push_logical_or(col, "$gt", value.into_sql_value())
    }

    /// Adds a logical `OR` condition for the column not less than a value.
    #[inline]
    pub fn or_ge<C, V>(self, col: C, value: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        self.push_logical_or(col, "$ge", value.into_sql_value())
    }

    /// Adds a logical `OR` condition for the column `IN` a list of values.
    #[inline]
    pub fn or_in<C, T, V>(self, col: C, values: V) -> Self
    where
        C: ModelColumn<E>,
        T: IntoSqlValue,
        V: Into<Vec<T>>,
    {
        self.push_logical_or(col, "$in", values.into().into_sql_value())
    }

    /// Adds a logical `OR` condition for the column `NOT IN` a list of values.
    #[inline]
    pub fn or_not_in<C, T, V>(self, col: C, values: V) -> Self
    where
        C: ModelColumn<E>,
        T: IntoSqlValue,
        V: Into<Vec<T>>,
    {
        self.push_logical_or(col, "$nin", values.into().into_sql_value())
    }

    /// Adds a logical `OR` condition for the column `IN` a list of values
    /// if the list is nonempty.
    #[inline]
    pub fn or_in_if_nonempty<C, T, V>(self, col: C, values: V) -> Self
    where
        C: ModelColumn<E>,
        T: IntoSqlValue,
        V: Into<Vec<T>>,
    {
        let values = values.into();
        if values.is_empty() {
            self
        } else {
            self.push_logical_or(col, "$in", values.into_sql_value())
        }
    }

    /// Adds a logical `OR` condition for the column `NOT IN` a list of values
    /// if the list is nonempty.
    #[inline]
    pub fn or_not_in_if_nonempty<C, T, V>(self, col: C, values: V) -> Self
    where
        C: ModelColumn<E>,
        T: IntoSqlValue,
        V: Into<Vec<T>>,
    {
        let values = values.into();
        if values.is_empty() {
            self
        } else {
            self.push_logical_or(col, "$nin", values.into_sql_value())
        }
    }

    /// Adds a logical `OR` condition for the columns `IN` a subquery.
    pub fn or_in_subquery<C, V, M>(mut self, cols: V, subquery: QueryBuilder<M>) -> Self
    where
        C: ModelColumn<E>,
        V: Into<Vec<C>>,
        M: Entity + Schema,
    {
        let cols = cols
            .into()
            .into_iter()
            .map(|col| {
                let col_name = col.into_column_expr();
                Query::format_field(&col_name).into_owned()
            })
            .collect::<Vec<_>>()
            .join(", ");
        let field = format!("({cols})");
        let condition = Map::from_entry("$in", subquery.into_sql_value());
        self.logical_or.push(Map::from_entry(field, condition));
        self
    }

    /// Adds a logical `OR` condition for the columns `NOT IN` a subquery.
    pub fn or_not_in_subquery<C, V, M>(mut self, cols: V, subquery: QueryBuilder<M>) -> Self
    where
        C: ModelColumn<E>,
        V: Into<Vec<C>>,
        M: Entity + Schema,
    {
        let cols = cols
            .into()
            .into_iter()
            .map(|col| {
                let col_name = col.into_column_expr();
                Query::format_field(&col_name).into_owned()
            })
            .collect::<Vec<_>>()
            .join(", ");
        let field = format!("({cols})");
        let condition = Map::from_entry("$nin", subquery.into_sql_value());
        self.logical_or.push(Map::from_entry(field, condition));
        self
    }

    /// Adds a logical `OR` condition for the column is in a range `[min, max)`.
    pub fn or_in_range<C, V>(mut self, col: C, min: V, max: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        let field = col.into_column_expr();
        let mut condition = Map::new();
        condition.upsert("$ge", min.into_sql_value());
        condition.upsert("$lt", max.into_sql_value());
        self.logical_or.push(Map::from_entry(field, condition));
        self
    }

    /// Adds a logical `OR` condition for the column `BETWEEN` two values.
    #[inline]
    pub fn or_between<C, V>(self, col: C, min: V, max: V) -> Self
    where
        C: ModelColumn<E>,
        V: IntoSqlValue,
    {
        self.push_logical_or(col, "$betw", [min, max].into_sql_value())
    }

    /// Adds a logical `OR` condition for the column `LIKE` a string value.
    #[inline]
    pub fn or_like<C: ModelColumn<E>>(self, col: C, value: String) -> Self {
        self.push_logical_or(col, "$like", value.into_sql_value())
    }

    /// Adds a logical `OR` condition for the column `ILIKE` a string value.
    #[inline]
    pub fn or_ilike<C: ModelColumn<E>>(self, col: C, value: String) -> Self {
        self.push_logical_or(col, "$ilike", value.into_sql_value())
    }

    /// Adds a logical `OR` condition for the column `RLIKE` a string value.
    #[inline]
    pub fn or_rlike<C: ModelColumn<E>>(self, col: C, value: String) -> Self {
        self.push_logical_or(col, "$rlike", value.into_sql_value())
    }

    /// Adds a logical `OR` condition for the column which contains a string value.
    #[inline]
    pub fn or_contains<C: ModelColumn<E>>(self, col: C, value: &str) -> Self {
        let value = ["%", value, "%"].concat();
        self.push_logical_or(col, "$like", value.into_sql_value())
    }

    /// Adds a logical `OR` condition for the column which starts with a string value.
    #[inline]
    pub fn or_starts_with<C: ModelColumn<E>>(self, col: C, value: &str) -> Self {
        let value = [value, "%"].concat();
        self.push_logical_or(col, "$like", value.into_sql_value())
    }

    /// Adds a logical `OR` condition for the column which ends with a string value.
    #[inline]
    pub fn or_ends_with<C: ModelColumn<E>>(self, col: C, value: &str) -> Self {
        let value = ["%", value].concat();
        self.push_logical_or(col, "$like", value.into_sql_value())
    }

    /// Adds a logical `OR` condition for the column which is null.
    #[inline]
    pub fn or_null<C: ModelColumn<E>>(self, col: C) -> Self {
        self.or_filter(col, JsonValue::Null)
    }

    /// Adds a logical `OR` condition for the column which is not null.
    #[inline]
    pub fn or_not_null<C: ModelColumn<E>>(self, col: C) -> Self {
        self.or_filter(col, "not_null")
    }

    /// Adds a logical `OR` condition for the column which is an empty string or a null.
    #[inline]
    pub fn or_empty<C: ModelColumn<E>>(self, col: C) -> Self {
        self.or_filter(col, "empty")
    }

    /// Adds a logical `OR` condition for the column which is not an empty string or a null.
    #[inline]
    pub fn or_nonempty<C: ModelColumn<E>>(self, col: C) -> Self {
        self.or_filter(col, "nonempty")
    }

    /// Adds a logical `OR` condition for the two ranges which overlaps with each other.
    pub fn or_overlaps<L, R, V>(mut self, cols: (L, R), values: (V, V)) -> Self
    where
        L: ModelColumn<E>,
        R: ModelColumn<E>,
        V: IntoSqlValue,
    {
        let mut condition = Map::new();
        condition.upsert(
            cols.0.into_column_expr(),
            Map::from_entry("$le", values.1.into_sql_value()),
        );
        condition.upsert(
            cols.1.into_column_expr(),
            Map::from_entry("$ge", values.0.into_sql_value()),
        );
        self.logical_or.push(condition);
        self
    }

    /// Adds a query order.
    #[inline]
    pub fn order_by<C: ModelColumn<E>>(mut self, col: C, descending: bool) -> Self {
        self.sort_order
            .push(QueryOrder::new(col.into_column_expr(), descending));
        self
    }

    /// Adds a query order with an extra flag to indicate whether the nulls appear first or last.
    #[inline]
    pub fn order_by_with_nulls<C: ModelColumn<E>>(
        mut self,
        col: C,
        descending: bool,
        nulls_first: bool,
    ) -> Self {
        let mut order = QueryOrder::new(col.into_column_expr(), descending);
        if nulls_first {
            order.set_nulls_first();
        } else {
            order.set_nulls_last();
        }
        self.sort_order.push(order);
        self
    }

    /// Adds a query order with an ascending order.
    #[inline]
    pub fn order_asc<C: ModelColumn<E>>(mut self, col: C) -> Self {
        self.sort_order
            .push(QueryOrder::new(col.into_column_expr(), false));
        self
    }

    /// Adds a query order with an descending order.
    #[inline]
    pub fn order_desc<C: ModelColumn<E>>(mut self, col: C) -> Self {
        self.sort_order
            .push(QueryOrder::new(col.into_column_expr(), true));
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
    pub fn build(mut self) -> Query {
        let mut filters = self.filters;
        let group_by_fields = self.group_by_fields;
        let having_conditions = self.having_conditions;
        let logical_and = self.logical_and;
        let logical_or = self.logical_or;
        if !group_by_fields.is_empty() {
            filters.upsert("$group", group_by_fields);
        }
        if !having_conditions.is_empty() {
            filters.upsert("$having", having_conditions);
        }
        if !logical_and.is_empty() {
            filters.upsert("$and", logical_and);
        }
        if !logical_or.is_empty() {
            filters.upsert("$or", logical_or);
        }

        let mut query = Query::new(filters);
        query.set_fields(self.fields);
        query.set_order(self.sort_order);
        query.set_offset(self.offset);
        query.set_limit(self.limit);
        query.append_extra_flags(&mut self.extra);
        query
    }

    /// Adds a `HAVING` condition for non-equal parts.
    fn push_having_condition(
        mut self,
        aggregation: Aggregation<E>,
        operator: &str,
        value: JsonValue,
    ) -> Self {
        let condition = Map::from_entry(operator, value);
        self.having_conditions
            .push(Map::from_entry(aggregation.expr(), condition));
        self
    }

    /// Pushes a logical `AND` condition for the column and expressions.
    fn push_logical_and<C: ModelColumn<E>>(
        mut self,
        col: C,
        operator: &str,
        value: JsonValue,
    ) -> Self {
        let condition = Map::from_entry(operator, value);
        self.logical_and
            .push(Map::from_entry(col.into_column_expr(), condition));
        self
    }

    /// Pushes a logical `OR` condition for the column and expressions.
    fn push_logical_or<C: ModelColumn<E>>(
        mut self,
        col: C,
        operator: &str,
        value: JsonValue,
    ) -> Self {
        let condition = Map::from_entry(operator, value);
        self.logical_or
            .push(Map::from_entry(col.into_column_expr(), condition));
        self
    }
}

impl<E: Entity + Schema> QueryBuilder<E> {
    /// Builds a subquery SQL expression.
    #[inline]
    pub fn build_subquery(self) -> String {
        let query = self.build();
        let table_name = query.format_table_name::<E>();
        let projection = query.format_table_fields::<E>();
        let filters = query.format_filters::<E>();
        let sort = query.format_sort();
        let pagination = query.format_pagination();
        format!("(SELECT {projection} FROM {table_name} {filters} {sort} {pagination})")
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
    fn query_order(&self) -> &[QueryOrder];

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

    /// Escapes the table name.
    fn escape_table_name(table_name: &str) -> String;

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
                        let condition = if let Some(subquery) =
                            value.as_object().and_then(|m| m.get_str("$subquery"))
                        {
                            let key = Self::format_field(key);
                            format!(r#"{key} = {subquery}"#)
                        } else {
                            col.format_filter(key, value)
                        };
                        if !condition.is_empty() {
                            logical_and_conditions.push(condition);
                        }
                    } else if key.contains('.') {
                        let condition = Self::format_query_filter::<M>(key, value);
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
                        _ => {
                            if let Some(col) = M::get_column(key) {
                                let condition = if let Some(subquery) =
                                    value.as_object().and_then(|m| m.get_str("$subquery"))
                                {
                                    let key = Self::format_field(key);
                                    format!(r#"{key} = {subquery}"#)
                                } else {
                                    col.format_filter(key, value)
                                };
                                if !condition.is_empty() {
                                    logical_and_conditions.push(condition);
                                }
                            } else if key.contains('.') {
                                let condition = Self::format_query_filter::<M>(key, value);
                                if !condition.is_empty() {
                                    logical_and_conditions.push(condition);
                                }
                            }
                        }
                    }
                }
                if !logical_and_conditions.is_empty() {
                    let condition = Self::join_conditions(logical_and_conditions, " AND ");
                    conditions.push(condition);
                }
            }
        }
        Self::join_conditions(conditions, operator)
    }

    /// Formats a query filter.
    fn format_query_filter<M: Schema>(key: &str, value: &JsonValue) -> String {
        let json_field = key.split_once('.').and_then(|(key, path)| {
            M::get_column(key)
                .filter(|col| col.type_name() == "Map")
                .map(|col| {
                    let key = [M::model_name(), ".", col.name()].concat();
                    let field = Self::format_field(&key);
                    if cfg!(feature = "orm-postgres") {
                        let path = path.replace('.', ", ");
                        format!(r#"({field} #> '{{{path}}}')"#)
                    } else {
                        format!(r#"json_extract({field}, '$.{path}')"#)
                    }
                })
        });
        let requires_json_value = json_field.is_some();
        let field = json_field
            .map(|s| s.into())
            .unwrap_or_else(|| Self::format_field(key));
        if let Some(filter) = value.as_object() {
            let mut conditions = Vec::with_capacity(filter.len());
            for (name, value) in filter {
                let operator = match name.as_str() {
                    "$eq" => "=",
                    "$ne" => "<>",
                    "$lt" => "<",
                    "$le" => "<=",
                    "$gt" => ">",
                    "$ge" => ">=",
                    "$in" => "IN",
                    "$nin" => "NOT IN",
                    _ => "=",
                };
                let condition = if let Some(subquery) =
                    value.as_object().and_then(|m| m.get_str("$subquery"))
                {
                    format!(r#"{field} {operator} {subquery}"#)
                } else if requires_json_value {
                    Self::format_json_filter(&field, operator, value)
                } else if let Some(s) = value.as_str() {
                    if name == "$subquery" {
                        format!(r#"{field} {operator} {s}"#)
                    } else {
                        let value = Self::escape_string(s);
                        format!(r#"{field} {operator} {value}"#)
                    }
                } else {
                    format!(r#"{field} {operator} {value}"#)
                };
                conditions.push(condition);
            }
            Self::join_conditions(conditions, " AND ")
        } else if requires_json_value {
            Self::format_json_filter(&field, "=", value)
        } else if let Some(s) = value.as_str() {
            let value = Self::escape_string(s);
            format!(r#"{field} = {value}"#)
        } else {
            format!(r#"{field} = {value}"#)
        }
    }

    /// Formats the filter for a JSON field.
    fn format_json_filter(field: &str, operator: &str, value: &JsonValue) -> String {
        match value {
            JsonValue::Null => format!(r#"{field} IS NULL"#),
            JsonValue::Bool(b) => {
                let value = if *b { "TRUE" } else { "FALSE" };
                if cfg!(feature = "orm-postgres") {
                    format!(r#"({field})::boolean IS {value}"#)
                } else {
                    format!(r#"{field} = {value}"#)
                }
            }
            JsonValue::Number(n) => {
                if cfg!(feature = "orm-postgres") {
                    format!(r#"{field} {operator} '{n}'"#)
                } else {
                    format!(r#"{field} {operator} {n}"#)
                }
            }
            JsonValue::String(s) => {
                if s == "null" {
                    format!(r#"{field} IS NULL"#)
                } else if s == "not_null" {
                    format!(r#"{field} IS NOT NULL"#)
                } else if s == "true" || s == "false" {
                    if cfg!(feature = "orm-postgres") {
                        format!(r#"({field})::boolean IS {s}"#)
                    } else {
                        format!(r#"{field} = {s}"#)
                    }
                } else if let Ok(n) = s.parse::<serde_json::Number>() {
                    if cfg!(feature = "orm-postgres") {
                        format!(r#"{field} {operator} '{n}'"#)
                    } else {
                        format!(r#"{field} {operator} {n}"#)
                    }
                } else {
                    let value = if cfg!(feature = "orm-postgres") {
                        Self::escape_string(value)
                    } else {
                        Self::escape_string(s)
                    };
                    format!(r#"{field} {operator} {value}"#)
                }
            }
            _ => {
                let value = Self::escape_string(value);
                format!(r#"{field} {operator} {value}"#)
            }
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
                .map(|order| {
                    let sort_field = Self::format_field(order.field());
                    let mut expr = if order.is_descending() {
                        format!("{sort_field} DESC")
                    } else {
                        format!("{sort_field} ASC")
                    };
                    if order.nulls_first() {
                        expr.push_str(" NULLS FIRST");
                    } else if order.nulls_last() {
                        expr.push_str(" NULLS LAST");
                    }
                    expr
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

    /// Joins the conditions.
    fn join_conditions(mut conditions: Vec<String>, operator: &str) -> String {
        match conditions.len() {
            0 => String::new(),
            1 => conditions.remove(0),
            _ => format!("({})", conditions.join(operator)),
        }
    }
}

/// Formats the query using interpolation of the parameters.
///
/// The interpolation parameter is represented as `${param}`,
/// in which `param` can only contain restricted chracters `[a-zA-Z]+[\w\.]*`.
pub(crate) fn format_query<'a>(query: &'a str, params: Option<&'a Map>) -> Cow<'a, str> {
    if let Some(params) = params.filter(|_| query.contains('$')) {
        INTERPOLATION_PATTERN.replace_all(query, |captures: &Captures| {
            let key = &captures[1];
            params
                .get(key)
                .map(|value| match value {
                    JsonValue::String(s) => s.to_owned(),
                    _ => value.to_string(),
                })
                .unwrap_or_else(|| ["${", key, "}"].concat())
        })
    } else {
        Cow::Borrowed(query)
    }
}

/// Prepares the SQL query for binding parameters
/// (`?` for most SQL flavors and `$N` for PostgreSQL).
///
/// The parameter is represented as `${param}` or `#{param}`,
/// in which `param` can only contain restricted chracters `[a-zA-Z]+[\w\.]*`.
pub(crate) fn prepare_sql_query<'a>(
    query: &'a str,
    params: Option<&'a Map>,
    placeholder: char,
) -> (Cow<'a, str>, Vec<&'a JsonValue>) {
    let sql = format_query(query, params);
    if let Some(params) = params.filter(|_| sql.contains('#')) {
        let mut values = Vec::new();
        let sql = STATEMENT_PATTERN.replace_all(&sql, |captures: &Captures| {
            let key = &captures[1];
            let value = params.get(key).unwrap_or(&JsonValue::Null);
            values.push(value);
            if placeholder == '$' {
                Cow::Owned(format!("${}", values.len()))
            } else {
                Cow::Borrowed("?")
            }
        });
        (sql.into_owned().into(), values)
    } else {
        (sql, Vec::new())
    }
}

/// Regex for the interpolation parameter.
static INTERPOLATION_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\$\{\s*([a-zA-Z]+[\w\.]*)\s*\}")
        .expect("fail to create a regex for the interpolation parameter")
});

/// Regex for the prepared statement.
static STATEMENT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\#\{\s*([a-zA-Z]+[\w\.]*)\s*\}")
        .expect("fail to create a regex for the prepared statement")
});

#[cfg(test)]
mod tests {
    use zino_core::{Map, extension::JsonObjectExt};

    #[test]
    fn it_formats_query_params() {
        let query = "SELECT ${fields} FROM users WHERE name = 'alice' AND age >= #{age};";
        let mut params = Map::new();
        params.upsert("fields", "id, name, age");
        params.upsert("age", 18);

        let sql = super::format_query(query, Some(&params));
        assert_eq!(
            sql,
            "SELECT id, name, age FROM users WHERE name = 'alice' AND age >= #{age};"
        );
    }

    #[test]
    fn it_formats_sql_query_params() {
        let query = "SELECT ${fields} FROM users WHERE name = 'alice' AND age >= #{age};";
        let mut params = Map::new();
        params.upsert("fields", "id, name, age");
        params.upsert("age", 18);

        let (sql, values) = super::prepare_sql_query(query, Some(&params), '?');
        assert_eq!(
            sql,
            "SELECT id, name, age FROM users WHERE name = 'alice' AND age >= ?;"
        );
        assert_eq!(values[0], 18);

        let (sql, values) = super::prepare_sql_query(query, Some(&params), '$');
        assert_eq!(
            sql,
            "SELECT id, name, age FROM users WHERE name = 'alice' AND age >= $1;"
        );
        assert_eq!(values[0], 18);
    }
}
