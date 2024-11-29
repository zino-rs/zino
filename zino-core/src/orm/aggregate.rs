use self::Aggregation::*;
use super::{query::QueryExt, Entity};
use crate::model::Query;

/// SQL aggregate functions.
///
/// # Examples
/// ```rust,ignore
/// use crate::model::{Task, TaskColumn};
/// use zino_core::{orm::{Aggregation, QueryBuilder, Schema}, Map};
///
/// let query = QueryBuilder::<Task>::new()
///     .aggregate(Aggregation::Count(TaskColumn::Id, false), Some("num_tasks"))
///     .aggregate(Aggregation::Sum(TaskColumn::Manhours), Some("total_manhours"))
///     .aggregate(Aggregation::Avg(TaskColumn::Manhours), Some("average_manhours"))
///     .and_eq(TaskColumn::Status, "Completed")
///     .group_by(TaskColumn::ProjectId)
///     .having_ge(Aggregation::Avg(TaskColumn::Manhours), 50)
///     .order_desc("total_manhours")
///     .limit(10)
///     .build();
/// let entries = Task::aggregate::<Map>(&query).await?;
/// ```
#[derive(Debug, Clone, Copy)]
pub enum Aggregation<E: Entity> {
    /// A `COUNT` function with the `DISTINCT` modifier.
    Count(E::Column, bool),
    /// A `SUM` function.
    Sum(E::Column),
    /// An `AVG` function.
    Avg(E::Column),
    /// A `Min` function.
    Min(E::Column),
    /// A `Max` function.
    Max(E::Column),
}

impl<E: Entity> Aggregation<E> {
    /// Returns a default alias for the aggregation.
    pub(super) fn default_alias(&self) -> String {
        match self {
            Count(col, distinct) => {
                if *distinct {
                    [col.as_ref(), "_", "distinct"].concat()
                } else {
                    [col.as_ref(), "_", "count"].concat()
                }
            }
            Sum(col) => [col.as_ref(), "_", "sum"].concat(),
            Avg(col) => [col.as_ref(), "_", "avg"].concat(),
            Min(col) => [col.as_ref(), "_", "min"].concat(),
            Max(col) => [col.as_ref(), "_", "col"].concat(),
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
        }
    }
}
