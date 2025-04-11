use self::Aggregation::*;
use super::{Entity, query::QueryExt};
use zino_core::model::Query;

/// SQL aggregate functions.
///
/// # Examples
/// ```rust,ignore
/// use crate::model::{Task, TaskColumn};
/// use zino_core::Map;
/// use zino_orm::{Aggregation, QueryBuilder, Schema};
///
/// let query = QueryBuilder::new()
///     .aggregate(Aggregation::Count(TaskColumn::Id, false), Some("num_tasks"))
///     .aggregate(Aggregation::Sum(TaskColumn::Manhours), Some("total_manhours"))
///     .aggregate(Aggregation::Avg(TaskColumn::Manhours), Some("average_manhours"))
///     .and_eq(TaskColumn::Status, "Completed")
///     .group_by(TaskColumn::ProjectId, None)
///     .having_ge(Aggregation::Avg(TaskColumn::Manhours), 50)
///     .order_desc("total_manhours")
///     .limit(10)
///     .build();
/// let entries = Task::aggregate::<Map>(&query).await?;
/// ```
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Aggregation<E: Entity> {
    /// The `COUNT` function with the `DISTINCT` modifier.
    Count(E::Column, bool),
    /// The `SUM` function.
    Sum(E::Column),
    /// The `AVG` function.
    Avg(E::Column),
    /// The `MIN` function.
    Min(E::Column),
    /// The `MAX` function.
    Max(E::Column),
    /// The `STDDEV` function.
    Stddev(E::Column),
    /// The `VARIANCE` function.
    Variance(E::Column),
    /// The `JSON_ARRAYAGG` function.
    JsonArrayagg(E::Column),
    /// The `JSON_OBJECTAGG` function.
    JsonObjectagg(E::Column, E::Column),
}

impl<E: Entity> Aggregation<E> {
    /// Returns a default alias for the aggregation.
    pub(super) fn default_alias(&self) -> String {
        match self {
            Count(col, distinct) => {
                if *distinct {
                    [col.as_ref(), "_distinct"].concat()
                } else {
                    [col.as_ref(), "_count"].concat()
                }
            }
            Sum(col) => [col.as_ref(), "_sum"].concat(),
            Avg(col) => [col.as_ref(), "_avg"].concat(),
            Min(col) => [col.as_ref(), "_min"].concat(),
            Max(col) => [col.as_ref(), "_max"].concat(),
            Stddev(col) => [col.as_ref(), "_stddev"].concat(),
            Variance(col) => [col.as_ref(), "_variance"].concat(),
            JsonArrayagg(col) => [col.as_ref(), "_arrayagg"].concat(),
            JsonObjectagg(key_col, val_col) => {
                [key_col.as_ref(), "_", val_col.as_ref(), "_objectagg"].concat()
            }
        }
    }

    /// Returns the SQL expression.
    pub(super) fn expr(&self) -> String {
        match self {
            Count(col, distinct) => {
                let col_name = E::format_column(col);
                let field = Query::format_field(&col_name);
                if *distinct {
                    format!("count(distinct {field})")
                } else {
                    format!("count({field})")
                }
            }
            Sum(col) => {
                let col_name = E::format_column(col);
                let field = Query::format_field(&col_name);
                format!("sum({field})")
            }
            Avg(col) => {
                let col_name = E::format_column(col);
                let field = Query::format_field(&col_name);
                format!("avg({field})")
            }
            Min(col) => {
                let col_name = E::format_column(col);
                let field = Query::format_field(&col_name);
                format!("min({field})")
            }
            Max(col) => {
                let col_name = E::format_column(col);
                let field = Query::format_field(&col_name);
                format!("max({field})")
            }
            Stddev(col) => {
                let col_name = E::format_column(col);
                let field = Query::format_field(&col_name);
                format!("stddev({field})")
            }
            Variance(col) => {
                let col_name = E::format_column(col);
                let field = Query::format_field(&col_name);
                format!("variance({field})")
            }
            JsonArrayagg(col) => {
                let col_name = E::format_column(col);
                let field = Query::format_field(&col_name);
                if cfg!(any(
                    feature = "orm-mariadb",
                    feature = "orm-mysql",
                    feature = "orm-tidb"
                )) {
                    format!("json_arrayagg({field})")
                } else if cfg!(feature = "orm-postgres") {
                    format!("jsonb_agg({field})")
                } else {
                    format!("json_group_array({field})")
                }
            }
            JsonObjectagg(key_col, val_col) => {
                let key_col_name = E::format_column(key_col);
                let val_col_name = E::format_column(val_col);
                let key_field = Query::format_field(&key_col_name);
                let val_field = Query::format_field(&val_col_name);
                if cfg!(any(
                    feature = "orm-mariadb",
                    feature = "orm-mysql",
                    feature = "orm-tidb"
                )) {
                    format!("json_objectagg({key_field}, {val_field})")
                } else if cfg!(feature = "orm-postgres") {
                    format!("jsonb_object_agg({key_field}, {val_field})")
                } else {
                    format!("json_group_object({key_field}, {val_field})")
                }
            }
        }
    }
}
