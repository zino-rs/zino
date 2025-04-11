use super::{Entity, ModelColumn, Schema, query::QueryExt};
use zino_core::model::Query;

/// Variants for `JOIN` types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum JoinType {
    /// The default join type.
    #[default]
    Default,
    /// The `INNER JOIN` type.
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
    pub(super) fn as_str(&self) -> &'static str {
        match self {
            JoinType::Default => "JOIN",
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
/// let query = QueryBuilder::new()
///     .fields([TaskColumn::Id, TaskColumn::Name, TaskColumn::ProjectId])
///     .and_eq(TaskColumn::Status, "Completed")
///     .and(
///         QueryBuilder::new()
///             .alias(ProjectColumn::Name, "project_name")
///             .fields([ProjectColumn::StartDate, ProjectColumn::EndDate])
///             .and_overlaps(
///                 (ProjectColumn::StartDate, ProjectColumn::EndDate),
///                 ("2023-10-01", "2023-10-07"),
///             ),
///     )
///     .order_desc(TaskColumn::UpdatedAt)
///     .build();
/// let join_on = JoinOn::new::<Project>()
///     .eq(TaskColumn::ProjectId, ProjectColumn::Id);
/// let entries = Task::lookup::<Project, Map>(&query, &[join_on]).await?;
/// ```
#[derive(Debug, Clone)]
pub struct JoinOn {
    /// The join type.
    join_type: JoinType,
    /// The join table.
    join_table: String,
    /// The join conditions.
    conditions: Vec<String>,
}

impl JoinOn {
    /// Creates a new instance with the default join type.
    #[inline]
    pub fn new<M: Schema>() -> Self {
        Self {
            join_type: JoinType::default(),
            join_table: Self::format_join_table::<M>(),
            conditions: Vec::new(),
        }
    }

    /// Constructs an instance with the `INNER JOIN` type.
    #[inline]
    pub fn inner_join<M: Schema>() -> Self {
        Self {
            join_type: JoinType::Inner,
            join_table: Self::format_join_table::<M>(),
            conditions: Vec::new(),
        }
    }

    /// Constructs an instance with the `LEFT (OUTER) JOIN` type.
    #[inline]
    pub fn left_join<M: Schema>() -> Self {
        Self {
            join_type: JoinType::Left,
            join_table: Self::format_join_table::<M>(),
            conditions: Vec::new(),
        }
    }

    /// Constructs an instance with the `RIGHT (OUTER) JOIN` type.
    #[inline]
    pub fn right_join<M: Schema>() -> Self {
        Self {
            join_type: JoinType::Right,
            join_table: Self::format_join_table::<M>(),
            conditions: Vec::new(),
        }
    }

    /// Constructs an instance with the `FULL (OUTER) JOIN` type.
    #[inline]
    pub fn full_join<M: Schema>() -> Self {
        Self {
            join_type: JoinType::Full,
            join_table: Self::format_join_table::<M>(),
            conditions: Vec::new(),
        }
    }

    /// Constructs an instance with the `CROSS JOIN` type.
    #[inline]
    pub fn cross_join<M: Schema>() -> Self {
        Self {
            join_type: JoinType::Cross,
            join_table: Self::format_join_table::<M>(),
            conditions: Vec::new(),
        }
    }

    /// Specifies a relation for which the left column is equal to the right column.
    #[inline]
    pub fn eq<E1, E2, C1, C2>(self, left_col: C1, right_col: C2) -> Self
    where
        E1: Entity,
        E2: Entity,
        C1: ModelColumn<E1>,
        C2: ModelColumn<E2>,
    {
        self.push_op::<E1, E2, C1, C2>(left_col, "=", right_col)
    }

    /// Specifies a relation for which the left column is not equal to the right column.
    #[inline]
    pub fn ne<E1, E2, C1, C2>(self, left_col: C1, right_col: C2) -> Self
    where
        E1: Entity,
        E2: Entity,
        C1: ModelColumn<E1>,
        C2: ModelColumn<E2>,
    {
        self.push_op::<E1, E2, C1, C2>(left_col, "<>", right_col)
    }

    /// Specifies a relation for which the left column is less than the right column.
    #[inline]
    pub fn lt<E1, E2, C1, C2>(self, left_col: C1, right_col: C2) -> Self
    where
        E1: Entity,
        E2: Entity,
        C1: ModelColumn<E1>,
        C2: ModelColumn<E2>,
    {
        self.push_op::<E1, E2, C1, C2>(left_col, "<", right_col)
    }

    /// Specifies a relation for which the left column is not greater than the right column.
    #[inline]
    pub fn le<E1, E2, C1, C2>(self, left_col: C1, right_col: C2) -> Self
    where
        E1: Entity,
        E2: Entity,
        C1: ModelColumn<E1>,
        C2: ModelColumn<E2>,
    {
        self.push_op::<E1, E2, C1, C2>(left_col, "<=", right_col)
    }

    /// Specifies a relation for which the left column is greater than the right column.
    #[inline]
    pub fn gt<E1, E2, C1, C2>(self, left_col: C1, right_col: C2) -> Self
    where
        E1: Entity,
        E2: Entity,
        C1: ModelColumn<E1>,
        C2: ModelColumn<E2>,
    {
        self.push_op::<E1, E2, C1, C2>(left_col, ">", right_col)
    }

    /// Specifies a relation for which the left column is not less than the right column.
    #[inline]
    pub fn ge<E1, E2, C1, C2>(self, left_col: C1, right_col: C2) -> Self
    where
        E1: Entity,
        E2: Entity,
        C1: ModelColumn<E1>,
        C2: ModelColumn<E2>,
    {
        self.push_op::<E1, E2, C1, C2>(left_col, ">=", right_col)
    }

    /// Returns the join type.
    #[inline]
    pub(super) fn join_type(&self) -> JoinType {
        self.join_type
    }

    /// Returns the join table name.
    #[inline]
    pub(super) fn join_table(&self) -> &str {
        &self.join_table
    }

    /// Formats the conditions.
    #[inline]
    pub(super) fn format_conditions(&self) -> String {
        self.conditions.join(" AND ")
    }

    /// Formats the join table.
    #[inline]
    fn format_join_table<M: Schema>() -> String {
        let table_name = Query::escape_table_name(M::table_name());
        let model_name = Query::escape_table_name(M::model_name());
        format!("{table_name} AS {model_name}")
    }

    /// Pushes a condition for the two columns.
    fn push_op<E1, E2, C1, C2>(mut self, left_col: C1, operator: &str, right_col: C2) -> Self
    where
        E1: Entity,
        E2: Entity,
        C1: ModelColumn<E1>,
        C2: ModelColumn<E2>,
    {
        let left_col = left_col.into_column_expr();
        let right_col = right_col.into_column_expr();
        let left_col_field = Query::format_field(&left_col);
        let right_col_field = Query::format_field(&right_col);
        let condition = format!("{left_col_field} {operator} {right_col_field}");
        self.conditions.push(condition);
        self
    }
}
