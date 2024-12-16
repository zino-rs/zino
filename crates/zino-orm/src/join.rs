use super::{query::QueryExt, Entity, Schema};
use std::marker::PhantomData;
use zino_core::model::Query;

/// Variants for `JOIN` types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum JoinType {
    /// The `INNER JOIN` type.
    #[default]
    Inner,
    /// The `LEFT (OUTER) JOIN` type.
    Left,
    /// The `RIGHT (OUTER) JOIN` type.
    Right,
    /// The `FULL (OUTER) JOIN` type.
    Full,
    /// The `CROSS JOIN` type.
    Cross,
}

impl JoinType {
    /// Returns the join type as str.
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            JoinType::Inner => "INNER JOIN",
            JoinType::Left => "LEFT JOIN",
            JoinType::Right => "RIGHT JOIN",
            JoinType::Full => "FULL JOIN",
            JoinType::Cross => "CROSS JOIN",
        }
    }
}

/// SQL joins on two tables.
///
/// # Examples
/// ```rust,ignore
/// use crate::model::{Project, ProjectColumn, Task, TaskColumn};
/// use zino_core::Map;
/// use zino_orm::{JoinOn, QueryBuilder, Schema};
///
/// let query = QueryBuilder::<Task>::new()
///     .fields([TaskColumn::Id, TaskColumn::Name, TaskColumn::ProjectId])
///     .and_eq(TaskColumn::Status, "Completed")
///     .and(
///         QueryBuilder::<Project>::new()
///             .alias(ProjectColumn::Name, "project_name")
///             .fields([ProjectColumn::StartDate, ProjectColumn::EndDate])
///             .and_overlaps(
///                 (ProjectColumn::StartDate, ProjectColumn::EndDate),
///                 ("2023-10-01", "2023-10-07"),
///             ),
///     )
///     .order_desc(TaskColumn::UpdatedAt)
///     .build();
/// let join_on = JoinOn::<Task, Project>::new()
///     .eq(TaskColumn::ProjectId, ProjectColumn::Id);
/// let entries = Task::lookup::<Project, Map>(&query, &join_on).await?;
/// ```
#[derive(Debug, Clone, Default)]
pub struct JoinOn<L: Schema, R: Schema> {
    /// The join type.
    join_type: JoinType,
    /// The join conditions.
    conditions: Vec<String>,
    /// The phantom data.
    phantom: PhantomData<(L, R)>,
}

impl<L: Schema, R: Schema> JoinOn<L, R> {
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs an instance for the `INNER JOIN`.
    #[inline]
    pub fn inner_join() -> Self {
        Self {
            join_type: JoinType::Inner,
            conditions: Vec::new(),
            phantom: PhantomData,
        }
    }

    /// Constructs an instance for the `LEFT (OUTER) JOIN`.
    #[inline]
    pub fn left_join() -> Self {
        Self {
            join_type: JoinType::Left,
            conditions: Vec::new(),
            phantom: PhantomData,
        }
    }

    /// Constructs an instance for the `RIGHT (OUTER) JOIN`.
    #[inline]
    pub fn right_join() -> Self {
        Self {
            join_type: JoinType::Right,
            conditions: Vec::new(),
            phantom: PhantomData,
        }
    }

    /// Constructs an instance for the `FULL (OUTER) JOIN`.
    #[inline]
    pub fn full_join() -> Self {
        Self {
            join_type: JoinType::Full,
            conditions: Vec::new(),
            phantom: PhantomData,
        }
    }

    /// Constructs an instance for the `CROSS JOIN`.
    #[inline]
    pub fn cross_join() -> Self {
        Self {
            join_type: JoinType::Cross,
            conditions: Vec::new(),
            phantom: PhantomData,
        }
    }

    /// Specifies an equality relation for the two columns.
    pub fn with(mut self, left_col: &str, right_col: &str) -> Self {
        let left_col = [L::model_name(), ".", left_col].concat();
        let right_col = [R::model_name(), ".", right_col].concat();
        let left_col_field = Query::format_field(&left_col);
        let right_col_field = Query::format_field(&right_col);
        let condition = format!("{left_col_field} = {right_col_field}");
        self.conditions.push(condition);
        self
    }

    /// Returns the join type.
    #[inline]
    pub(super) fn join_type(&self) -> JoinType {
        self.join_type
    }

    /// Formats the conditions.
    #[inline]
    pub(super) fn format_conditions(&self) -> String {
        self.conditions.join(" AND ")
    }
}

impl<L: Entity + Schema, R: Entity + Schema> JoinOn<L, R> {
    /// Specifies a relation for which the left column is equal to the right column.
    #[inline]
    pub fn eq(self, left_col: L::Column, right_col: R::Column) -> Self {
        self.push_op(left_col, "=", right_col)
    }

    /// Specifies a relation for which the left column is not equal to the right column.
    #[inline]
    pub fn ne(self, left_col: L::Column, right_col: R::Column) -> Self {
        self.push_op(left_col, "<>", right_col)
    }

    /// Specifies a relation for which the left column is less than the right column.
    #[inline]
    pub fn lt(self, left_col: L::Column, right_col: R::Column) -> Self {
        self.push_op(left_col, "<", right_col)
    }

    /// Specifies a relation for which the left column is not greater than the right column.
    #[inline]
    pub fn le(self, left_col: L::Column, right_col: R::Column) -> Self {
        self.push_op(left_col, "<=", right_col)
    }

    /// Specifies a relation for which the left column is greater than the right column.
    #[inline]
    pub fn gt(self, left_col: L::Column, right_col: R::Column) -> Self {
        self.push_op(left_col, ">", right_col)
    }

    /// Specifies a relation for which the left column is not less than the right column.
    #[inline]
    pub fn ge(self, left_col: L::Column, right_col: R::Column) -> Self {
        self.push_op(left_col, ">=", right_col)
    }

    /// Pushes a condition for the two columns.
    fn push_op(mut self, left_col: L::Column, operator: &str, right_col: R::Column) -> Self {
        let left_col = L::format_column(&left_col);
        let right_col = R::format_column(&right_col);
        let left_col_field = Query::format_field(&left_col);
        let right_col_field = Query::format_field(&right_col);
        let condition = format!("{left_col_field} {operator} {right_col_field}");
        self.conditions.push(condition);
        self
    }
}
